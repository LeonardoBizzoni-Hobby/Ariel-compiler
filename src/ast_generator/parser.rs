use std::{
    collections::VecDeque,
    thread::{self, JoinHandle},
};

use crate::tokens::{
    error::{Error, ParseError},
    token::Token,
    token_type::TokenType,
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub enum Ast {
    Integer(i32),
}

pub fn parse(source: &str) -> Result<Vec<Box<Ast>>, Error> {
    let mut lexer = match Tokenizer::new(source) {
        Ok(lexer) => lexer,
        Err(e) => match e {
            Error::FileNotFound(ref path, ref os_error)
            | Error::MemoryMapFiled(ref path, ref os_error) => {
                eprintln!("[{path}] :: {os_error}");
                return Err(e);
            }
            _ => panic!("How?"),
        },
    };

    let mut ast: Vec<Box<Ast>> = vec![];
    let mut handlers: VecDeque<JoinHandle<Result<Vec<Box<Ast>>, Error>>> = VecDeque::new();
    let mut curr: Box<Token> = lexer.get_token();
    let mut prev: Box<Token> = Box::new(Token::empty());

    loop {
        match curr.ttype {
            TokenType::Eof => {
                while let Some(handle) = handlers.pop_front() {
                    if handle.is_finished() {
                        ast.append(&mut handle.join().unwrap()?);
                    } else {
                        handlers.push_back(handle);
                    }
                }

                break;
            }
            TokenType::Import => {
                advance(&mut curr, &mut prev, &mut lexer);

                if curr.ttype != TokenType::String {
                    return Err(Error::Parser(ParseError::InvalidImport));
                } else {
                    let source = curr.lexeme.clone();
                    handlers.push_back(thread::spawn(move || parse(&source)));
                }
            }
            TokenType::Integer => ast.push(Box::new(Ast::Integer(curr.lexeme.parse().unwrap()))),
            _ => {}
        }

        advance(&mut curr, &mut prev, &mut lexer);
    }

    Ok(ast)
}

fn advance(curr: &mut Box<Token>, prev: &mut Box<Token>, lexer: &mut Tokenizer) {
    *prev = std::mem::replace(curr, lexer.get_token());
}
