use std::collections::HashMap;

use super::{expressions::Expression, variables::Variable};

#[derive(Debug, Eq, PartialEq)]
pub enum ScopeBoundStatement {
    Scope(Vec<ScopeBoundStatement>),

    VariableDeclaration(Variable),

    Return(Option<Box<ScopeBoundStatement>>),
    ImplicitReturn(Expression),
    Expression(Expression),
    Defer(Box<ScopeBoundStatement>),

    Conditional {
        condition: Expression,
        true_branch: Vec<ScopeBoundStatement>,
        false_branch: Option<Vec<ScopeBoundStatement>>,
    },
    Match {
        on: Expression,
        cases: HashMap<Expression, Vec<ScopeBoundStatement>>,
    },

    Loop(Option<Vec<ScopeBoundStatement>>),
    While {
        condition: Expression,
        body: Option<Vec<ScopeBoundStatement>>,
    },
    For {
        initialization: Option<Box<ScopeBoundStatement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Option<Vec<ScopeBoundStatement>>,
    },

    Break,
    Continue,
}
