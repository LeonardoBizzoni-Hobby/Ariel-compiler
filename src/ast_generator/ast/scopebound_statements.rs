use std::collections::HashMap;

use super::{expressions::Expression, variables::Variable};

#[derive(Debug)]
pub enum ScopeBoundStatement {
    Scope(Vec<ScopeBoundStatement>),
    VariableDeclaration(Variable),
    Return(Box<Expression>),
    Conditional {
        condition: Box<Expression>,
        true_branch: Box<ScopeBoundStatement>,
        false_branch: Option<Box<ScopeBoundStatement>>,
    },
    Match {
        on: Box<Expression>,
        cases: HashMap<Expression, ScopeBoundStatement>,
    },
    While {
        condition: Box<Expression>,
        body: Option<Box<ScopeBoundStatement>>,
    },
    For {
        initialization: Option<Box<ScopeBoundStatement>>,
        condition: Option<Box<ScopeBoundStatement>>,
        increment: Option<Box<Expression>>,
        body: Box<ScopeBoundStatement>,
    },
    Expression(Box<Expression>),
    Break,
    Continue,
}
