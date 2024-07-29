use crate::tokens::{error::ParseError, token_type::TokenType};

use super::{ast::expressions::Expression, parser_head::ParserHead};

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
            let operation = std::mem::take(&mut head.curr);
            head.advance();
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
        head.advance();

        left = Box::new(Expression::Binary {
            left,
            operation: std::mem::take(&mut head.prev),
            right: and_expression(head)?,
        });
    }

    Ok(left)
}

pub fn and_expression(head: &mut ParserHead) -> Result<Box<Expression>, ParseError> {
    let mut left: Box<Expression> = equality_check(head)?;

    while matches!(head.curr.ttype, TokenType::And | TokenType::BitAnd) {
        let operation = std::mem::take(&mut head.curr);
        head.advance();

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
        head.advance();
        left = Box::new(Expression::Binary {
            left,
            operation: std::mem::take(&mut head.prev),
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
        let operation = std::mem::take(&mut head.curr);
        head.advance();

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
        let operation = std::mem::take(&mut head.curr);
        head.advance();

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
        let operation = std::mem::take(&mut head.curr);
        head.advance();

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
            operation: std::mem::take(&mut head.curr),
            value: {
                head.advance();
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
        head.advance();

        if !matches!(head.curr.ttype, TokenType::RightParen) {
            args.push(*parse_expression(head)?);

            while !matches!(head.curr.ttype, TokenType::Comma) {
                head.advance();
                args.push(*parse_expression(head)?);
            }
        }

        head.require_current_is(TokenType::RightParen)?;
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
        head.advance();

        head.require_current_is(TokenType::Identifier)?;
        let property = std::mem::take(&mut head.curr);
        head.advance();

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
            head.advance();
            Ok(Box::new(Expression::Name {
                name: std::mem::take(&mut head.prev),
            }))
        }
        TokenType::Integer
        | TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => {
            head.advance();
            Ok(Box::new(Expression::Literal {
                literal: std::mem::take(&mut head.prev),
            }))
        }
        TokenType::BitAnd => {
            head.advance();

            let next_tk = std::mem::take(&mut head.curr);
            let of: Box<Expression> = primary(head)?;
            match *of {
                Expression::Name { .. } => Ok(Box::new(Expression::AddressOf { of })),
                _ => Err(ParseError::InvalidAddressOfValue { at: next_tk }),
            }
        }
        TokenType::LeftSquare => {
            head.advance();
            let mut values: Vec<Box<Expression>> = vec![];

            while !matches!(head.curr.ttype, TokenType::RightSquare) {
                values.push(parse_expression(head)?);

                match head.curr.ttype {
                    TokenType::RightSquare => break,
                    TokenType::Comma => {
                        head.advance();
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: std::mem::take(&mut head.curr),
                            expected: TokenType::RightSquare,
                            msg: None,
                        });
                    }
                }
            }

            head.require_current_is(TokenType::RightSquare)?;
            head.advance();

            Ok(Box::new(Expression::ArrayLiteral { values }))
        }
        TokenType::LeftParen => {
            head.advance();
            let nested: Box<Expression> = parse_expression(head)?;

            head.require_current_is(TokenType::RightParen)?;
            head.advance();
            Ok(Box::new(Expression::Nested { nested }))
        }
        _ => Err(ParseError::InvalidExpression {
            token: std::mem::take(&mut head.curr),
        }),
    }
}

pub fn match_pattern_expression(head: &mut ParserHead) -> Result<Expression, ParseError> {
    match head.curr.ttype {
        TokenType::Identifier | TokenType::DontCare => {
            head.advance();
            Ok(Expression::Name {
                name: std::mem::take(&mut head.prev),
            })
        }
        TokenType::Integer => {
            head.advance();
            match head.curr.ttype {
                TokenType::SequenceUpTo | TokenType::SequenceUpToIncluding => {
                    let start: isize = head.prev.lexeme.parse().unwrap();
                    head.advance();

                    head.require_current_is(TokenType::Integer)?;
                    let end: isize = head.curr.lexeme.parse::<isize>().unwrap()
                        + if matches!(head.prev.ttype, TokenType::SequenceUpTo) {
                            -1
                        } else {
                            0
                        };

                    head.advance();

                    Ok(Expression::Sequence { start, end })
                }
                _ => Ok(Expression::Literal {
                    literal: std::mem::take(&mut head.prev),
                }),
            }
        }
        TokenType::Double
        | TokenType::String
        | TokenType::True
        | TokenType::False
        | TokenType::Nil => {
            head.advance();
            Ok(Expression::Literal {
                literal: std::mem::take(&mut head.prev),
            })
        }
        _ => Err(ParseError::InvalidExpression {
            token: std::mem::take(&mut head.curr),
        }),
    }
}
