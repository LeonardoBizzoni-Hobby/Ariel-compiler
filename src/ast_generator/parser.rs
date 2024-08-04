use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use colored::Colorize;

use crate::{
    ast_generator::{
        ast::{enums::Enum, function::Function, datatypes::DataType},
        statement_parser::parse_scope_block,
    },
    tokens::{
        error::{Error, ParseError},
        source::SourceFile,
        token::Token,
        token_type::TokenType,
        tokenizer,
    },
};

use super::{
    ast::{function_arg::Argument, structs::Struct, ASTs},
    parser_head::ParserHead,
    utils,
};

pub fn parse(path: &str, imported_files: Arc<Mutex<HashSet<String>>>) -> ASTs {
    let mut ast = ASTs::new();

    let mut mutex_data = match imported_files.lock() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("[{path}] :: {e}");
            return ast;
        }
    };

    if mutex_data.contains(path) {
        eprintln!(
            "{} `{path}` has already been included. Skipping it.",
            "[Warning]".bold().yellow()
        );
        return ast;
    } else {
        mutex_data.insert(path.to_owned());
        drop(mutex_data);
    }

    let mut source = match SourceFile::new(path) {
        Ok(source) => source,
        Err(e) => match e {
            Error::FileNotFound(source, msg) | Error::MemoryMapFiled(source, msg) => {
                eprintln!("[{source}] :: {msg}");
                return ast;
            }
        },
    };

    let mut handlers: VecDeque<JoinHandle<ASTs>> = VecDeque::new();
    let mut head: ParserHead = ParserHead::new(
        tokenizer::get_token(&mut source),
        Box::new(Token::new()),
        &mut source,
    );

    // Actual parse loop
    parse_global_stmt(&mut head, &mut ast, imported_files, &mut handlers, path);

    // After the parse loop wait for the other threads to finish if there are any
    while let Some(handle) = handlers.pop_front() {
        if handle.is_finished() {
            match handle.join() {
                Ok(other) => ast.merge(other),
                Err(e) => eprintln!("{e:?}"),
            }
        } else {
            handlers.push_back(handle);
        }
    }

    ast
}

fn parse_global_stmt(
    head: &mut ParserHead,
    ast: &mut ASTs,
    imported_files: Arc<Mutex<HashSet<String>>>,
    thread_handles: &mut VecDeque<JoinHandle<ASTs>>,
    curr_file_name: &str,
) {
    loop {
        match head.curr.ttype {
            TokenType::Eof => break,
            TokenType::Import => {
                head.advance();

                match head.require_current_is(TokenType::String) {
                    Ok(_) => {
                        // need to be cloned because of the move in the closure
                        let imported_path = head.curr.lexeme.clone();
                        let imported_files = Arc::clone(&imported_files);

                        thread_handles.push_back(thread::spawn(move || -> ASTs {
                            parse(&imported_path, imported_files)
                        }));
                    }
                    Err(e) => {
                        utils::print_error(curr_file_name, "import", e);
                        head.synchronize();
                    }
                }

                head.advance();
                if let Err(e) = head.require_current_is(TokenType::Semicolon) {
                    utils::print_error(curr_file_name, &head.prev.lexeme, e);
                    head.synchronize();
                    continue;
                }
            }
            TokenType::Fn => {
                match parse_function_definition(head) {
                    Ok(func_ast) => ast.fns.push(func_ast),
                    Err(e) => {
                        utils::print_error(curr_file_name, &head.prev.lexeme, e);
                        head.synchronize();
                    },
                }
            }
            TokenType::Enum => {
                match parse_enum_definition(head) {
                    Ok(enum_ast) => ast.enums.push(enum_ast),
                    Err(e) => {
                        utils::print_error(curr_file_name, &head.prev.lexeme, e);
                        head.synchronize();
                    },
                }
            }
            TokenType::Struct => {
                match parse_struct_definition(head) {
                    Ok(struct_ast) => ast.structs.push(struct_ast),
                    Err(e) => {
                        utils::print_error(curr_file_name, &head.prev.lexeme, e);
                        head.synchronize();
                    },
                }
            }
            _ => {}
        }
    }
}

