use core::fmt;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::multispace0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::{IResult, Parser};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseTree<T> {
    pub root: T,
    pub descendants: Descendants<T>,
}

impl ParseTree<String> {
    fn print(&self) -> String {
        match &self.descendants {
            Descendants::Atom(atom) => format!("({} {})", self.root, atom),
            Descendants::Expressions(trees) => {
                let tree_list = trees
                    .iter()
                    .map(|tree| Self::print(tree))
                    .collect::<Vec<String>>()
                    .join(" ");

                format!("({} {})", self.root, tree_list)
            }
        }
    }
}

impl fmt::Display for ParseTree<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Descendants<T> {
    Atom(T),
    Expressions(Vec<ParseTree<T>>),
}

trait PTBExpressionParser {
    fn parse(s: &str) -> Result<ParseTree<String>, nom::error::Error<String>> {
        match Self::expression.parse(s) {
            Ok((_input, tree)) => Ok(tree),
            Err(e) => match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => Err(nom::error::Error {
                    input: String::from(e.input),
                    code: e.code,
                }),
                nom::Err::Incomplete(e) => Err(nom::error::Error {
                    input: String::from(s.clone()),
                    code: nom::error::ErrorKind::Fail,
                }),
            },
        }
    }
    fn atom(input: &str) -> IResult<&str, Descendants<String>>;
    fn head(input: &str) -> IResult<&str, String>;
    fn expression_list(input: &str) -> IResult<&str, Descendants<String>>;
    fn expression(input: &str) -> IResult<&str, ParseTree<String>>;
}

pub struct PTBParser;

impl PTBExpressionParser for PTBParser {
    fn atom(input: &str) -> IResult<&str, Descendants<String>> {
        let (input, atom) = delimited(multispace0, is_not(" ()"), multispace0).parse(input)?;

        Ok((input, Descendants::Atom(String::from(atom))))
    }

    fn head(input: &str) -> IResult<&str, String> {
        let (input, atom) = delimited(multispace0, is_not(" ()"), multispace0).parse(input)?;

        Ok((input, String::from(atom)))
    }

    fn expression_list(input: &str) -> IResult<&str, Descendants<String>> {
        let (input, expressions) =
            many1(delimited(multispace0, Self::expression, multispace0)).parse(input)?;

        Ok((input, Descendants::Expressions(expressions)))
    }

    fn expression(input: &str) -> IResult<&str, ParseTree<String>> {
        let lparen = delimited(multispace0, tag("("), multispace0);
        let rparen = delimited(multispace0, tag(")"), multispace0);

        let (input, (head, tail)) = delimited(
            lparen,
            (Self::head, alt((Self::atom, Self::expression_list))),
            rparen,
        )
        .parse(input)?;

        Ok((
            input,
            ParseTree {
                root: head,
                descendants: tail,
            },
        ))
    }
}

impl PTBParser {
    pub fn parse(s: &str) -> Result<ParseTree<String>, nom::error::Error<String>> {
        <Self as PTBExpressionParser>::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use nom::ErrorConvert;

    use super::*;

    #[test]
    fn parses_correct_tree() {
        let input = "(ROOT (S (NP (NNP A)) (VP (VB screams))))";
        let tree = PTBParser::parse(input).expect("should be parsable");

        assert_eq!(
            tree,
            ParseTree {
                root: "ROOT".to_string(),
                descendants: Descendants::Expressions(vec![ParseTree {
                    root: "S".to_string(),
                    descendants: Descendants::Expressions(vec![
                        ParseTree {
                            root: "NP".to_string(),
                            descendants: Descendants::Expressions(vec![ParseTree {
                                root: "NNP".to_string(),
                                descendants: Descendants::Atom("A".to_string())
                            }])
                        },
                        ParseTree {
                            root: "VP".to_string(),
                            descendants: Descendants::Expressions(vec![ParseTree {
                                root: "VB".to_string(),
                                descendants: Descendants::Atom("screams".to_string())
                            }])
                        }
                    ])
                }])
            }
        )
    }

    #[test]
    fn empty_input() {
        let input = "";
        let _err = PTBParser::parse(input).expect_err("This should not be parsable");
    }

    #[test]
    fn erroneous_input() {
        let input = "(((NP)";
        let _err = PTBParser::parse(input).expect_err("This should not be parsable");
    }

    #[test]
    fn minimal_input() {
        let input = "(A A)";
        let tree = PTBParser::parse(input).expect("This should be parsable");
        assert_eq!(
            tree,
            ParseTree {
                root: "A".to_string(),
                descendants: Descendants::Atom("A".to_string())
            }
        )
    }

    #[test]
    fn generates_input_ptb_string() {
        let input = "(ROOT (S (NP-SBJ (NP (NNP Pierre) (NNP Vinken)) (, ,) (ADJP (NP (CD 61) (NNS years)) (JJ old)) (, ,)) (VP (MD will) (VP (VB join) (NP (DT the) (NN board)) (PP-CLR (IN as) (NP (DT a) (JJ nonexecutive) (NN director))) (NP-TMP (NNP Nov.) (CD 29)))) (. .)))";
        let output = PTBParser::parse(input)
            .expect("This should be parsable")
            .print();

        assert_eq!(input, output)
    }

    #[test]
    fn wild_spaces() {
        let input = " (S (NP ( NP \t John   ) (NP Maria )) ) ";
        let tree = PTBParser::parse(input).expect("This should be parsable");
        assert_eq!(
            tree,
            ParseTree {
                root: "S".to_string(),
                descendants: Descendants::Expressions(vec![ParseTree {
                    root: "NP".to_string(),
                    descendants: Descendants::Expressions(vec![
                        ParseTree {
                            root: "NP".to_string(),
                            descendants: Descendants::Atom("John".to_string())
                        },
                        ParseTree {
                            root: "NP".to_string(),
                            descendants: Descendants::Atom("Maria".to_string())
                        }
                    ])
                }])
            }
        )
    }
}
