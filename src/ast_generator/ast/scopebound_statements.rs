use std::collections::HashMap;

use super::{expressions::Expression, variables::Variable};

#[derive(Debug)]
pub enum ScopeBoundStatement {
    Scope(Vec<ScopeBoundStatement>),
    VariableDeclaration(Variable),
    Return(Expression),
    Conditional {
        condition: Expression,
        true_branch: Box<ScopeBoundStatement>,
        false_branch: Option<Box<ScopeBoundStatement>>,
    },
    Match {
        on: Expression,
        cases: HashMap<Expression, ScopeBoundStatement>,
    },
    While {
        condition: Expression,
        body: Option<Box<ScopeBoundStatement>>,
    },
    For {
        initialization: Option<Box<ScopeBoundStatement>>,
        condition: Option<Box<ScopeBoundStatement>>,
        increment: Option<Expression>,
        body: Box<ScopeBoundStatement>,
    },
    Expression(Expression),
    Break,
    Continue,
}
