use std::{fmt::Display, sync::Arc};

use crate::tokens::token::Token;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    Name {
        name: Arc<Token>,
    },
    GetField {
        from: Box<Expression>,
        get: Arc<Token>,
    },
    Binary {
        left: Box<Expression>,
        operation: Arc<Token>,
        right: Box<Expression>,
    },
    Unary {
        operation: Arc<Token>,
        value: Box<Expression>,
    },
    FnCall {
        fn_identifier: Box<Expression>,
        args: Vec<Expression>,
    },
    Literal {
        literal: Arc<Token>,
    },
    Nested {
        nested: Box<Expression>,
    },
    Monad {
        value: Box<Expression>,
    },
    Sequence {
        start: isize,
        end: isize,
    },
    AddressOf {
        of: Box<Expression>,
    },
    ArrayLiteral {
        values: Vec<Box<Expression>>,
    },
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Name { name } => write!(f, "variable: {}", name.lexeme),
            Expression::GetField { from, get } => write!(f, "get_field {} from {from}", get.lexeme),
            Expression::Binary {
                left,
                operation,
                right,
            } => write!(f, "{left} {} {right}", operation.lexeme),
            Expression::Unary { operation, value } => write!(f, "{} {value}", operation.lexeme),
            Expression::FnCall {
                fn_identifier,
                args,
            } => {
                for arg in args.iter() {
                    match write!(f, "{fn_identifier}({arg},") {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                }
                write!(f, ")")
            }
            Expression::Literal { literal } => write!(f, "literal {}", literal.lexeme),
            Expression::Nested { nested } => write!(f, "({nested})"),
            Expression::Monad { value } => write!(f, "monadic expression {value}"),
            Expression::Sequence { start, end } => {
                write!(f, "sequence from `{start}` to `{end}` included")
            }
            Expression::AddressOf { of } => write!(f, "address of: {of}"),
            Expression::ArrayLiteral { values } => {
                let mut str = String::from("[ ");
                for value in values {
                    str.push_str(&format!("{value},"));
                }

                write!(f, "{str} ]")
            },
        }
    }
}
