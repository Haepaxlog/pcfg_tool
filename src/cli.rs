pub use clap::{CommandFactory, Parser};

use clap::Subcommand;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Reads a sequence of contituent trees from the stdin and prints an induced PCFG to the stdout
    Induce {
        /// If this is set, the induced grammar is written into GRAMMAR.rules , GRAMMAR.lexicon, and GRAMMAR.words files instead of the stdout
        grammar: Option<String>,
    },
}
