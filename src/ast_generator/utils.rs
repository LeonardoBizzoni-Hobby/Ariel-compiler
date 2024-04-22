use std::sync::Arc;

use colored::Colorize;

use crate::tokens::{
    error::ParseError, source::Source, token::Token, token_type::TokenType, tokenizer,
};

use super::ast::variables::DataType;

pub fn parse_datatype(
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

pub fn advance(curr: &mut Arc<Token>, prev: &mut Arc<Token>, source: &mut Source) -> Arc<Token> {
    *prev = Arc::clone(curr);
    *curr = tokenizer::get_token(source);

    Arc::clone(prev)
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
        ParseError::InvalidExpression { token } => {
            eprintln!(
                "[{}] :: Invalid expression, RTFM!",
                format!("{} {}:{}", token.found_in, token.line, token.column)
                    .red()
                    .bold()
            );
        }
    }
}
