use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Reads a sequence of contituent trees from the stdin and prints an induced PCFG to the stdout
    Induce {
        /// If this is set, the induced grammar is written into GRAMMAR.rules , GRAMMAR.lexicon, and GRAMMAR.words files instead of the stdout
        #[arg(short, long)]
        grammar: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Induce { grammar }) => {
            todo!()
        }
        None => {
            Cli::command()
                .print_help()
                .expect("Couldn't print help to stdout");
        }
    }
}
