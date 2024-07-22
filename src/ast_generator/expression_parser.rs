use std::sync::Arc;

use crate::tokens::{error::ParseError, token_type::TokenType};

use super::{
    ast::expressions::Expression,
    parser_head::ParserHead,
    utils::{self},
};

pub fn parse_expression(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let left: Box<Expression> = assignment_expression(head)?;

    match head.curr.ttype {
        TokenType::Question => Ok(Box::new(Expression::Monad { value: left })),
        _ => Ok(left),
    }
}

pub fn assignment_expression(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let left: Box<Expression> = or_expression(head)?;

    match head.curr.ttype {
        TokenType::Equal
        | TokenType::PlusEquals
        | TokenType::MinusEquals
        | TokenType::StarEquals
        | TokenType::SlashEquals
        | TokenType::PowerEquals
        | TokenType::ShiftLeftEqual
        | TokenType::ShiftRightEqual => {
            let operation = Arc::clone(head.curr);
            utils::advance(head);
            let value: Box<Expression> = or_expression(head)?;

            match *left {
                Expression::GetField { from: _, get: _ } | Expression::Name { name: _ } => {
                    Ok(Box::new(Expression::Binary {
                        left,
                        operation,
                        right: value,
                    }))
                }
                _ => Err(ParseError::InvalidAssignmentExpression {
                    operation,
                    assign_to: left,
                }),
            }
        }
        _ => Ok(left),
    }
}

pub fn or_expression(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = and_expression(head)?;

    while matches!(head.curr.ttype, TokenType::Or | TokenType::BitOr) {
        utils::advance(head);

        left = Box::new(Expression::Binary {
            left,
            operation: Arc::clone(&head.prev),
            right: and_expression(head)?,
        });
    }

    Ok(left)
}

pub fn and_expression(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = equality_check(head)?;

    while matches!(head.curr.ttype, TokenType::And | TokenType::BitAnd) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Box::new(Expression::Binary {
            left,
            operation,
            right: equality_check(head)?,
        });
    }

    Ok(left)
}

pub fn equality_check(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = comparison_check(head)?;

    while matches!(head.curr.ttype, TokenType::EqualEqual | TokenType::NotEqual) {
        utils::advance(head);
        left = Box::new(Expression::Binary {
            left,
            operation: Arc::clone(&head.prev),
            right: comparison_check(head)?,
        });
    }

    Ok(left)
}

pub fn comparison_check(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = term(head)?;

    while matches!(
        head.curr.ttype,
        TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual
    ) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Box::new(Expression::Binary {
            left,
            operation,
            right: term(head)?,
        });
    }

    Ok(left)
}

pub fn term(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = factor(head)?;

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

        left = Box::new(Expression::Binary {
            left,
            operation,
            right: factor(head)?,
        });
    }

    Ok(left)
}

pub fn factor(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = unary(head)?;

    while matches!(
        head.curr.ttype,
        TokenType::Star | TokenType::Slash | TokenType::Power | TokenType::IntegerSlash
    ) {
        let operation = Arc::clone(head.curr);
        utils::advance(head);

        left = Box::new(Expression::Binary {
            left,
            operation,
            right: unary(head)?,
        });
    }

    Ok(left)
}

pub fn unary(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    match head.curr.ttype {
        TokenType::Not | TokenType::Minus => Ok(Box::new(Expression::Unary {
            operation: Arc::clone(head.curr),
            value: {
                utils::advance(head);
                unary(head)?
            },
        })),
        _ => call(head),
    }
}

pub fn call(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut expr: Box<Expression> = get(head)?;

    while matches!(head.curr.ttype, TokenType::LeftParen) {
        let mut args: Vec<Expression> = vec![];
        utils::advance(head);

        if !matches!(head.curr.ttype, TokenType::RightParen) {
            args.push(*parse_expression(head)?);

            while !matches!(head.curr.ttype, TokenType::Comma) {
                utils::advance(head);
                args.push(*parse_expression(head)?);
            }
        }

        utils::require_token_type(head.curr, TokenType::RightParen)?;
        expr = Box::new(Expression::FnCall {
            fn_identifier: expr,
            args,
        });
    }

    Ok(expr)
}

pub fn get(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut expr: Box<Expression> = primary(head)?;

    while matches!(
        head.curr.ttype,
        TokenType::Dot | TokenType::StaticScopeGetter
    ) {
        utils::advance(head);

        utils::require_token_type(head.curr, TokenType::Identifier)?;
        let property = Arc::clone(head.curr);
        utils::advance(head);

        expr = Box::new(Expression::GetField {
            from: expr,
            get: property,
        });
    }

    return Ok(expr);
}

pub fn primary(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    match head.curr.ttype {
        TokenType::Identifier | TokenType::DontCare => {
            utils::advance(head);
            Ok(Box::new(Expression::Name {
                name: Arc::clone(head.prev),
            }))
        }
        TokenType::Integer
        | TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => {
            utils::advance(head);
            Ok(Box::new(Expression::Literal {
                literal: Arc::clone(head.prev),
            }))
        }
        TokenType::BitAnd => {
            utils::advance(head);

            let next_tk = Arc::clone(head.curr);
            let of: Box<Expression> = primary(head)?;
            match *of {
                Expression::Name { .. } => Ok(Box::new(Expression::AddressOf { of })),
                _ => Err(ParseError::InvalidAddressOfValue { at: next_tk }),
            }
        }
        TokenType::LeftSquare => {
            utils::advance(head);
            let mut values: Vec<Box<Expression>> = vec![];

            while !matches!(head.curr.ttype, TokenType::RightSquare) {
                values.push(parse_expression(head)?);

                match head.curr.ttype {
                    TokenType::RightSquare => break,
                    TokenType::Comma => {
                        utils::advance(head);
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            line: head.curr.line,
                            col: head.curr.column,
                            found: head.curr.ttype.clone(),
                            expected: TokenType::RightSquare,
                            msg: None,
                        });
                    }
                }
            }

            utils::require_token_type(head.curr, TokenType::RightSquare)?;
            utils::advance(head);

            Ok(Box::new(Expression::ArrayLiteral { values }))
        }
        TokenType::LeftParen => {
            utils::advance(head);
            let nested: Box<Expression> = parse_expression(head)?;

            utils::require_token_type(head.curr, TokenType::RightParen)?;
            utils::advance(head);
            Ok(Box::new(Expression::Nested { nested }))
        }
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(head.curr),
        }),
    }
}

pub fn match_pattern_expression(head: &mut ParserHead) -> Result<Expression, ParseError> {
    match head.curr.ttype {
        TokenType::Identifier | TokenType::DontCare => {
            utils::advance(head);
            Ok(Expression::Name {
                name: Arc::clone(head.prev),
            })
        }
        TokenType::Integer => {
            utils::advance(head);
            match head.curr.ttype {
                TokenType::SequenceUpTo | TokenType::SequenceUpToIncluding => {
                    let start: isize = head.prev.lexeme.parse().unwrap();
                    utils::advance(head);

                    utils::require_token_type(&head.curr, TokenType::Integer)?;
                    let end: isize = head.curr.lexeme.parse::<isize>().unwrap()
                        + if matches!(head.prev.ttype, TokenType::SequenceUpTo) {
                            -1
                        } else {
                            0
                        };

                    utils::advance(head);

                    Ok(Expression::Sequence { start, end })
                }
                _ => Ok(Expression::Literal {
                    literal: Arc::clone(head.prev),
                }),
            }
        }
        TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => {
            utils::advance(head);
            Ok(Expression::Literal {
                literal: Arc::clone(head.prev),
            })
        }
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(head.curr),
        }),
    }
}
