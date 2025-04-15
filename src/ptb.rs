use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::multispace0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::{IResult, Parser};

#[derive(Debug)]
pub struct ParseTree<T> {
    root: T,
    descendants: Descendants<T>,
}

#[derive(Debug)]
pub enum Descendants<T> {
    Atom(T),
    Expressions(Vec<ParseTree<T>>),
}

trait PTBExpressionParser {
    fn parse(s: String) -> Result<ParseTree<String>, nom::error::Error<String>> {
        match Self::expression.parse(s.as_str()) {
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
        let (input, (head, tail)) = delimited(
            tag("("),
            (Self::head, alt((Self::atom, Self::expression_list))),
            tag(")"),
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
    pub fn parse(s: String) -> Result<ParseTree<String>, nom::error::Error<String>> {
        <Self as PTBExpressionParser>::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let input = String::from("(ROOT (S (NP-SBJ (NP (NNP Pierre) (NNP Vinken)) (, ,) (ADJP (NP (CD 61) (NNS years)) (JJ old)) (, ,)) (VP (MD will) (VP (VB join) (NP (DT the) (NN board)) (PP-CLR (IN as) (NP (DT a) (JJ nonexecutive) (NN director))) (NP-TMP (NNP Nov.) (CD 29)))) (. .)))");
        let tree = PTBParser::parse(input).expect("should be parsable");
        println!("{:?}", tree);
    }
}
