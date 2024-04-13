use ariel::{CliArgs, Commands};
use clap::Parser;

fn main() {
    let args = CliArgs::parse();

    match &args.commands {
        Some(Commands::Repl) => {
            ariel::repl();
        },
        None => match args.source {
            Some(source) => ariel::compile(&source),
            None => eprintln!("You need to provide the path to the source file to compile!"),
        }
    }
}
