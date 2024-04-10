use ast_generator::parser::parse;
use clap::{Parser, Subcommand};

mod tokens;
mod ast_generator;

#[derive(Parser)]
pub struct Args {
    pub source: Option<String>,

    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Repl,
}

pub fn repl() {
    todo!("REPL.")
}

pub fn compile(source: &str) {
    println!("{:#?}", parse(source));
}
