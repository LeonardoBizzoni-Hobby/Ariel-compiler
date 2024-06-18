use std::sync::Arc;

use crate::tokens::token::Token;

#[derive(Debug)]
pub enum Expression {
    Variable {
        name: Arc<Token>,
    },
    GetField {
        from: Box<Expression>,
        get: Arc<Token>,
    },
    Ternary {
        condition: Box<Expression>,
        true_branch: Box<Expression>,
        false_branch: Box<Expression>,
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
}
