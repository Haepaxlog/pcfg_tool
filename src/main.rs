use std::{fs::File, io::Write};

use pcfg_tool::{
    berkeley::{BerkeleyFormatWriter, BerkeleyWriter},
    cli::{Cli, CommandFactory, Commands, Parser},
    induce::PCFGGrammar,
    ptb::PTBParser,
    Grammar,
};

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Induce { grammar }) => {
            if let Some(gname) = grammar {
                let mut trees = Vec::new();
                for (i, line) in std::io::stdin().lines().enumerate() {
                    match line {
                        Ok(line) => match PTBParser::parse(&line) {
                            Ok(tree) => trees.push(tree),
                            Err(e) => {
                                eprintln!("Error while parsing tree {} at line {}: {}", line, i, e)
                            }
                        },
                        Err(e) => eprintln!("Error on line {}: {}", i, e),
                    }
                }

                let initial = "ROOT";
                let grammar = Grammar::from_parse_trees(initial.to_string(), trees);

                match grammar {
                    Ok(g) => {
                        let berkeley_writer = BerkeleyWriter::from_grammar(g);

                        let mut rules = File::create(format!("{}.rules", gname))
                            .expect("Error while creating rules file");
                        berkeley_writer
                            .rules_io(&mut rules)
                            .expect("Couldn't write rules file");

                        let mut lexicon = File::create(format!("{}.lexicon", gname))
                            .expect("Error while creating lexicon file");
                        berkeley_writer
                            .lexicon_io(&mut lexicon)
                            .expect("Couldn't write lexicon file");

                        let mut words = File::create(format!("{}.words", gname))
                            .expect("Error while creating words file");
                        berkeley_writer
                            .words_io(&mut words)
                            .expect("Couldn't write words file");
                    }
                    Err(e) => eprintln!("Error while creating PCFG from trees: {}", e),
                }
            } else {
                let mut trees = Vec::new();
                for (i, line) in std::io::stdin().lines().enumerate() {
                    match line {
                        Ok(line) => match PTBParser::parse(&line) {
                            Ok(tree) => trees.push(tree),
                            Err(e) => {
                                eprintln!("Error while parsing tree {} at line {}: {}", line, i, e)
                            }
                        },
                        Err(e) => eprintln!("Error on line {}: {}", i, e),
                    }
                }

                let initial = "ROOT";
                let grammar = Grammar::from_parse_trees(initial.to_string(), trees);

                match grammar {
                    Ok(g) => {
                        let berkeley_writer = BerkeleyWriter::from_grammar(g);

                        let mut stdout = std::io::stdout();
                        berkeley_writer.rules_io(&mut stdout).expect("works");
                        berkeley_writer.lexicon_io(&mut stdout).expect("works");
                        berkeley_writer.words_io(&mut stdout).expect("works");

                        stdout.flush().expect("works");
                    }
                    Err(e) => eprintln!("Error while creating PCFG from trees: {}", e),
                }
            }
        }
        None => {
            Cli::command()
                .print_help()
                .expect("Couldn't print help to stdout");
        }
    }
}
