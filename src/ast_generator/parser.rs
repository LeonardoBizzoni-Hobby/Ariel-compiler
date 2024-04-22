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
        TokenType::If => parse_conditional(curr, prev, source),
        TokenType::Match => todo!(),
        TokenType::While => parse_while_loop(curr, prev, source),
        TokenType::Loop => parse_loop(curr, prev, source),
        TokenType::For => todo!(),
        TokenType::LeftBrace => {
            advance(curr, prev, source);
            parse_scopebound_statement(curr, prev, source)
        }
        TokenType::Return => {
            advance(curr, prev, source);

            let expr: Expression = or_expression(curr, prev, source)?;

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
        TokenType::Break => {
            advance(curr, prev, source);

            require_token_type(curr, TokenType::Semicolon)?;
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::Break)
        }
        TokenType::Continue => {
            advance(curr, prev, source);

            require_token_type(curr, TokenType::Semicolon)?;
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::Continue)
        }
        _ => {
            let expr: Expression = parse_expression(curr, prev, source)?;

            require_token_type(curr, TokenType::Semicolon)?;
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::Expression(expr))
        }
    }
}

fn parse_loop(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    advance(curr, prev, source);
    let condition = Expression::Literal {
        literal: Arc::new(Token::new(
            curr.line,
            curr.column,
            TokenType::True,
            "true".to_owned(),
            curr.found_in.clone(),
        )),
    };

    match curr.ttype {
        TokenType::LeftBrace => {
            advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(parse_scope_block(curr, prev, source)?),
            })
        }
        TokenType::Semicolon => {
            advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            line: curr.line,
            column: curr.column,
        }),
    }
}

fn parse_while_loop(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    advance(curr, prev, source);
    let condition = parse_expression(curr, prev, source)?;

    match curr.ttype {
        TokenType::LeftBrace => {
            advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(parse_scope_block(curr, prev, source)?),
            })
        }
        TokenType::Semicolon => {
            advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            line: curr.line,
            column: curr.column,
        }),
    }
}

fn parse_conditional(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    let parse_branch = |curr: &mut Arc<Token>,
                        prev: &mut Arc<Token>,
                        source: &mut Source|
     -> Result<Vec<ScopeBoundStatement>, ParseError> {
        require_token_type(curr, TokenType::LeftBrace)?;
        advance(curr, prev, source);

        parse_scope_block(curr, prev, source)
    };

    advance(curr, prev, source);
    let condition: Expression = or_expression(curr, prev, source)?;
    let true_branch = parse_branch(curr, prev, source)?;

    match curr.ttype {
        TokenType::Else => {
            advance(curr, prev, source);

            Ok(ScopeBoundStatement::Conditional {
                condition,
                true_branch,
                false_branch: Some(match curr.ttype {
                    TokenType::If => vec![parse_conditional(curr, prev, source)?],
                    _ => parse_branch(curr, prev, source)?,
                }),
            })
        }
        _ => Ok(ScopeBoundStatement::Conditional {
            condition,
            true_branch,
            false_branch: None,
        }),
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
                        or_expression(curr, prev, source)?,
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
    assignment_expression(curr, prev, source)
}

fn assignment_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let left: Expression = or_expression(curr, prev, source)?;

    match curr.ttype {
        TokenType::Equal
        | TokenType::PlusEquals
        | TokenType::MinusEquals
        | TokenType::StarEquals
        | TokenType::SlashEquals
        | TokenType::PowerEquals
        | TokenType::ShiftLeftEqual
        | TokenType::ShiftRightEqual => {
            let _operation = Arc::clone(curr);
            advance(curr, prev, source);
            let _value: Expression = or_expression(curr, prev, source)?;

            match left {
                Expression::Variable { name: _ } => todo!(),
                Expression::GetField { from: _, get: _ } => todo!(),
                _ => Err(ParseError::InvalidAssignmentExpression {
                    token: Arc::clone(curr),
                }),
            }
        }
        TokenType::Question => {
            let condition: Box<Expression> = Box::new(left);
            advance(curr, prev, source);

            let true_branch: Box<Expression> = Box::new(assignment_expression(curr, prev, source)?);

            require_token_type(curr, TokenType::Colon)?;
            advance(curr, prev, source);

            Ok(Expression::Ternary {
                condition,
                true_branch,
                false_branch: Box::new(assignment_expression(curr, prev, source)?),
            })
        }
        _ => Ok(left),
    }
}

fn or_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = and_expression(curr, prev, source)?;

    while matches!(curr.ttype, TokenType::Or | TokenType::BitOr) {
        left = Expression::Binary {
            left: Box::new(left),
            operation: Arc::clone(&advance(curr, prev, source)),
            right: Box::new(and_expression(curr, prev, source)?),
        };
    }

    Ok(left)
}

