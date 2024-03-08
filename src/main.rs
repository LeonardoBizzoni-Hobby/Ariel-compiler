use clap::Parser;
use lang::{Args, repl, compile};

fn main() {
    let args = Args::parse();

    match &args.commands {
        Some(lang::Commands::Repl) => {
            repl(&args.source);
        },
        None => match args.source {
            Some(source) => compile(&source),
            None => eprintln!("You need to provide the path to the source file to compile!"),
        }
    }
}
