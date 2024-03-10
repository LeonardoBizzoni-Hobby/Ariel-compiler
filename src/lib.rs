use clap::{Parser, Subcommand};
use tokens::{error::Error, tokenizer::Tokenizer};

mod tokens;

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

pub fn repl(_source: &Option<String>) {
    todo!("REPL.")
}

pub fn compile(source: &str) {
    let prova: Vec<std::rc::Rc<tokens::token::Token>>;
    {
        let mut lexer = match Tokenizer::new(source) {
            Ok(lexer) => lexer,
            Err(e) => match e {
                Error::FileNotFound(path, os_error) | Error::MemoryMapFiled(path, os_error) => {
                    eprintln!("[{path}] :: {os_error}");
                    return;
                }
            },
        };

        prova = lexer.get_all_tokens();
    }

    println!("{:#?}", prova);
}
