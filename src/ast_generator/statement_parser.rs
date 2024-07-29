use std::collections::HashMap;

use crate::{
    ast_generator::expression_parser,
    tokens::{error::ParseError, token_type::TokenType},
};

use super::{
    ast::{
        expressions::Expression, scopebound_statements::ScopeBoundStatement, variables::Variable,
    },
    expression_parser::{or_expression, parse_expression},
    parser_head::ParserHead,
    utils,
};

pub fn parse_scopebound_statement(
    head: &mut ParserHead,
) -> Result<ScopeBoundStatement, ParseError> {
    match head.curr.ttype {
        TokenType::If => parse_conditional(head),
        TokenType::Match => parse_match(head),
        TokenType::While => parse_while_loop(head),
        TokenType::Loop => parse_loop(head),
        TokenType::For => parse_for(head),
        TokenType::Let => parse_variable_declaration(head),
        TokenType::LeftBrace => {
            head.advance();
            Ok(ScopeBoundStatement::Scope(parse_scope_block(head)?))
        }
        TokenType::Defer => {
            head.advance();

            let stmt: ScopeBoundStatement = parse_scopebound_statement(head)?;

            if head.prev.ttype != TokenType::Semicolon {
                head.require_current_is(TokenType::Semicolon)?;
                head.advance();
            }

            Ok(ScopeBoundStatement::Defer(Box::new(stmt)))
        }
        TokenType::Return => {
            head.advance();

            let expr: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
                TokenType::Semicolon => None,
                _ => Some(Box::new(parse_assignable_stmt(head)?)),
            };

            head.require_current_is(TokenType::Semicolon)?;
            head.advance();

            Ok(ScopeBoundStatement::Return(expr))
        }
        TokenType::Break => {
            head.advance();

            head.require_current_is(TokenType::Semicolon)?;
            head.advance();

            Ok(ScopeBoundStatement::Break)
        }
        TokenType::Continue => {
            head.advance();

            head.require_current_is(TokenType::Semicolon)?;
            head.advance();

            Ok(ScopeBoundStatement::Continue)
        }
        _ => parse_expression_statement(head),
    }
}

pub fn parse_scope_block(head: &mut ParserHead) -> Result<Vec<ScopeBoundStatement>, ParseError> {
    let mut body: Vec<ScopeBoundStatement> = vec![];

    while !matches!(head.curr.ttype, TokenType::RightBrace | TokenType::Eof) {
        match parse_scopebound_statement(head) {
            Ok(stmt) => body.push(stmt),
            Err(e) => {
                utils::print_error(&head.curr.found_in, &head.prev.lexeme, e);

                while !matches!(
                    head.curr.ttype,
                    TokenType::Semicolon | TokenType::RightBrace | TokenType::Eof
                ) {
                    head.advance();
                }

                if matches!(head.curr.ttype, TokenType::Semicolon) {
                    head.advance();
                }
            }
        }
    }

    head.require_current_is(TokenType::RightBrace)?;
    head.advance();

    Ok(body)
}

fn parse_match(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    head.advance();

    let on: Expression = *parse_expression(head)?;

    head.require_current_is(TokenType::LeftBrace)?;
    head.advance();

    let mut cases: HashMap<Expression, Vec<ScopeBoundStatement>> = HashMap::new();
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        let case: Expression = expression_parser::match_pattern_expression(head)?;

        head.require_current_is(TokenType::Arrow)?;
        head.advance();

        head.require_current_is(TokenType::LeftBrace)?;
        head.advance();

        let value: Vec<ScopeBoundStatement> = parse_scope_block(head)?;
        cases.insert(case, value);

        match head.curr.ttype {
            TokenType::Comma => {
                head.advance();
            }
            TokenType::RightBrace => {
                break;
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: std::mem::take(&mut head.curr),
                    expected: TokenType::RightParen,
                    msg: None,
                });
            }
        }
    }

    head.advance();
    Ok(ScopeBoundStatement::Match { on, cases })
}

fn parse_expression_statement(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let expr: Expression = *parse_expression(head)?;

    match head.curr.ttype {
        TokenType::Semicolon => {
            head.advance();
            Ok(ScopeBoundStatement::Expression(expr))
        }
        TokenType::RightBrace => {
            Ok(ScopeBoundStatement::ImplicitReturn(expr))
        }
        _ => Err(ParseError::InvalidExpression {
            token: std::mem::take(&mut head.curr),
        }),
    }
}

