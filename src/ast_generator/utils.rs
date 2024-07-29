use colored::Colorize;

use crate::tokens::error::ParseError;

pub fn print_error(source: &str, after: &str, e: ParseError) {
    match e {
        ParseError::UnexpectedToken {
            token,
            expected,
            msg,
        } => {
            if let Some(msg) = msg {
                eprintln!(
                    "[{}] {msg}",
                    format!("{source} {}:{}", token.line, token.column)
                        .red()
                        .bold()
                );
            } else {
                eprintln!("[{}] :: there should have been a {} after the token `{after}`, but instead there was a {}.",
                format!("{source} {}:{}", token.line, token.column).red().bold(),
                format!("{expected}").blue().bold(),
                format!("{}", token.found_in).red().italic());
            }
        }
        ParseError::InvalidDataType { token, msg } => {
            if let Some(msg) = msg {
                eprintln!(
                    "[{}] {msg}",
                    format!("{source} {}:{}", token.line, token.column)
                        .red()
                        .bold()
                );
            } else {
                eprintln!(
                    "[{}] :: {} is not a valid data type.",
                    format!("{source} {}:{}", token.line, token.column)
                        .red()
                        .bold(),
                    format!("{}", token.found_in).red().italic()
                );
            }
        }
        ParseError::InvalidVariableDeclaration { token } => {
            eprintln!(
                "[{}] :: You can create a variable using a dynamic definition `:=` followed by the value to assign to the variable, or by specifying the datatype statically. You cannot create a variable without assign it a value.",
                format!("{source} {}:{}", token.line, token.column).red().bold()
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
        // ParseError::InvalidIterator { token, msg } => {
        //     if let Some(msg) = msg {
        //         eprintln!(
        //             "[{}] {msg}",
        //             format!("{} {}:{}", token.found_in, token.line, token.column)
        //                 .red()
        //                 .bold()
        //         );
        //     } else {
        //         eprintln!(
        //             "[{}] :: {} is not a valid iterator.",
        //             format!("{} {}:{}", token.found_in, token.line, token.column)
        //                 .red()
        //                 .bold(),
        //             format!("{} ({})", token.lexeme, token.ttype).red().italic()
        //         );
        //     }
        // }
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
        ParseError::InvalidAddressOfValue { at } => {
            eprintln!(
                "[{}] :: {} is not a at assignable to a variable.",
                format!("{} {}:{}", at.found_in, at.line, at.column)
                    .red()
                    .bold(),
                format!("{}", at.lexeme).red().italic()
            );
        }
    }
}
