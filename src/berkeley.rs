use std::io::{BufWriter, Write};

use crate::{induce::PCFGGrammar, Grammar, Probability, Rule};

pub trait BerkeleyFormatWriter {
    fn rules_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write;
    fn lexicon_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write;
    fn words_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write;
    fn rules_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write;
    fn lexicon_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write;
    fn words_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write;
}

pub struct BerkeleyWriter {
    pub grammar: Grammar,
}

impl BerkeleyWriter {
    pub fn from_grammar(grammar: Grammar) -> Self {
        Self { grammar }
    }
}

impl BerkeleyFormatWriter for BerkeleyWriter {
    fn rules_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write,
    {
        for (rule, probability) in self.grammar.nonlexical_rules() {
            rule.print_fmt(f, probability)?;
        }

        Ok(())
    }

    fn lexicon_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write,
    {
        for (rule, probability) in self.grammar.lexical_rules() {
            rule.print_fmt(f, probability)?;
        }

        Ok(())
    }

    fn words_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write,
    {
        for terminal in self.grammar.terminals() {
            writeln!(f, "{}", terminal)?;
        }

        Ok(())
    }

    fn rules_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write,
    {
        let mut w = BufWriter::new(f);
        for (rule, probability) in self.grammar.nonlexical_rules() {
            rule.print_io(&mut w, probability)?;
        }

        w.flush()?;

        Ok(())
    }

    fn lexicon_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write,
    {
        let mut w = BufWriter::new(f);
        for (rule, probability) in self.grammar.lexical_rules() {
            rule.print_io(&mut w, probability)?;
        }

        w.flush()?;

        Ok(())
    }

    fn words_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write,
    {
        let mut w = BufWriter::new(f);
        for terminal in self.grammar.terminals() {
            writeln!(w, "{}", terminal)?;
        }

        w.flush()?;

        Ok(())
    }
}

trait BerkeleyRuleIo {
    fn print_io<F>(&self, w: &mut F, probability: Probability) -> std::io::Result<()>
    where
        F: std::io::Write;
}

impl BerkeleyRuleIo for Rule {
    fn print_io<F>(&self, w: &mut F, probabilty: Probability) -> std::io::Result<()>
    where
        F: std::io::Write,
    {
        match &self.body {
            crate::Body::Lexical(terminal) => {
                writeln!(w, "{} {} {}", self.head, terminal, probabilty)?;
            }
            crate::Body::NonLexical(nonterminals) => {
                write!(w, "{} -> ", self.head)?;
                for (index, nonterminal) in nonterminals.iter().enumerate() {
                    if index != nonterminals.len() - 1 {
                        write!(w, "{} ", nonterminal)?;
                    } else {
                        write!(w, "{}", nonterminal)?;
                    }
                }

                writeln!(w, " {}", probabilty)?;
            }
        }

        Ok(())
    }
}

trait BerkeleyRuleFmt {
    fn print_fmt<F>(&self, w: &mut F, probability: Probability) -> std::fmt::Result
    where
        F: std::fmt::Write;
}

impl BerkeleyRuleFmt for Rule {
    fn print_fmt<F>(&self, w: &mut F, probability: Probability) -> std::fmt::Result
    where
        F: std::fmt::Write,
    {
        match &self.body {
            crate::Body::Lexical(terminal) => {
                write!(w, "{} {} {}", self.head, terminal, probability)?;
            }
            crate::Body::NonLexical(nonterminals) => {
                write!(w, "{} -> ", self.head)?;
                for (index, nonterminal) in nonterminals.iter().enumerate() {
                    if index != nonterminals.len() - 1 {
                        write!(w, "{} ", nonterminal)?;
                    } else {
                        write!(w, "{}", nonterminal)?;
                    }
                }

                writeln!(w, " {}", probability)?;
            }
        }

        Ok(())
    }
}
