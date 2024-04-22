use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    ast_generator::{ast::function::Function, statement_parser::parse_scope_block},
    tokens::{
        error::{Error, ParseError},
        source::Source,
        token::Token,
        token_type::TokenType,
        tokenizer,
    },
};

use super::{
    ast::{function_arg::Argument, Ast},
    utils,
};

pub fn parse(path: &str, imported_files: Arc<Mutex<HashSet<String>>>) -> Vec<Box<Ast>> {
    let mut ast: Vec<Box<Ast>> = vec![];

    {
        let mut mutex_data = match imported_files.lock() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("[{path}] :: {e}");
                return vec![];
            }
        };

        if mutex_data.contains(path) {
            return vec![];
        } else {
            mutex_data.insert(path.to_owned());
        }
    }

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
    while !matches!(curr.ttype, TokenType::Eof) {
        match curr.ttype {
            TokenType::Import => {
                utils::advance(&mut curr, &mut prev, &mut source);

                if let Err(e) = utils::require_token_type(&mut curr, TokenType::String) {
                    utils::print_error(path, "import", e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                } else {
                    let imported_path = curr.lexeme.clone();
                    let imported_files = Arc::clone(&imported_files);
                    handlers
                        .push_back(thread::spawn(move || parse(&imported_path, imported_files)));
                }

                utils::advance(&mut curr, &mut prev, &mut source);
                if let Err(e) = utils::require_token_type(&mut curr, TokenType::Semicolon) {
                    utils::print_error(path, &prev.lexeme, e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                }
            }
            TokenType::Fn | TokenType::Struct | TokenType::Template | TokenType::Enum => {
                match {
                    match curr.ttype {
                        TokenType::Fn => parse_function_definition,
                        TokenType::Struct => parse_struct_definition,
                        TokenType::Template => parse_template_definition,
                        TokenType::Enum => parse_enum_definition,
                        _ => panic!(),
                    }
                }(&mut curr, &mut prev, &mut source)
                {
                    Ok(function) => {
                        ast.push(function);
                    }
                    Err(e) => {
                        utils::print_error(path, &prev.lexeme, e);
                        global_synchronize(&mut curr, &mut prev, &mut source);
                        continue;
                    }
                }
            }
            TokenType::Eof => {}
            _ => {}
        }

        utils::advance(&mut curr, &mut prev, &mut source);
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

fn parse_function_definition(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Box<Ast>, ParseError> {
    let mut function: Function;
    let mut args: Vec<Argument> = vec![];

    // fn -> fn_name
    utils::advance(curr, prev, source);

    if matches!(curr.ttype, TokenType::Main) {
        function = Function::make_main(curr.clone());
    } else {
        function = Function::make_func(curr.clone());
    }

    // fn_name -> (
    utils::advance(curr, prev, source);
    utils::require_token_type(curr, TokenType::LeftParen)?;

    // ( -> arg_name:datatype
    utils::advance(curr, prev, source);

    // Function argument parsing
    while !matches!(curr.ttype, TokenType::RightParen) {
        args.push(parse_argument(curr, prev, source)?);

        if matches!(curr.ttype, TokenType::Comma) {
            // , -> arg_name:datatype
            utils::advance(curr, prev, source);
        } else if !matches!(curr.ttype, TokenType::RightParen) {
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

    // ) -> ->
    // ) -> {
    utils::advance(curr, prev, source);

    function.args(args);

    // Return type parsing
    if matches!(curr.ttype, TokenType::Arrow) {
        // -> -> datatype
        utils::advance(curr, prev, source);

        function.ret_type(utils::parse_datatype(curr, prev, source)?);
    }

    // Function body parsing
    utils::require_token_type(curr, TokenType::LeftBrace)?;
    utils::advance(curr, prev, source);

    function.body(parse_scope_block(curr, prev, source)?);

    Ok(Box::new(Ast::Fn(function)))
}

fn parse_argument(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Argument, ParseError> {
    utils::require_token_type(curr, TokenType::Identifier)?;
    let field_name = curr.clone();

    // arg_name -> :
    utils::advance(curr, prev, source);
    utils::require_token_type(curr, TokenType::Colon)?;

    // : -> datatype
    utils::advance(curr, prev, source);

    Ok(Argument::new(
        field_name,
        utils::parse_datatype(curr, prev, source)?,
    ))
}

fn parse_struct_definition(
    _curr: &mut Arc<Token>,
    _prev: &mut Arc<Token>,
    _source: &mut Source,
) -> Result<Box<Ast>, ParseError> {
    todo!()
}

fn parse_template_definition(
    _curr: &mut Arc<Token>,
    _prev: &mut Arc<Token>,
    _source: &mut Source,
) -> Result<Box<Ast>, ParseError> {
    todo!()
}

fn parse_enum_definition(
    _curr: &mut Arc<Token>,
    _prev: &mut Arc<Token>,
    _source: &mut Source,
) -> Result<Box<Ast>, ParseError> {
    todo!()
}

fn global_synchronize(curr: &mut Arc<Token>, prev: &mut Arc<Token>, source: &mut Source) {
    loop {
        utils::advance(curr, prev, source);

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
