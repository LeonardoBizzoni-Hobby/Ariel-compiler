use super::{expressions::Expression, variables::Variable};

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
    For,
    Expression(Expression),
    Break,
    Continue,
}
