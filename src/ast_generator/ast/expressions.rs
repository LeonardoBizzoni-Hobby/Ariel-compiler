use std::fmt::Display;

use crate::tokens::token::Token;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    Name {
        line: usize,
        column: usize,
        name: Box<Token>,
    },
    GetField {
        line: usize,
        column: usize,
        from: Box<Expression>,
        get: Box<Token>,
    },
    Binary {
        line: usize,
        column: usize,
        left: Box<Expression>,
        operation: Box<Token>,
        right: Box<Expression>,
    },
    Unary {
        line: usize,
        column: usize,
        operation: Box<Token>,
        value: Box<Expression>,
    },
    FnCall {
        line: usize,
        column: usize,
        fn_identifier: Box<Expression>,
        args: Vec<Expression>,
    },
    Literal {
        line: usize,
        column: usize,
        literal: Box<Token>,
    },
    Nested {
        line: usize,
        column: usize,
        nested: Box<Expression>,
    },
    Monad {
        line: usize,
        column: usize,
        value: Box<Expression>,
    },
    Sequence {
        line: usize,
        column: usize,
        start: isize,
        end: isize,
    },
    AddressOf {
        line: usize,
        column: usize,
        of: Box<Expression>,
    },
    ArrayLiteral {
        line: usize,
        column: usize,
        values: Vec<Box<Expression>>,
    },
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Name { name, .. } => write!(f, "variable: {}", name.lexeme),
            Expression::GetField { from, get, .. } => {
                write!(f, "get_field {} from {from}", get.lexeme)
            }
            Expression::Binary {
                left,
                operation,
                right,
                ..
            } => write!(f, "{left} {} {right}", operation.lexeme),
            Expression::Unary {
                operation, value, ..
            } => write!(f, "{} {value}", operation.lexeme),
            Expression::FnCall {
                fn_identifier,
                args,
                ..
            } => {
                for arg in args.iter() {
                    match write!(f, "{fn_identifier}({arg},") {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                }
                write!(f, ")")
            }
            Expression::Literal { literal, .. } => write!(f, "literal {}", literal.lexeme),
            Expression::Nested { nested, .. } => write!(f, "({nested})"),
            Expression::Monad { value, .. } => write!(f, "monadic expression {value}"),
            Expression::Sequence { start, end, .. } => {
                write!(f, "sequence from `{start}` to `{end}` included")
            }
            Expression::AddressOf { of, .. } => write!(f, "address of: {of}"),
            Expression::ArrayLiteral { values, .. } => {
                let mut str = String::from("[ ");
                for value in values {
                    str.push_str(&format!("{value},"));
                }

                write!(f, "{str} ]")
            }
        }
    }
}

impl Expression {
    pub fn line(&self) -> usize {
        match self {
            Expression::Name { line, .. } => *line,
            Expression::GetField { line, .. } => *line,
            Expression::Binary { line, .. } => *line,
            Expression::Unary { line, .. } => *line,
            Expression::FnCall { line, .. } => *line,
            Expression::Literal { line, .. } => *line,
            Expression::Nested { line, .. } => *line,
            Expression::Monad { line, .. } => *line,
            Expression::Sequence { line, .. } => *line,
            Expression::AddressOf { line, .. } => *line,
            Expression::ArrayLiteral { line, .. } => *line,
        }
    }

    pub fn column(&self) -> usize {
        match self {
            Expression::Name { column, .. } => *column,
            Expression::GetField { column, .. } => *column,
            Expression::Binary { column, .. } => *column,
            Expression::Unary { column, .. } => *column,
            Expression::FnCall { column, .. } => *column,
            Expression::Literal { column, .. } => *column,
            Expression::Nested { column, .. } => *column,
            Expression::Monad { column, .. } => *column,
            Expression::Sequence { column, .. } => *column,
            Expression::AddressOf { column, .. } => *column,
            Expression::ArrayLiteral { column, .. } => *column,
        }
    }
}