fn parse_function_definition(head: &mut ParserHead) -> Result<Function, ParseError> {
    let mut function: Function;
    let mut args: Vec<Argument> = vec![];

    // fn -> fn_name
    head.advance();

    match head.curr.ttype {
        TokenType::Main => {
            function = Function::make_main(std::mem::take(&mut head.curr));
        }
        TokenType::Identifier => {
            function = Function::make_func(std::mem::take(&mut head.curr));
        }
        _ => return Err(ParseError::InvalidFnName { name: std::mem::take(&mut head.curr) }),
    }

    // fn_name -> (
    head.advance();
    head.require_current_is(TokenType::LeftParen)?;

    // ( -> arg_name:datatype
    head.advance();

    // Function argument parsing
    while !matches!(head.curr.ttype, TokenType::RightParen) {
        args.push(head.parse_argument()?);

        match head.curr.ttype {
            TokenType::RightParen => break,
            TokenType::Comma => {
                // , -> arg_name:datatype
                head.advance();
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: std::mem::take(&mut head.curr),
                    expected: TokenType::RightParen,
                    msg: Some(String::from(
                        "After a function argument there should have been either a `,` or a `)`.",
                    )),
                });
            }
        }
    }

    // ) -> ->
    // ) -> {
    // ) -> ;
    head.advance();
    function.args(args);

    // Return type parsing
    let body = match head.curr.ttype {
        TokenType::Arrow => {
            // -> -> datatype
            head.advance();
            function.ret_type(head.parse_datatype()?);

            // Function body parsing
            head.require_current_is(TokenType::LeftBrace)?;
            head.advance();

            Some(parse_scope_block(head)?)
        }
        TokenType::LeftBrace => {
            // { -> scope body
            head.advance();
            Some(parse_scope_block(head)?)
        }
        TokenType::Semicolon => None,
        _ => return Err(ParseError::InvalidFnBody { body: std::mem::take(&mut head.curr) }),
    };

    function.body(body);
    Ok(function)
}

fn parse_enum_definition(head: &mut ParserHead) -> Result<Enum, ParseError> {
    // enum -> enum_name
    head.advance();

    head.require_current_is(TokenType::Identifier)?;
    head.advance();

    let enum_name = std::mem::take(&mut head.prev);

    head.require_current_is(TokenType::LeftBrace)?;
    head.advance();

    let mut variants: HashMap<Box<Token>, Option<DataType>> = HashMap::new();
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        head.require_current_is(TokenType::Identifier)?;
        head.advance();

        let variant_name = std::mem::take(&mut head.prev);
        let variant_type = match head.curr.ttype {
            TokenType::LeftParen => {
                head.advance();
                let variant_type = head.parse_datatype()?;

                head.require_current_is(TokenType::RightParen)?;
                head.advance();

                Some(variant_type)
            }
            _ => None,
        };

        variants.insert(Box::from(*variant_name), variant_type);

        match head.curr.ttype {
            TokenType::RightBrace => break,
            TokenType::Comma => {
                head.advance();
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: std::mem::take(&mut head.curr),
                    expected: TokenType::RightParen,
                    msg: Some(String::from(
                        "After an enum variant there should have been either a `,` or a `}`.",
                    )),
                });
            }
        }
    }

    head.require_current_is(TokenType::RightBrace)?;
    head.advance();

    Ok(Enum::new(Box::from(*enum_name), variants))
}

fn parse_struct_definition(head: &mut ParserHead) -> Result<Struct, ParseError> {
    // struct -> struct_name
    head.advance();

    head.require_current_is(TokenType::Identifier)?;
    head.advance();

    let struct_name = std::mem::take(&mut head.prev);

    head.require_current_is(TokenType::LeftBrace)?;
    head.advance();

    let mut fields: Vec<(Box<Token>, DataType)> = vec![];
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        head.require_current_is(TokenType::Identifier)?;
        head.advance();

        let field_name = std::mem::take(&mut head.prev);

        head.require_current_is(TokenType::Colon)?;
        head.advance();

        let field_type: DataType = head.parse_datatype()?;

        fields.push((field_name, field_type));

        match head.curr.ttype {
            TokenType::RightBrace => break,
            TokenType::Comma => {
                head.advance();
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: std::mem::take(&mut head.curr),
                    expected: TokenType::RightParen,
                    msg: Some(String::from(
                        "After a struct field there should have been either a `,` or a `}`.",
                    )),
                });
            }
        }
    }

    head.require_current_is(TokenType::RightBrace)?;
    head.advance();

    Ok(Struct::new(struct_name, fields))
}
