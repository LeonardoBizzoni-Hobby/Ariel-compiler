use std::sync::Arc;

use colored::Colorize;

use crate::tokens::{error::ParseError, token::Token, token_type::TokenType, tokenizer};

use super::{ast::variables::DataType, parser_head::ParserHead};

pub fn parse_datatype(head: &mut ParserHead) -> Result<DataType, ParseError> {
    match head.curr.ttype {
        TokenType::U8 => Ok(handle_pointer_datatype(DataType::U8, head)),
        TokenType::U16 => Ok(handle_pointer_datatype(DataType::U16, head)),
        TokenType::U32 => Ok(handle_pointer_datatype(DataType::U32, head)),
        TokenType::U64 => Ok(handle_pointer_datatype(DataType::U64, head)),
        TokenType::Usize => Ok(handle_pointer_datatype(DataType::Usize, head)),
        TokenType::I8 => Ok(handle_pointer_datatype(DataType::I8, head)),
        TokenType::I16 => Ok(handle_pointer_datatype(DataType::I16, head)),
        TokenType::I32 => Ok(handle_pointer_datatype(DataType::I32, head)),
        TokenType::I64 => Ok(handle_pointer_datatype(DataType::I64, head)),
        TokenType::Isize => Ok(handle_pointer_datatype(DataType::Isize, head)),
        TokenType::F32 => Ok(handle_pointer_datatype(DataType::F32, head)),
        TokenType::F64 => Ok(handle_pointer_datatype(DataType::F64, head)),
        TokenType::Bool => Ok(handle_pointer_datatype(DataType::Bool, head)),
        TokenType::StringType => Ok(handle_pointer_datatype(DataType::String, head)),
        TokenType::Void => {
            let datatype = handle_pointer_datatype(DataType::Void, head);
            if matches!(datatype, DataType::Pointer(_)) {
                Ok(datatype)
            } else {
                Err(ParseError::InvalidDataType {
                    line: head.curr.line,
                    col: head.curr.column,
                    found: head.curr.ttype.clone(),
                    msg: Some(
                        "`void` by itself isn't a valid datatype, it should have been a void pointer `void*`."
                            .to_owned(),
                    ),
                })
            }
        }
        TokenType::LeftSquare => {
            advance(head);
            let array_of: DataType = parse_datatype(head)?;

            require_token_type(head.curr, TokenType::RightSquare)?;
            Ok(handle_pointer_datatype(
                DataType::Array(Box::new(array_of)),
                head,
            ))
        }
        TokenType::Identifier => Ok(handle_pointer_datatype(
            DataType::Compound {
                name: Arc::clone(head.curr),
            },
            head,
        )),
        _ => Err(ParseError::InvalidDataType {
            line: head.curr.line,
            col: head.curr.column,
            found: head.curr.ttype.clone(),
            msg: None,
        }),
    }
}

fn handle_pointer_datatype(datatype: DataType, head: &mut ParserHead) -> DataType {
    let mut res = datatype;

    // datatype -> *
    // datatype -> ,
    // datatype -> )
    // datatype -> {
    advance(head);

    while matches!(head.curr.ttype, TokenType::Star) {
        advance(head);
        res = DataType::Pointer(Box::new(res));
    }

    res
}

pub fn require_token_type(curr: &Token, expected: TokenType) -> Result<(), ParseError> {
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

pub fn advance(head: &mut ParserHead) {
    *head.prev = Arc::clone(head.curr);
    *head.curr = tokenizer::get_token(head.source);
}

pub fn print_error(source: &str, after: &str, e: ParseError) {
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
        ParseError::LoopBodyNotFound { body } => {
            eprintln!("[{}] :: After a loop there must be either a scope block representing the body of the loop or a `;` for a loop without a body.",
                format!("{} {}:{}", body.found_in, body.line, body.column).red().bold());
        }
        ParseError::InvalidAssignmentExpression {
            operation,
            assign_to,
        } => {
            eprintln!(
                "[{}] :: Invalid assignment expression, can't assign a value to `{}`!",
                format!(
                    "{} {}:{}",
                    operation.found_in, operation.line, operation.column
                )
                .red()
                .bold(),
                assign_to
            );
        }
        ParseError::InvalidExpression { token } => {
            eprintln!(
                "[{}] :: Invalid expression, RTFM!",
                format!("{} {}:{}", token.found_in, token.line, token.column)
                    .red()
                    .bold()
            );
        }
        ParseError::InvalidIterator { token, msg } => {
            if let Some(msg) = msg {
                eprintln!(
                    "[{}] {msg}",
                    format!("{} {}:{}", token.found_in, token.line, token.column)
                        .red()
                        .bold()
                );
            } else {
                eprintln!(
                    "[{}] :: {} is not a valid iterator.",
                    format!("{} {}:{}", token.found_in, token.line, token.column)
                        .red()
                        .bold(),
                    format!("{} ({})", token.lexeme, token.ttype).red().italic()
                );
            }
        }
        ParseError::InvalidFnName { name } => {
            eprintln!(
                "[{}] :: {} is not a valid function name.",
                format!("{} {}:{}", name.found_in, name.line, name.column)
                    .red()
                    .bold(),
                format!("{} ({})", name.lexeme, name.ttype).red().italic()
            );
        }
        ParseError::InvalidFnBody { body } => {
            eprintln!(
                "[{}] :: {} is not a valid function body.",
                format!("{} {}:{}", body.found_in, body.line, body.column)
                    .red()
                    .bold(),
                format!("{} ({})", body.lexeme, body.ttype).red().italic()
            );
        }
        ParseError::InvalidVariableAssignment { value } => {
            eprintln!(
                "[{}] :: {} is not a value assignable to a variable.",
                format!("{} {}:{}", value.found_in, value.line, value.column)
                    .red()
                    .bold(),
                format!("{}", value.lexeme).red().italic()
            );
        }
    }
}
