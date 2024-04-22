use std::sync::Arc;

use crate::tokens::token::Token;

use self::function::Function;

pub mod function;
pub mod function_field;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ScopeBoundStatement {
    VariableDeclaration(Box<Variable>),
    Return(Expression),
    Conditional {
        condition: Expression,
        true_branch: Vec<ScopeBoundStatement>,
        false_branch: Option<Vec<ScopeBoundStatement>>,
    },
    Match,
    While {
        condition: Expression,
        body: Option<Vec<ScopeBoundStatement>>,
    },
    Loop,
    For,
    Expression(Expression),
    Break,
    Continue,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Variable {
    name: Arc<Token>,
    datatype: Option<DataType>,
    value: Expression,
}

impl Variable {
    pub fn new(name: Arc<Token>, datatype: Option<DataType>, value: Expression) -> Self {
        Self {
            name,
            datatype,
            value,
        }
    }
}

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
    This,
    Literal {
        literal: Arc<Token>,
    },
    Nested {
        nested: Box<Expression>,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Ast {
    Integer(i32),
    Fn(Function),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DataType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bool,
    Void,
    Array(Box<DataType>),
    Pointer(Box<DataType>),
}
