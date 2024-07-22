use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use colored::Colorize;

use crate::{
    ast_generator::{
        ast::{enums::Enum, function::Function, variables::DataType},
        statement_parser::parse_scope_block, utils::parse_datatype,
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

    let mut curr: Arc<Token> = tokenizer::get_token(&mut source);
    let mut prev: Arc<Token> = Arc::new(Token::empty());
    let mut handlers: VecDeque<JoinHandle<Vec<Ast>>> = VecDeque::new();
    let mut head: ParserHead = ParserHead::new(&mut curr, &mut prev, &mut source);

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
                utils::advance(head);

                match utils::require_token_type(&head.curr, TokenType::String) {
                    Ok(_) => {
                        // need to be cloned because of the move in the closure
                        let imported_path = head.curr.lexeme.clone();
                        let imported_files = Arc::clone(&imported_files);

                        thread_handles.push_back(thread::spawn(move || {
                            parse(&imported_path, imported_files)
                        }));
                    }
                    Err(e) => {
                        utils::print_error(curr_file_name, "import", e);
                        synchronize(head);
                    }
                }

                utils::advance(head);
                if let Err(e) = utils::require_token_type(&mut head.curr, TokenType::Semicolon) {
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
    utils::advance(head);

    match head.curr.ttype {
        TokenType::Main => {
            function = Function::make_main(Arc::clone(head.curr));
        }
        TokenType::Identifier => {
            function = Function::make_func(Arc::clone(head.curr));
        }
        _ => {
            return Err(ParseError::InvalidFnName {
                name: Arc::clone(head.curr),
            })
        }
    }

    // fn_name -> (
    utils::advance(head);
    utils::require_token_type(head.curr, TokenType::LeftParen)?;

    // ( -> arg_name:datatype
    utils::advance(head);

    // Function argument parsing
    while !matches!(head.curr.ttype, TokenType::RightParen) {
        args.push(parse_argument(head)?);

        match head.curr.ttype {
            TokenType::RightParen => break,
            TokenType::Comma => {
                // , -> arg_name:datatype
                utils::advance(head);
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    line: head.curr.line,
                    col: head.curr.column,
                    found: head.curr.ttype.clone(),
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
    utils::advance(head);
    function.args(args);

    // Return type parsing
    let body = match head.curr.ttype {
        TokenType::Arrow => {
            // -> -> datatype
            utils::advance(head);
            function.ret_type(utils::parse_datatype(head)?);

            // Function body parsing
            utils::require_token_type(head.curr, TokenType::LeftBrace)?;
            utils::advance(head);

            Some(parse_scope_block(head)?)
        }
        TokenType::LeftBrace => {
            // { -> scope body
            utils::advance(head);
            Some(parse_scope_block(head)?)
        }
        TokenType::Semicolon => None,
        _ => {
            return Err(ParseError::InvalidFnBody {
                body: Arc::clone(head.curr),
            })
        }
    };

    function.body(body);
    Ok(Ast::Fn(function))
}

fn parse_enum_definition(head: &mut ParserHead) -> Result<Ast, ParseError> {
    // enum -> enum_name
    utils::advance(head);

    utils::require_token_type(&head.curr, TokenType::Identifier)?;
    utils::advance(head);

    let enum_name = Arc::clone(head.prev);

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    let mut variants: HashMap<Arc<Token>, Option<DataType>> = HashMap::new();
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        utils::require_token_type(&head.curr, TokenType::Identifier)?;
        utils::advance(head);

        let variant_name = Arc::clone(head.prev);
        let variant_type = match head.curr.ttype {
            TokenType::LeftParen => {
                utils::advance(head);
                let variant_type = parse_datatype(head)?;

                utils::require_token_type(&head.curr, TokenType::RightParen)?;
                utils::advance(head);

                Some(variant_type)
            },
            _ => None,
        };

        variants.insert(variant_name, variant_type);

        match head.curr.ttype {
            TokenType::RightBrace => break,
            TokenType::Comma => {
                utils::advance(head);
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    line: head.curr.line,
                    col: head.curr.column,
                    found: head.curr.ttype.clone(),
                    expected: TokenType::RightParen,
                    msg: Some(String::from(
                        "After an enum variant there should have been either a `,` or a `}`.",
                    )),
                });
            }
        }
    }

    utils::require_token_type(&head.curr, TokenType::RightBrace)?;
    utils::advance(head);

    Ok(Ast::Enum(Enum::new(enum_name, variants)))
}

fn parse_struct_definition(head: &mut ParserHead) -> Result<Ast, ParseError> {
    // struct -> struct_name
    utils::advance(head);

    utils::require_token_type(&head.curr, TokenType::Identifier)?;
    utils::advance(head);

    let struct_name = Arc::clone(head.prev);

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    let mut fields: Vec<(Arc<Token>, DataType)> = vec![];
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        utils::require_token_type(&head.curr, TokenType::Identifier)?;
        utils::advance(head);

        let field_name = Arc::clone(head.prev);

        utils::require_token_type(&head.curr, TokenType::Colon)?;
        utils::advance(head);

        let field_type: DataType = parse_datatype(head)?;

        fields.push((field_name, field_type));

        match head.curr.ttype {
            TokenType::RightBrace => break,
            TokenType::Comma => {
                utils::advance(head);
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    line: head.curr.line,
                    col: head.curr.column,
                    found: head.curr.ttype.clone(),
                    expected: TokenType::RightParen,
                    msg: Some(String::from(
                        "After a struct field there should have been either a `,` or a `}`.",
                    )),
                });
            }
        }
    }

    utils::require_token_type(&head.curr, TokenType::RightBrace)?;
    utils::advance(head);

    Ok(Ast::Struct(Struct::new(struct_name, fields)))
}

fn parse_argument(head: &mut ParserHead) -> Result<Argument, ParseError> {
    utils::require_token_type(head.curr, TokenType::Identifier)?;
    let field_name = Arc::clone(head.curr);

    // arg_name -> :
    utils::advance(head);
    utils::require_token_type(head.curr, TokenType::Colon)?;

    // : -> datatype
    utils::advance(head);

    Ok(Argument(field_name, utils::parse_datatype(head)?))
}

fn synchronize(head: &mut ParserHead) {
    loop {
        utils::advance(head);

        match head.curr.ttype {
            TokenType::Import | TokenType::Struct | TokenType::Fn | TokenType::Eof => break,
            _ => {}
        }
    }
}
