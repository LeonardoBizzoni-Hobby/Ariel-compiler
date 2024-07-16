use std::collections::HashMap;

use super::{expressions::Expression, variables::Variable};

#[derive(Debug, Eq, PartialEq)]
pub enum ScopeBoundStatement {
    Scope(Vec<ScopeBoundStatement>),

    VariableDeclaration(Variable),

    Return(Box<Expression>),
    ImplicitReturn(Box<Expression>),
    Expression(Box<Expression>),
    Defer(Box<ScopeBoundStatement>),

    Conditional {
        condition: Box<Expression>,
        true_branch: Vec<ScopeBoundStatement>,
        false_branch: Option<Vec<ScopeBoundStatement>>,
    },
    Match {
        on: Box<Expression>,
        cases: HashMap<Expression, Vec<ScopeBoundStatement>>,
    },

    Loop(Option<Vec<ScopeBoundStatement>>),
    While {
        condition: Box<Expression>,
        body: Option<Vec<ScopeBoundStatement>>,
    },
    For {
        initialization: Option<Box<ScopeBoundStatement>>,
        condition: Option<Box<ScopeBoundStatement>>,
        increment: Option<Box<Expression>>,
        body: Option<Vec<ScopeBoundStatement>>,
    },

    Break,
    Continue,
}
