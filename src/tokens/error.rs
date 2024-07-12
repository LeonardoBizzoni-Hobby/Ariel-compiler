use std::sync::Arc;

use crate::ast_generator::ast::expressions::Expression;

use super::{token::Token, token_type::TokenType};

#[derive(Debug)]
pub enum Error {
    FileNotFound(String, String),
    MemoryMapFiled(String, String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    UnexpectedToken {
        line: usize,
        col: usize,
        found: TokenType,
        expected: TokenType,
        msg: Option<String>,
    },
    InvalidDataType {
        line: usize,
        col: usize,
        found: TokenType,
        msg: Option<String>,
    },
    InvalidVariableDeclaration {
        line: usize,
        column: usize,
    },
    LoopBodyNotFound {
        body: Arc<Token>,
    },
    InvalidAssignmentExpression {
        operation: Arc<Token>,
        assign_to: Box<Expression>,
    },
    InvalidExpression {
        token: Arc<Token>,
    },
    InvalidIterator { token: Arc<Token>, msg: Option<String> },
    InvalidFnName { name: Arc<Token> },
    InvalidFnBody { body: Arc<Token> },
}
