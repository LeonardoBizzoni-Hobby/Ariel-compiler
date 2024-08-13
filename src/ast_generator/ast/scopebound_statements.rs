use std::collections::HashMap;

use super::{expressions::Expression, variables::Variable};

#[derive(Debug, Eq, PartialEq)]
pub enum ScopeBoundStatement {
    Scope {
        line: usize,
        column: usize,
        body: Vec<ScopeBoundStatement>,
    },

    VariableDeclaration {
        line: usize,
        column: usize,
        var: Variable,
    },

    Return {
        line: usize,
        column: usize,
        value: Option<Box<ScopeBoundStatement>>,
    },
    ImplicitReturn {
        line: usize,
        column: usize,
        expr: Expression,
    },
    Expression {
        line: usize,
        column: usize,
        expr: Expression,
    },
    Defer {
        line: usize,
        column: usize,
        stmt: Box<ScopeBoundStatement>,
    },

    Conditional {
        line: usize,
        column: usize,
        condition: Expression,
        true_branch: Vec<ScopeBoundStatement>,
        false_branch: Option<Vec<ScopeBoundStatement>>,
    },
    Match {
        line: usize,
        column: usize,
        on: Expression,
        cases: HashMap<Expression, Vec<ScopeBoundStatement>>,
    },

    Loop {
        line: usize,
        column: usize,
        body: Option<Vec<ScopeBoundStatement>>,
    },
    While {
        line: usize,
        column: usize,
        condition: Expression,
        body: Option<Vec<ScopeBoundStatement>>,
    },
    For {
        line: usize,
        column: usize,
        initialization: Option<Box<ScopeBoundStatement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Option<Vec<ScopeBoundStatement>>,
    },

    Break {
        line: usize,
        column: usize,
    },
    Continue {
        line: usize,
        column: usize,
    },
}

impl ScopeBoundStatement {
    pub fn line(&self) -> usize {
        match self {
            ScopeBoundStatement::Scope { line, .. } => *line,
            ScopeBoundStatement::VariableDeclaration { line, .. } => *line,
            ScopeBoundStatement::Return { line, .. } => *line,
            ScopeBoundStatement::ImplicitReturn { line, .. } => *line,
            ScopeBoundStatement::Expression { line, .. } => *line,
            ScopeBoundStatement::Defer { line, .. } => *line,
            ScopeBoundStatement::Conditional { line, .. } => *line,
            ScopeBoundStatement::Match { line, .. } => *line,
            ScopeBoundStatement::Loop { line, .. } => *line,
            ScopeBoundStatement::While { line, .. } => *line,
            ScopeBoundStatement::For { line, .. } => *line,
            ScopeBoundStatement::Break { line, .. } => *line,
            ScopeBoundStatement::Continue { line, .. } => *line,
        }
    }

    pub fn column(&self) -> usize {
        match self {
            ScopeBoundStatement::Scope { column, .. } => *column,
            ScopeBoundStatement::VariableDeclaration { column, .. } => *column,
            ScopeBoundStatement::Return { column, .. } => *column,
            ScopeBoundStatement::ImplicitReturn { column, .. } => *column,
            ScopeBoundStatement::Expression { column, .. } => *column,
            ScopeBoundStatement::Defer { column, .. } => *column,
            ScopeBoundStatement::Conditional { column, .. } => *column,
            ScopeBoundStatement::Match { column, .. } => *column,
            ScopeBoundStatement::Loop { column, .. } => *column,
            ScopeBoundStatement::While { column, .. } => *column,
            ScopeBoundStatement::For { column, .. } => *column,
            ScopeBoundStatement::Break { column, .. } => *column,
            ScopeBoundStatement::Continue { column, .. } => *column,
        }
    }
}
