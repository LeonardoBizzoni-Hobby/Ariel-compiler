use clap::{Parser as ClapParser, Subcommand};

use crate::ast_generator::parser;

mod ast_generator;
mod tokens;

#[derive(ClapParser)]
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
    let start_timer = std::time::Instant::now();
    let _ast = parser::parse(source);
    let elapsed = start_timer.elapsed();
    println!(
        "Parsing took: {}ns ({}ms)",
        elapsed.as_nanos(),
        elapsed.as_millis()
    );
    // println!("{:#?}", ast);
}
