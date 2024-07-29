use crate::ast_generator::ast::expressions::Expression;

use super::{token::Token, token_type::TokenType};

#[derive(Debug)]
pub enum Error {
    FileNotFound(String, String),
    MemoryMapFiled(String, String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        token: Box<Token>,
        expected: TokenType,
        msg: Option<String>,
    },
    InvalidDataType {
        token: Box<Token>,
        msg: Option<String>,
    },
    InvalidVariableDeclaration {
        token: Box<Token>,
    },
    LoopBodyNotFound {
        body: Box<Token>,
    },
    InvalidAssignmentExpression {
        operation: Box<Token>,
        assign_to: Box<Expression>,
    },
    InvalidExpression {
        token: Box<Token>,
    },
    // InvalidIterator {
    //     token: Box<Token>,
    //     msg: Option<String>,
    // },
    InvalidFnName {
        name: Box<Token>,
    },
    InvalidFnBody {
        body: Box<Token>,
    },
    InvalidVariableAssignment {
        value: Box<Token>,
    },
    InvalidAddressOfValue {
        at: Box<Token>,
    },
}
