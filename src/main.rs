use clap::Parser;
use cli_args::CliArgs;

mod cli_args;

fn main() {
    let args = CliArgs::parse();
    match args.source {
        Some(source) => ariel::compile(&source),
        None => eprintln!("You need to provide the path to the source file to compile!"),
    }
}
