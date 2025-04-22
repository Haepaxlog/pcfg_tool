use pcfg_tool::cli::{Cli, CommandFactory, Commands, Parser};

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
