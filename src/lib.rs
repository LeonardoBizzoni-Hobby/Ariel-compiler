use std::{
    collections::HashSet,
    io::{self, Write},
    sync::{Arc, Mutex},
};

use clap::{Parser as ClapParser, Subcommand};
use tempfile::NamedTempFile;

use crate::ast_generator::parser;

mod ast_generator;
mod tokens;

#[derive(ClapParser)]
pub struct CliArgs {
    pub source: Option<String>,

    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Repl,
}

pub fn repl() {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    loop {
        let mut command = String::new();
        print!("λ> ");
        io::stdout().flush().unwrap();

        if let Err(e) = io::stdin().read_line(&mut command) {
            eprintln!("{e}");
            break;
        };

        if command.trim().eq("quit") || command.trim().eq("exit") {
            return;
        } else {
            let mut temp_file = match NamedTempFile::new() {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("{e}");
                    return;
                }
            };

            if let Err(e) = write!(temp_file, "{command}") {
                eprintln!("{e}");
                return;
            };

            let ast = parser::parse(
                temp_file.path().to_str().unwrap(),
                Arc::clone(&imported_files),
            );

            println!("{ast:?}");
        }
    }
}

pub fn compile(source: &str) {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    let start_timer = std::time::Instant::now();
    let ast = parser::parse(source, Arc::clone(&imported_files));
    let elapsed = start_timer.elapsed();

    println!(
        "Parsing took: {}ns ({}ms)",
        elapsed.as_nanos(),
        elapsed.as_millis()
    );
    println!("{:#?}", ast);
}
