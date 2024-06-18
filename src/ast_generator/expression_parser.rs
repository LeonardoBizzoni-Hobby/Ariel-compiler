use std::sync::Arc;

use crate::tokens::{error::ParseError, token_type::TokenType};

use super::{ast::expressions::Expression, parser_head::ParserHead, utils};

pub fn parse_expression(head: &mut ParserHead) -> Result<Expression, ParseError> {
    assignment_expression(head)
}

pub fn assignment_expression(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let left: Expression = or_expression(head)?;

    match head.curr.ttype {
        TokenType::Equal
        | TokenType::PlusEquals
        | TokenType::MinusEquals
        | TokenType::StarEquals
        | TokenType::SlashEquals
        | TokenType::PowerEquals
        | TokenType::ShiftLeftEqual
        | TokenType::ShiftRightEqual => {
            let _operation = Arc::clone(head.curr);
            utils::advance(head);
            let _value: Expression = or_expression(head)?;

            match left {
                Expression::Variable { name: _ } => todo!(),
                Expression::GetField { from: _, get: _ } => todo!(),
                _ => Err(ParseError::InvalidAssignmentExpression {
                    token: Arc::clone(head.curr),
                }),
            }
        }
        TokenType::Question => {
            let condition: Box<Expression> = Box::new(left);
            utils::advance(head);

            let true_branch: Box<Expression> = Box::new(assignment_expression(head)?);

            utils::require_token_type(head.curr, TokenType::Colon)?;
            utils::advance(head);

            Ok(Expression::Ternary {
                condition,
                true_branch,
                false_branch: Box::new(assignment_expression(head)?),
            })
        }
        _ => Ok(left),
    }
}

pub fn or_expression(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut left: Expression = and_expression(head)?;

    while matches!(head.curr.ttype, TokenType::Or | TokenType::BitOr) {
        utils::advance(head);

        left = Expression::Binary {
            left: Box::new(left),
            operation: Arc::clone(&head.prev),
            right: Box::new(and_expression(head)?),
        };
    }

    Ok(left)
}

pub fn and_expression(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut left: Expression = equality_check(head)?;

    while matches!(head.curr.ttype, TokenType::And | TokenType::BitAnd) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(equality_check(head)?),
        };
    }

    Ok(left)
}

pub fn equality_check(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut left: Expression = comparison_check(head)?;

    while matches!(head.curr.ttype, TokenType::EqualEqual | TokenType::NotEqual) {
        utils::advance(head);
        left = Expression::Binary {
            left: Box::new(left),
            operation: Arc::clone(&head.prev),
            right: Box::new(comparison_check(head)?),
        };
    }

    Ok(left)
}

pub fn comparison_check(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut left: Expression = term(head)?;

    while matches!(
        head.curr.ttype,
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual
    ) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(term(head)?),
        };
    }

    Ok(left)
}

pub fn term(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut left: Expression = factor(head)?;

    while matches!(
        head.curr.ttype,
        TokenType::Plus
            | TokenType::Minus
            | TokenType::Mod
            | TokenType::ShiftLeft
            | TokenType::ShiftRight
    ) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(factor(head)?),
        };
    }

    Ok(left)
}

pub fn factor(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut left: Expression = unary(head)?;

    while matches!(
        head.curr.ttype,
        TokenType::Star | TokenType::Slash | TokenType::Power | TokenType::IntegerSlash
    ) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Expression::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(unary(head)?),
        };
    }

    Ok(left)
}

pub fn unary(head: &mut ParserHead) -> Result<Expression, ParseError> {
    match head.curr.ttype {
        TokenType::Not | TokenType::Minus => Ok(Expression::Unary {
            operation: Arc::clone(head.curr),
            value: Box::new(unary(head)?),
        }),
        _ => call(head),
    }
}

// TODO: This allows parsing stuff like: `obj.field1()`. change it.
pub fn call(head: &mut ParserHead) -> Result<Expression, ParseError> {
    let mut expr: Expression = primary(head)?;

    loop {
        match head.curr.ttype {
            TokenType::LeftParen => {
                let mut args: Vec<Expression> = vec![];
                utils::advance(head);

                if !matches!(head.curr.ttype, TokenType::RightParen) {
                    args.push(parse_expression(head)?);

                    while !matches!(head.curr.ttype, TokenType::Comma) {
                        utils::advance(head);
                        args.push(parse_expression(head)?);
                    }
                }

                utils::require_token_type(head.curr, TokenType::RightParen)?;
                expr = Expression::FnCall {
                    fn_identifier: Box::new(expr),
                    args,
                };
            }
            TokenType::Dot => {
                utils::advance(head);

                utils::require_token_type(head.curr, TokenType::Identifier)?;
                let property = Arc::clone(head.curr);
                utils::advance(head);

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

pub fn primary(head: &mut ParserHead) -> Result<Expression, ParseError> {
    match head.curr.ttype {
        TokenType::Identifier => {
            utils::advance(head);
            Ok(Expression::Variable {
                name: Arc::clone(head.prev),
            })
        }
        TokenType::Integer
        | TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => {
            utils::advance(head);
            Ok(Expression::Literal {
                literal: Arc::clone(head.prev),
            })
        }
        TokenType::LeftParen => {
            utils::advance(head);
            let nested = Box::new(parse_expression(head)?);

            utils::require_token_type(head.curr, TokenType::RightParen)?;
            Ok(Expression::Nested { nested })
        }
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(head.curr),
        }),
    }
}
