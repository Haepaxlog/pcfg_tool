use crate::{induce::PCFGGrammar, Grammar};

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
            writeln!(f, "{} {}", rule, probability)?;
        }

        Ok(())
    }

    fn lexicon_fmt<F>(&self, f: &mut F) -> std::fmt::Result
    where
        F: std::fmt::Write,
    {
        for (rule, probability) in self.grammar.lexical_rules() {
            writeln!(f, "{} {}", rule, probability)?;
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
        for (rule, probability) in self.grammar.nonlexical_rules() {
            writeln!(f, "{} {}", rule, probability)?;
        }

        Ok(())
    }

    fn lexicon_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write,
    {
        for (rule, probability) in self.grammar.lexical_rules() {
            writeln!(f, "{} {}", rule, probability)?;
        }

        Ok(())
    }

    fn words_io<F>(&self, f: &mut F) -> std::io::Result<()>
    where
        F: std::io::Write,
    {
        for terminal in self.grammar.terminals() {
            writeln!(f, "{}", terminal)?;
        }

        Ok(())
    }
}
