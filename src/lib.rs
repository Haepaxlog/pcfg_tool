use core::fmt;
use std::collections::HashMap;

use crate::berkeley::BerkeleyFormatWriter;
use berkeley::BerkeleyWriter;

pub mod berkeley;
pub mod cli;
pub mod induce;
pub mod ptb;

type Nonterminal = String;
type Terminal = String;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum Body {
    Lexical(Terminal),
    NonLexical(Vec<Nonterminal>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Rule {
    head: Nonterminal,
    body: Body,
}

impl Rule {
    pub fn is_lexical_rule(&self) -> bool {
        match self.body {
            Body::Lexical(_) => true,
            Body::NonLexical(_) => false,
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            Body::Lexical(terminal) => write!(f, "{} -> {}", self.head, terminal),
            Body::NonLexical(nonterminals) => {
                write!(f, "{} -> ", self.head)?;
                for (index, nonterminal) in nonterminals.iter().enumerate() {
                    if index != nonterminals.len() - 1 {
                        write!(f, "{} ", nonterminal)?;
                    } else {
                        write!(f, "{}", nonterminal)?;
                    }
                }
                Ok(())
            }
        }
    }
}

type Probability = f64;
type Occurence = u32;

type ProbabilityRules = HashMap<Rule, Probability>;
type OccurenceRules = HashMap<Rule, Occurence>;

#[derive(Debug, PartialEq, Clone)]
pub struct Grammar {
    initial: Nonterminal,
    rules: ProbabilityRules,
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let berkeley_writer = BerkeleyWriter::from_grammar((*self).clone());
        berkeley_writer.rules_fmt(f)?;
        berkeley_writer.lexicon_fmt(f)?;
        berkeley_writer.words_fmt(f)?;

        Ok(())
    }
}
