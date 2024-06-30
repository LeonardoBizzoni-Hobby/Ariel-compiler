use super::{expressions::Expression, variables::Variable};

#[derive(Debug)]
pub enum ScopeBoundStatement {
    Scope(Vec<ScopeBoundStatement>),
    VariableDeclaration(Box<Variable>),
    Return(Expression),
    Conditional {
        condition: Expression,
        true_branch: Box<ScopeBoundStatement>,
        false_branch: Option<Box<ScopeBoundStatement>>,
    },
    Match,
    While {
        condition: Expression,
        body: Option<Box<ScopeBoundStatement>>,
    },
    For,
    Expression(Expression),
    Break,
    Continue,
}
