use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use colored::Colorize;

use crate::{
    ast_generator::{
        ast::{enums::Enum, function::Function, variables::DataType},
        statement_parser::parse_scope_block,
        utils::parse_datatype,
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
    ast::{function_arg::Argument, structs::Struct, Ast},
    parser_head::ParserHead,
    utils,
};

pub fn parse(path: &str, imported_files: Arc<Mutex<HashSet<String>>>) -> Vec<Ast> {
    let mut ast: Vec<Ast> = vec![];

    {
        let mut mutex_data = match imported_files.lock() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("[{path}] :: {e}");
                return vec![];
            }
        };

        if mutex_data.contains(path) {
            eprintln!(
                "{} `{path}` has already been included. Skipping it.",
                "[Warning]".bold().yellow()
            );
            return vec![];
        } else {
            mutex_data.insert(path.to_owned());
        }
    }

    let mut source = match SourceFile::new(path) {
        Ok(source) => source,
        Err(e) => match e {
            Error::FileNotFound(source, msg) | Error::MemoryMapFiled(source, msg) => {
                eprintln!("[{source}] :: {msg}");
                return vec![];
            }
        },
    };

    let mut handlers: VecDeque<JoinHandle<Vec<Ast>>> = VecDeque::new();
    let mut head: ParserHead = ParserHead::new(
        tokenizer::get_token(&mut source),
        Box::new(Token::default()),
        &mut source,
    );

    // Actual parse loop
    parse_global_stmt(&mut head, &mut ast, imported_files, &mut handlers, path);

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

fn parse_global_stmt(
    head: &mut ParserHead,
    ast: &mut Vec<Ast>,
    imported_files: Arc<Mutex<HashSet<String>>>,
    thread_handles: &mut VecDeque<JoinHandle<Vec<Ast>>>,
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

                        thread_handles.push_back(thread::spawn(move || -> Vec<Ast> {
                            parse(&imported_path, imported_files)
                        }));
                    }
                    Err(e) => {
                        utils::print_error(curr_file_name, "import", e);
                        synchronize(head);
                    }
                }

                head.advance();
                if let Err(e) = head.require_current_is(TokenType::Semicolon) {
                    utils::print_error(curr_file_name, &head.prev.lexeme, e);
                    synchronize(head);
                    continue;
                }
            }
            TokenType::Fn | TokenType::Struct | TokenType::Enum => {
                match {
                    match head.curr.ttype {
                        TokenType::Fn => parse_function_definition,
                        TokenType::Struct => parse_struct_definition,
                        TokenType::Enum => parse_enum_definition,
                        _ => panic!(),
                    }
                }(head)
                {
                    Ok(global_stmt) => {
                        ast.push(global_stmt);
                    }
                    Err(e) => {
                        utils::print_error(curr_file_name, &head.prev.lexeme, e);
                        synchronize(head);
                    }
                }
            }
            _ => {}
        }
    }
}

fn parse_function_definition(head: &mut ParserHead) -> Result<Ast, ParseError> {
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
        _ => return Err(ParseError::InvalidFnName { name: std::mem::take(&mut  head.curr) }),
    }

    // fn_name -> (
    head.advance();
    head.require_current_is(TokenType::LeftParen)?;

    // ( -> arg_name:datatype
    head.advance();

    // Function argument parsing
    while !matches!(head.curr.ttype, TokenType::RightParen) {
        args.push(parse_argument(head)?);

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
            function.ret_type(utils::parse_datatype(head)?);

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
    Ok(Ast::Fn(function))
}

fn parse_enum_definition(head: &mut ParserHead) -> Result<Ast, ParseError> {
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
                let variant_type = parse_datatype(head)?;

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

    Ok(Ast::Enum(Enum::new(Box::from(*enum_name), variants)))
}

fn parse_struct_definition(head: &mut ParserHead) -> Result<Ast, ParseError> {
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

        let field_type: DataType = parse_datatype(head)?;

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

    Ok(Ast::Struct(Struct::new(struct_name, fields)))
}

fn parse_argument(head: &mut ParserHead) -> Result<Argument, ParseError> {
    head.require_current_is(TokenType::Identifier)?;
    let field_name = std::mem::take(&mut head.curr);

    // arg_name -> :
    head.advance();
    head.require_current_is(TokenType::Colon)?;

    // : -> datatype
    head.advance();

    Ok(Argument(field_name, utils::parse_datatype(head)?))
}

fn synchronize(head: &mut ParserHead) {
    loop {
        head.advance();

        match head.curr.ttype {
            TokenType::Import | TokenType::Struct | TokenType::Fn | TokenType::Eof => break,
            _ => {}
        }
    }
}