fn parse_for(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let parse_value = |head: &mut ParserHead| -> Result<Expression, ParseError> {
        let res: Expression = *parse_expression(head)?;

        head.require_current_is(TokenType::Semicolon)?;
        head.advance();

        Ok(res)
    };

    // for -> (
    head.advance();

    head.require_current_is(TokenType::LeftParen)?;
    head.advance();

    let initialization: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Let => Some(Box::new(parse_variable_declaration(head)?)),
        TokenType::Semicolon => {
            head.advance();
            None
        }
        _ => Some(Box::new(ScopeBoundStatement::Expression(parse_value(
            head,
        )?))),
    };

    let condition: Option<Expression> = match head.curr.ttype {
        TokenType::Semicolon => {
            head.advance();
            None
        }
        _ => Some(parse_value(head)?),
    };

    let increment: Option<Expression> = match head.curr.ttype {
        TokenType::RightParen => None,
        _ => Some(*parse_expression(head)?),
    };

    head.require_current_is(TokenType::RightParen)?;
    head.advance();

    let body: Option<Vec<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Semicolon => {
            head.advance();
            None
        }
        TokenType::LeftBrace => {
            head.advance();
            Some(parse_scope_block(head)?)
        }
        _ => {
            return Err(ParseError::LoopBodyNotFound {
                body: std::mem::take(&mut head.curr),
            })
        }
    };

    Ok(ScopeBoundStatement::For {
        initialization,
        condition,
        increment,
        body,
    })
}

pub fn parse_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    head.advance();

    match head.curr.ttype {
        TokenType::LeftBrace => {
            head.advance();
            Ok(ScopeBoundStatement::Loop(Some(parse_scope_block(head)?)))
        }
        TokenType::Semicolon => {
            head.advance();
            Ok(ScopeBoundStatement::Loop(None))
        }
        _ => Err(ParseError::LoopBodyNotFound {
            body: std::mem::take(&mut head.curr),
        }),
    }
}

pub fn parse_while_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    head.advance();
    let condition: Expression = *parse_expression(head)?;

    match head.curr.ttype {
        TokenType::LeftBrace => {
            head.advance();
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(parse_scope_block(head)?),
            })
        }
        TokenType::Semicolon => {
            head.advance();
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            body: std::mem::take(&mut head.curr),
        }),
    }
}

pub fn parse_conditional(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    head.advance();
    let condition: Expression = *or_expression(head)?;
    let true_branch: Vec<ScopeBoundStatement> = parse_conditional_branch(head)?;

    match head.curr.ttype {
        TokenType::Else => {
            head.advance();

            Ok(ScopeBoundStatement::Conditional {
                condition,
                true_branch,
                false_branch: Some(match head.curr.ttype {
                    TokenType::If => vec![parse_conditional(head)?],
                    _ => parse_conditional_branch(head)?,
                }),
            })
        }
        _ => Ok(ScopeBoundStatement::Conditional {
            condition,
            true_branch,
            false_branch: None,
        }),
    }
}

fn parse_conditional_branch(head: &mut ParserHead) -> Result<Vec<ScopeBoundStatement>, ParseError> {
    head.require_current_is(TokenType::LeftBrace)?;
    head.advance();

    parse_scope_block(head)
}

fn parse_assignable_stmt(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    match head.curr.ttype {
        TokenType::If => parse_conditional(head),
        TokenType::Match => parse_match(head),
        TokenType::LeftBrace => {
            head.advance();
            Ok(ScopeBoundStatement::Scope(parse_scope_block(head)?))
        }
        TokenType::While
        | TokenType::Loop
        | TokenType::For
        | TokenType::Return
        | TokenType::Let
        | TokenType::Break
        | TokenType::Continue => Err(ParseError::InvalidVariableAssignment {
            value: std::mem::take(&mut head.curr),
        }),
        _ => Ok(ScopeBoundStatement::Expression(
            *expression_parser::parse_expression(head)?,
        )),
    }
}

pub fn parse_variable_declaration(
    head: &mut ParserHead,
) -> Result<ScopeBoundStatement, ParseError> {
    // let -> var_name
    head.advance();

    let variable_name = std::mem::take(&mut head.curr);
    head.advance();

    let variable: ScopeBoundStatement = match head.curr.ttype {
        TokenType::Colon => {
            head.advance();
            let datatype = head.parse_datatype()?;

            match head.require_current_is(TokenType::Equal) {
                Ok(_) => {
                    head.advance();

                    ScopeBoundStatement::VariableDeclaration(Variable::new(
                        variable_name,
                        Some(datatype),
                        Box::new(parse_assignable_stmt(head)?),
                    ))
                }
                Err(_) => {
                    return Err(ParseError::InvalidVariableDeclaration {
                        token: std::mem::take(&mut head.curr),
                    })
                }
            }
        }
        TokenType::DynamicDefinition => {
            head.advance();
            ScopeBoundStatement::VariableDeclaration(Variable::new(
                variable_name,
                None,
                Box::new(parse_assignable_stmt(head)?),
            ))
        }
        _ => {
            return Err(ParseError::InvalidVariableDeclaration {
                token: std::mem::take(&mut head.curr),
            })
        }
    };

    head.require_current_is(TokenType::Semicolon)?;
    head.advance();

    Ok(variable)
}
