use std::sync::Arc;

use crate::tokens::{error::ParseError, source::Source, token::Token, token_type::TokenType};

use super::{ast::expressions::Expression, utils};

pub fn parse_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    assignment_expression(curr, prev, source)
}

pub fn assignment_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let left: Expression = or_expression(curr, prev, source)?;

    match curr.ttype {
        TokenType::Equal
        | TokenType::PlusEquals
        | TokenType::MinusEquals
        | TokenType::StarEquals
        | TokenType::SlashEquals
        | TokenType::PowerEquals
        | TokenType::ShiftLeftEqual
        | TokenType::ShiftRightEqual => {
            let _operation = Arc::clone(curr);
            utils::advance(curr, prev, source);
            let _value: Expression = or_expression(curr, prev, source)?;

            match left {
                Expression::Variable { name: _ } => todo!(),
                Expression::GetField { from: _, get: _ } => todo!(),
                _ => Err(ParseError::InvalidAssignmentExpression {
                    token: Arc::clone(curr),
                }),
            }
        }
        TokenType::Question => {
            let condition: Box<Expression> = Box::new(left);
            utils::advance(curr, prev, source);

            let true_branch: Box<Expression> = Box::new(assignment_expression(curr, prev, source)?);

            utils::require_token_type(curr, TokenType::Colon)?;
            utils::advance(curr, prev, source);

            Ok(Expression::Ternary {
                condition,
                true_branch,
                false_branch: Box::new(assignment_expression(curr, prev, source)?),
            })
        }
        _ => Ok(left),
    }
}

pub fn or_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = and_expression(curr, prev, source)?;

    while matches!(curr.ttype, TokenType::Or | TokenType::BitOr) {
        left = Expression::Binary {
            left: Box::new(left),
            operation: Arc::clone(&utils::advance(curr, prev, source)),
            right: Box::new(and_expression(curr, prev, source)?),
        };
    }

    Ok(left)
}

pub fn and_expression(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = equality_check(curr, prev, source)?;

    while matches!(curr.ttype, TokenType::And | TokenType::BitAnd) {
        let operation = Arc::clone(curr);
        utils::advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(equality_check(curr, prev, source)?),
        };
    }

    Ok(left)
}

pub fn equality_check(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = comparison_check(curr, prev, source)?;

    while matches!(curr.ttype, TokenType::EqualEqual | TokenType::BangEqual) {
        left = Expression::Binary {
            left: Box::new(left),
            operation: Arc::clone(&utils::advance(curr, prev, source)),
            right: Box::new(comparison_check(curr, prev, source)?),
        };
    }

    Ok(left)
}

pub fn comparison_check(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = term(curr, prev, source)?;

    while matches!(
        curr.ttype,
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual
    ) {
        let operation = Arc::clone(curr);
        utils::advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(term(curr, prev, source)?),
        };
    }

    Ok(left)
}

pub fn term(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = factor(curr, prev, source)?;

    while matches!(
        curr.ttype,
        TokenType::Plus
            | TokenType::Minus
            | TokenType::Mod
            | TokenType::ShiftLeft
            | TokenType::ShiftRight
    ) {
        let operation = Arc::clone(curr);
        utils::advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(factor(curr, prev, source)?),
        };
    }

    Ok(left)
}

pub fn factor(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut left: Expression = unary(curr, prev, source)?;

    while matches!(
        curr.ttype,
        TokenType::Star | TokenType::Slash | TokenType::Power
    ) {
        let operation = Arc::clone(curr);
        utils::advance(curr, prev, source);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(unary(curr, prev, source)?),
        };
    }

    Ok(left)
}

pub fn unary(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    match curr.ttype {
        TokenType::Bang | TokenType::Minus => Ok(Expression::Unary {
            operation: Arc::clone(curr),
            value: Box::new(unary(curr, prev, source)?),
        }),
        _ => call(curr, prev, source),
    }
}

pub fn call(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    let mut expr: Expression = primary(curr, prev, source)?;

    loop {
        match curr.ttype {
            TokenType::LeftParen => {
                let mut args: Vec<Expression> = vec![];
                utils::advance(curr, prev, source);

                if !matches!(curr.ttype, TokenType::RightParen) {
                    args.push(parse_expression(curr, prev, source)?);

                    while !matches!(curr.ttype, TokenType::Comma) {
                        utils::advance(curr, prev, source);
                        args.push(parse_expression(curr, prev, source)?);
                    }
                }

                utils::require_token_type(curr, TokenType::RightParen)?;
                expr = Expression::FnCall {
                    fn_identifier: Box::new(expr),
                    args,
                };
            }
            TokenType::Dot => {
                utils::advance(curr, prev, source);

                utils::require_token_type(curr, TokenType::Identifier)?;
                let property = Arc::clone(curr);
                utils::advance(curr, prev, source);

                expr = Expression::GetField {
                    from: Box::new(expr),
                    get: property,
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}

pub fn primary(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Expression, ParseError> {
    match curr.ttype {
        TokenType::This => {
            utils::advance(curr, prev, source);
            Ok(Expression::This)
        }
        TokenType::Identifier => Ok(Expression::Variable {
            name: utils::advance(curr, prev, source),
        }),
        TokenType::Integer
        | TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => Ok(Expression::Literal {
            literal: utils::advance(curr, prev, source),
        }),
        TokenType::LeftParen => {
            utils::advance(curr, prev, source);
            let nested = Box::new(parse_expression(curr, prev, source)?);

            utils::require_token_type(curr, TokenType::RightParen)?;
            Ok(Expression::Nested { nested })
        }
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(curr),
        }),
    }
}