fn and_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = equality_check(curr, prev, source)?;

    while matches!(curr.ttype, TokenType::And | TokenType::BitAnd) {
        let operation = Arc::clone(curr);
        advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(equality_check(curr, prev, source)?),
        };
    }

    Ok(left)
}

fn equality_check(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = comparison_check(curr, prev, source)?;

    while matches!(curr.ttype, TokenType::EqualEqual | TokenType::BangEqual) {
        left = Expression::Binary {
            left: Box::new(left),
            operation: Arc::clone(&advance(curr, prev, source)),
            right: Box::new(comparison_check(curr, prev, source)?),
        };
    }

    Ok(left)
}

fn comparison_check(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = term(curr, prev, source)?;

    while matches!(
        curr.ttype,
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual
    ) {
        let operation = Arc::clone(curr);
        advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(term(curr, prev, source)?),
        };
    }

    Ok(left)
}

fn term(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = factor(curr, prev, source)?;

    while matches!(
        curr.ttype,
        TokenType::Plus
            | TokenType::Minus
            | TokenType::Mod
            | TokenType::ShiftLeft
            | TokenType::ShiftRight
    ) {
        let operation = Arc::clone(curr);
        advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(factor(curr, prev, source)?),
        };
    }

    Ok(left)
}

fn factor(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = unary(curr, prev, source)?;

    while matches!(
        curr.ttype,
        TokenType::Star | TokenType::Slash | TokenType::Power
    ) {
        let operation = Arc::clone(curr);
        advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(unary(curr, prev, source)?),
        };
    }

    Ok(left)
}

fn unary(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    match curr.ttype {
        TokenType::Bang | TokenType::Minus => Ok(Expression::Unary {
            operation: Arc::clone(curr),
            value: Box::new(unary(curr, prev, source)?),
        }),
        _ => call(curr, prev, source),
    }
}

fn call(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut expr: Expression = primary(curr, prev, source)?;

    loop {
        match curr.ttype {
            TokenType::LeftParen => {
                let mut args: Vec<Expression> = vec![];
                advance(curr, prev, source);

                if !matches!(curr.ttype, TokenType::RightParen) {
                    args.push(parse_expression(curr, prev, source)?);

                    while !matches!(curr.ttype, TokenType::Comma) {
                        advance(curr, prev, source);
                        args.push(parse_expression(curr, prev, source)?);
                    }
                }

                require_token_type(curr, TokenType::RightParen)?;
                expr = Expression::FnCall {
                    fn_identifier: Box::new(expr),
                    args,
                };
            }
            TokenType::Dot => {
                advance(curr, prev, source);

                require_token_type(curr, TokenType::Identifier)?;
                let property = Arc::clone(curr);
                advance(curr, prev, source);

                expr = Expression::GetField {
                    from: Box::new(expr),
                    get: property,
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}

fn primary(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    match curr.ttype {
        TokenType::This => {
            advance(curr, prev, source);
            Ok(Expression::This)
        }
        TokenType::Identifier => Ok(Expression::Variable {
            name: advance(curr, prev, source),
        }),
        TokenType::Integer
        | TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => Ok(Expression::Literal {
            literal: advance(curr, prev, source),
        }),
        TokenType::LeftParen => {
            advance(curr, prev, source);
            let nested = Box::new(parse_expression(curr, prev, source)?);

            require_token_type(curr, TokenType::RightParen)?;
            Ok(Expression::Nested { nested })
        }
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(curr),
        }),
    }
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

fn advance(curr: &mut Arc<Token>, prev: &mut Arc<Token>, source: &mut Source) -> Arc<Token> {
    *prev = Arc::clone(curr);
    *curr = tokenizer::get_token(source);

    Arc::clone(prev)
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
        ParseError::LoopBodyNotFound { line, column } => {
            eprintln!("[{}] :: After a loop there must be either a scope block representing the body of the loop or a `;` for a loop without a body.",
                format!("{source} {line}:{column}").red().bold());
        }
        ParseError::InvalidAssignmentExpression { token } => {
            eprintln!(
                "[{}] :: Invalid assignment expression, RTFM!",
                format!("{} {}:{}", token.found_in, token.line, token.column)
                    .red()
                    .bold()
            );
        }
        ParseError::InvalidExpression { token } =>{
            eprintln!(
                "[{}] :: Invalid expression, RTFM!",
                format!("{} {}:{}", token.found_in, token.line, token.column)
                .red()
                .bold()
            );
        }
    }
}
