use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use colored::Colorize;

use crate::{
    ast_generator::ast::{function::Function, DataType, ScopeBoundStatement},
    tokens::{
        error::{Error, ParseError},
        source::Source,
        token::Token,
        token_type::TokenType,
        tokenizer,
    },
};

use super::ast::{function_field::Argument, Ast, Expression, Variable};

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
                advance(&mut curr, &mut prev, &mut source);

                if let Err(e) = require_token_type(&mut curr, TokenType::String) {
                    print_error(path, "import", e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                } else {
                    let imported_path = curr.lexeme.clone();
                    let imported_files = Arc::clone(&imported_files);
                    handlers
                        .push_back(thread::spawn(move || parse(&imported_path, imported_files)));
                }

                advance(&mut curr, &mut prev, &mut source);
                if let Err(e) = require_token_type(&mut curr, TokenType::Semicolon) {
                    print_error(path, &prev.lexeme, e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                }
            }
            TokenType::Fn => match parse_function_definition(&mut curr, &mut prev, &mut source) {
                Ok(function) => {
                    ast.push(function);
                }
                Err(e) => {
                    print_error(path, &prev.lexeme, e);
                    global_synchronize(&mut curr, &mut prev, &mut source);
                    continue;
                }
            },
            TokenType::Struct => {}
            TokenType::Template => {}
            TokenType::Enum => {}
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

fn parse_function_definition(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Box<Ast>, ParseError> {
    let mut function: Function;
    let mut args: Vec<Argument> = vec![];

    // fn -> fn_name
    advance(curr, prev, source);

    if matches!(curr.ttype, TokenType::Main) {
        function = Function::make_main(curr.clone());
    } else {
        function = Function::make_func(curr.clone());
    }

    // fn_name -> (
    advance(curr, prev, source);
    require_token_type(curr, TokenType::LeftParen)?;

    // ( -> arg_name:datatype
    advance(curr, prev, source);

    // Function argument parsing
    while !matches!(curr.ttype, TokenType::RightParen) {
        args.push(parse_argument(curr, prev, source)?);

        if matches!(curr.ttype, TokenType::Comma) {
            // , -> arg_name:datatype
            advance(curr, prev, source);
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
    advance(curr, prev, source);

    function.args(args);

    // Return type parsing
    if matches!(curr.ttype, TokenType::Arrow) {
        // -> -> datatype
        advance(curr, prev, source);

        function.ret_type(parse_datatype(curr, prev, source)?);
    }

    // Function body parsing
    require_token_type(curr, TokenType::LeftBrace)?;
    advance(curr, prev, source);

    function.body(parse_scope_block(curr, prev, source)?);

    Ok(Box::new(Ast::Fn(function)))
}

fn parse_argument(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Argument, ParseError> {
    require_token_type(curr, TokenType::Identifier)?;
    let field_name = curr.clone();

    // arg_name -> :
    advance(curr, prev, source);
    require_token_type(curr, TokenType::Colon)?;

    // : -> datatype
    advance(curr, prev, source);

    Ok(Argument::new(
        field_name,
        parse_datatype(curr, prev, source)?,
    ))
}

fn parse_scope_block(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Vec<ScopeBoundStatement>, ParseError> {
    let mut body: Vec<ScopeBoundStatement> = vec![];

    while !matches!(curr.ttype, TokenType::RightBrace | TokenType::Eof) {
        body.push(match parse_scopebound_statement(curr, prev, source) {
            Ok(stmt) => stmt,
            Err(e) => {
                print_error(&curr.found_in, &prev.lexeme, e);

                while !matches!(curr.ttype, TokenType::Semicolon | TokenType::Eof) {
                    advance(curr, prev, source);
                }

                // Consumes the `;`
                advance(curr, prev, source);
                continue;
            }
        })
    }

    require_token_type(&curr, TokenType::RightBrace)?;
    advance(curr, prev, source);

    Ok(body)
}

fn parse_scopebound_statement(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    match curr.ttype {
        TokenType::If => todo!(),
        TokenType::Match => todo!(),
        TokenType::Loop => todo!(),
        TokenType::While => todo!(),
        TokenType::For => todo!(),
        TokenType::LeftBrace => {
            advance(curr, prev, source);
            parse_scopebound_statement(curr, prev, source)
        }
        TokenType::Return => {
            advance(curr, prev, source);

            let expr: Expression = parse_expression(curr, prev, source)?;

            require_token_type(curr, TokenType::Semicolon)?;
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::Return(expr))
        }
        TokenType::Let => {
            advance(curr, prev, source);

            let variable: Box<Variable> = parse_variable_declaration(curr, prev, source)?;

            require_token_type(curr, TokenType::Semicolon)?;
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::VariableDeclaration(variable))
        }
        _ => {
            let expr: Expression = parse_expression(curr, prev, source)?;

            require_token_type(curr, TokenType::Semicolon)?;
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::Expression(expr))
        }
    }
}

fn parse_variable_declaration(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Box<Variable>, ParseError> {
    let variable_name = Arc::clone(curr);
    advance(curr, prev, source);

    match curr.ttype {
        TokenType::Colon => {
            advance(curr, prev, source);
            let datatype = parse_datatype(curr, prev, source)?;

            match require_token_type(curr, TokenType::Equal) {
                Ok(_) => {
                    advance(curr, prev, source);

                    Ok(Box::new(Variable::new(
                        variable_name,
                        Some(datatype),
                        parse_expression(curr, prev, source)?,
                    )))
                }
                Err(_) => Err(ParseError::InvalidVariableDeclaration {
                    line: curr.line,
                    column: curr.column,
                }),
            }
        }
        TokenType::DynamicDefinition => {
            advance(curr, prev, source);
            Ok(Box::new(Variable::new(
                variable_name,
                None,
                parse_expression(curr, prev, source)?,
            )))
        }
        _ => {
            return Err(ParseError::InvalidVariableDeclaration {
                line: curr.line,
                column: curr.column,
            })
        }
    }
}

fn parse_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    while !matches!(curr.ttype, TokenType::Semicolon | TokenType::Eof) {
        advance(curr, prev, source);
    }

    Ok(Expression::Tmp)
}

fn parse_datatype(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<DataType, ParseError> {
    match curr.ttype {
        TokenType::U8 => Ok(handle_pointer_datatype(DataType::U8, curr, prev, source)),
        TokenType::U16 => Ok(handle_pointer_datatype(DataType::U16, curr, prev, source)),
        TokenType::U32 => Ok(handle_pointer_datatype(DataType::U32, curr, prev, source)),
        TokenType::U64 => Ok(handle_pointer_datatype(DataType::U64, curr, prev, source)),
        TokenType::I8 => Ok(handle_pointer_datatype(DataType::I8, curr, prev, source)),
        TokenType::I16 => Ok(handle_pointer_datatype(DataType::I16, curr, prev, source)),
        TokenType::I32 => Ok(handle_pointer_datatype(DataType::I32, curr, prev, source)),
        TokenType::I64 => Ok(handle_pointer_datatype(DataType::I64, curr, prev, source)),
        TokenType::F32 => Ok(handle_pointer_datatype(DataType::F32, curr, prev, source)),
        TokenType::F64 => Ok(handle_pointer_datatype(DataType::F64, curr, prev, source)),
        TokenType::Bool => Ok(handle_pointer_datatype(DataType::Bool, curr, prev, source)),
        TokenType::StringType => Ok(handle_pointer_datatype(
            DataType::String,
            curr,
            prev,
            source,
        )),
        TokenType::Void => {
            let datatype = handle_pointer_datatype(DataType::Void, curr, prev, source);
            if matches!(datatype, DataType::Pointer(_)) {
                Ok(datatype)
            } else {
                Err(ParseError::InvalidDataType {
                    line: curr.line,
                    col: curr.column,
                    found: curr.ttype.clone(),
                    msg: Some(
                        "`void` by itself isn't a valid datatype, it should have been a void pointer `void*`."
                            .to_owned(),
                    ),
                })
            }
        }
        TokenType::LeftSquare => {
            advance(curr, prev, source);
            let array_of: DataType = parse_datatype(curr, prev, source)?;

            require_token_type(curr, TokenType::RightSquare)?;
            Ok(handle_pointer_datatype(
                DataType::Array(Box::new(array_of)),
                curr,
                prev,
                source,
            ))
        }
        _ => Err(ParseError::InvalidDataType {
            line: curr.line,
            col: curr.column,
            found: curr.ttype.clone(),
            msg: None,
        }),
    }
}

fn handle_pointer_datatype(
    datatype: DataType,
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> DataType {
    let mut res = datatype;

    // datatype -> *
    // datatype -> ,
    // datatype -> )
    // datatype -> {
    advance(curr, prev, source);

    while matches!(curr.ttype, TokenType::Star) {
        advance(curr, prev, source);
        res = DataType::Pointer(Box::new(res));
    }

    res
}

fn require_token_type(curr: &Token, expected: TokenType) -> Result<(), ParseError> {
    if curr.ttype == expected {
        Ok(())
    } else {
        Err(ParseError::UnexpectedToken {
            line: curr.line,
            col: curr.column,
            found: curr.ttype.clone(),
            expected,
            msg: None,
        })
    }
}

fn advance(curr: &mut Arc<Token>, prev: &mut Arc<Token>, source: &mut Source) {
    *prev = Arc::clone(curr);
    *curr = tokenizer::get_token(source);
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
            found,
            msg,
        } => {
            if let Some(msg) = msg {
                eprintln!("[{}] {msg}", format!("{source} {line}:{col}").red().bold());
            } else {
                eprintln!(
                    "[{}] :: {} is not a valid data type.",
                    format!("{source} {line}:{col}").red().bold(),
                    format!("{found}").red().italic()
                );
            }
        }
        ParseError::InvalidVariableDeclaration { line, column } => {
            eprintln!(
                "[{}] :: You can create a variable using a dynamic definition `:=` followed by the value to assign to the variable, or by specifying the datatype statically. You cannot create a variable without assign it a value.",
                format!("{source} {line}:{column}").red().bold()
            );
        }
    }
}
