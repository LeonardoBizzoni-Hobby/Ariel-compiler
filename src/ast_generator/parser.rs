use crate::ast_generator::ast::{
    function::LbFunction, function_field::FunctionField, ScopeBoundStatement,
};
use std::{
    collections::VecDeque,
    sync::Arc,
    thread::{self, JoinHandle},
};

use colored::Colorize;

use crate::tokens::{
    error::{Error, ParseError},
    source::Source,
    token::Token,
    token_type::TokenType,
    tokenizer,
};

use super::ast::Ast;

pub fn parse(path: &str) -> Vec<Box<Ast>> {
    let mut ast: Vec<Box<Ast>> = vec![];

    // TODO: use a Contex to check if the current file was already included
    // and share it with every other parser-thread
    let mut source = match Source::new(path) {
        Ok(source) => source,
        Err(e) => match e {
            Error::FileNotFound(source, msg) | Error::MemoryMapFiled(source, msg) => {
                eprintln!("[{source}] :: {msg}");
                return vec![];
            }
        },
    };

    let mut curr: Arc<Token> = tokenizer::get_token(&mut source);
    let mut prev: Arc<Token> = Arc::new(Token::empty());
    let mut handlers: VecDeque<JoinHandle<Vec<Box<Ast>>>> = VecDeque::new();

    // Actual parse loop
    while curr.ttype != TokenType::Eof {
        match curr.ttype {
            TokenType::Import => {
                advance(&mut curr, &mut prev, &mut source);

                if let Err(e) = consume(&mut curr, TokenType::String) {
                    print_error(path, "import", e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                } else {
                    let imported_path = curr.lexeme.clone();
                    handlers.push_back(thread::spawn(move || parse(&imported_path)));
                }

                advance(&mut curr, &mut prev, &mut source);
                if let Err(e) = consume(&mut curr, TokenType::Semicolon) {
                    print_error(path, &prev.lexeme, e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                }
            }
            TokenType::Fn => {
                match define_function(&mut curr, &mut prev, &mut source) {
                    Ok(function) => {
                        ast.push(function);
                    }
                    Err(e) => {
                        print_error(path, &prev.lexeme, e);
                        global_synchronize(&mut curr, &mut prev, &mut source);
                        continue;
                    }
                }
            },
            TokenType::Eof => {}
            _ => {}
        }

        advance(&mut curr, &mut prev, &mut source);
    }

    // After the parse loop wait for the other threads to finish if there are any
    while let Some(handle) = handlers.pop_front() {
        if handle.is_finished() {
            ast.append(&mut {
                match handle.join() {
                    Ok(ast_data) => ast_data,
                    Err(e) => {
                        eprintln!("{e:?}");
                        vec![]
                    }
                }
            });
        } else {
            handlers.push_back(handle);
        }
    }

    ast
}

fn define_function(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    lexer: &mut Source,
) -> Result<Box<Ast>, ParseError> {
    let mut function: LbFunction;
    let mut args: Vec<FunctionField> = vec![];

    advance(curr, prev, lexer);

    if curr.ttype == TokenType::Main {
        function = LbFunction::make_main(curr.clone());
    } else {
        function = LbFunction::make_func(curr.clone());
    }

    advance(curr, prev, lexer);
    consume(curr, TokenType::LeftParen)?;

    // Function argument parsing
    advance(curr, prev, lexer);
    while curr.ttype != TokenType::RightParen {
        consume(curr, TokenType::Identifier)?;

        args.push(parse_argument(curr, prev, lexer)?);
        advance(curr, prev, lexer);

        if curr.ttype == TokenType::Comma {
            advance(curr, prev, lexer);
        } else if curr.ttype != TokenType::RightParen {
            return Err(ParseError::UnexpectedToken {
                line: curr.line,
                col: curr.column,
                found: curr.ttype.clone(),
                expected: TokenType::RightParen,
                msg: Some(String::from(
                    "After a function argument there should have been either a `,` or a `)`.",
                )),
            });
        }
    }
    function.args(args);

    // Return type parsing
    advance(curr, prev, lexer);
    if curr.ttype == TokenType::Arrow {
        advance(curr, prev, lexer);
        if !is_datatype(&curr.ttype) {
            return Err(ParseError::InvalidDataType {
                line: curr.line,
                col: curr.column,
                data_type: curr.ttype.clone(),
            });
        }

        function.ret_type(curr.clone());
        advance(curr, prev, lexer);
    }

    // Function body parsing
    consume(curr, TokenType::LeftBrace)?;
    function.body(parse_scope_block(curr, prev, lexer)?);

    Ok(Box::new(Ast::Fn(function)))
}

fn parse_scope_block(
    _curr: &mut Arc<Token>,
    _prev: &mut Arc<Token>,
    _source: &mut Source,
) -> Result<Vec<ScopeBoundStatement>, ParseError> {
    Ok(vec![])
}

fn parse_argument(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<FunctionField, ParseError> {
    let field_name = curr.clone();

    advance(curr, prev, source);
    consume(curr, TokenType::Colon)?;
    advance(curr, prev, source);

    if !is_datatype(&curr.ttype) {
        return Err(ParseError::InvalidDataType {
            line: curr.line,
            col: curr.column,
            data_type: curr.ttype.clone(),
        });
    }

    Ok(FunctionField::new(field_name, curr.clone()))
}

fn is_datatype(ttype: &TokenType) -> bool {
    matches!(
        ttype,
        TokenType::StringType
            | TokenType::U8
            | TokenType::U16
            | TokenType::U32
            | TokenType::U64
            | TokenType::I8
            | TokenType::I16
            | TokenType::I32
            | TokenType::I64
            | TokenType::F32
            | TokenType::F64
            | TokenType::Bool
    )
}

fn global_synchronize(curr: &mut Arc<Token>, prev: &mut Arc<Token>, source: &mut Source) {
    loop {
        advance(curr, prev, source);

        match curr.ttype {
            TokenType::Import
            | TokenType::Struct
            | TokenType::Template
            | TokenType::Fn
            | TokenType::Eof => break,
            _ => {}
        }
    }
}

fn consume(curr: &Token, expected: TokenType) -> Result<(), ParseError> {
    if curr.ttype != expected {
        Err(ParseError::UnexpectedToken {
            line: curr.line,
            col: curr.column,
            found: curr.ttype.clone(),
            expected,
            msg: None,
        })
    } else {
        Ok(())
    }
}

fn advance(curr: &mut Arc<Token>, prev: &mut Arc<Token>, source: &mut Source) {
    *prev = Arc::clone(curr);
    *curr = tokenizer::get_token(source);
}

fn print_error(source: &str, after: &str, e: ParseError) {
    match e {
        ParseError::UnexpectedToken {
            line,
            col,
            found,
            expected,
            msg,
        } => {
            if let Some(msg) = msg {
                eprintln!("[{}] {msg}", format!("{source} {line}:{col}").red().bold());
            } else {
                eprintln!("[{}] :: there should have been a {} after the token `{after}`, but instead there was a {}.",
                format!("{source} {line}:{col}").red().bold(),
                format!("{expected}").blue().bold(),
                format!("{found}").red().italic());
            }
        }
        ParseError::InvalidDataType {
            line,
            col,
            data_type,
        } => {
            eprintln!(
                "[{}] :: {} is not a valid data type.",
                format!("{source} {line}:{col}").red().bold(),
                format!("{data_type}").red().italic()
            );
        }
    }
}
