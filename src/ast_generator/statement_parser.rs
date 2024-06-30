use std::sync::Arc;

use crate::{ast_generator::expression_parser, tokens::{error::ParseError, token::Token, token_type::TokenType}};

use super::{
    ast::{
        expressions::Expression, scopebound_statements::ScopeBoundStatement, variables::Variable,
    },
    expression_parser::{or_expression, parse_expression},
    parser_head::ParserHead,
    utils,
};

pub fn parse_scope_block(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let mut body: Vec<ScopeBoundStatement> = vec![];

    while !matches!(head.curr.ttype, TokenType::RightBrace | TokenType::Eof) {
        match parse_scopebound_statement(head) {
            Ok(stmt) => body.push(stmt),
            Err(e) => {
                utils::print_error(&head.curr.found_in, &head.prev.lexeme, e);

                while !matches!(head.curr.ttype, TokenType::Semicolon | TokenType::Eof) {
                    utils::advance(head);
                }

                // Consumes the `;`
                utils::advance(head);
            }
        }
    }

    utils::require_token_type(&head.curr, TokenType::RightBrace)?;
    utils::advance(head);

    Ok(ScopeBoundStatement::Scope(body))
}

pub fn parse_scopebound_statement(
    head: &mut ParserHead,
) -> Result<ScopeBoundStatement, ParseError> {
    match head.curr.ttype {
        TokenType::If => parse_conditional(head),
        TokenType::Match => parse_match(head),
        TokenType::While => parse_while_loop(head),
        TokenType::Loop => parse_loop(head),
        TokenType::For => parse_for(head),
        TokenType::LeftBrace => {
            utils::advance(head);
            parse_scope_block(head)
        }
        TokenType::Return => {
            utils::advance(head);

            let expr: Expression = or_expression(head)?;

            utils::require_token_type(head.curr, TokenType::Semicolon)?;
            utils::advance(head);

            Ok(ScopeBoundStatement::Return(expr))
        }
        TokenType::Let => {
            utils::advance(head);
            Ok(parse_variable_declaration(head)?)
        }
        TokenType::Break => {
            utils::advance(head);

            utils::require_token_type(head.curr, TokenType::Semicolon)?;
            utils::advance(head);

            Ok(ScopeBoundStatement::Break)
        }
        TokenType::Continue => {
            utils::advance(head);

            utils::require_token_type(head.curr, TokenType::Semicolon)?;
            utils::advance(head);

            Ok(ScopeBoundStatement::Continue)
        }
        _ => parse_expression_statement(head)
    }
}

fn parse_match(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);

    let on: Expression = parse_expression(head)?;

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    let mut cases: Vec<(Expression, ScopeBoundStatement)> = vec![];
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        let case: Expression = expression_parser::match_pattern_expression(head)?;

        utils::require_token_type(&head.curr, TokenType::Arrow)?;
        utils::advance(head);

        utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
        utils::advance(head);

        let value: ScopeBoundStatement = parse_scope_block(head)?;
        cases.push((case, value));
    }

    Ok(ScopeBoundStatement::Match { on, cases })
}

fn parse_expression_statement(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let expr: Expression = parse_expression(head)?;

    utils::require_token_type(head.curr, TokenType::Semicolon)?;
    utils::advance(head);

    Ok(ScopeBoundStatement::Expression(expr))
}

fn parse_for(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    // for -> (
    utils::advance(head);

    utils::require_token_type(&head.curr, TokenType::LeftParen)?;
    utils::advance(head);

    let initialization: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Let => Some(Box::new(parse_variable_declaration(head)?)),
        TokenType::Semicolon => {
            utils::advance(head);
            None
        },
        _ => Some(Box::new(parse_expression_statement(head)?))
    };

    let condition: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            None
        },
        _ => Some(Box::new(parse_expression_statement(head)?))
    };

    let increment: Option<Expression> = match head.curr.ttype {
        TokenType::RightParen => None,
        _ => Some(parse_expression(head)?)
    };

    utils::require_token_type(&head.curr, TokenType::RightParen)?;
    utils::advance(head);

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);
    let body: Box<ScopeBoundStatement> = Box::new(parse_scope_block(head)?);

    Ok(ScopeBoundStatement::For { initialization, condition, increment, body })
}

pub fn parse_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition = Expression::Literal {
        literal: Arc::new(Token::new(
            head.curr.line,
            head.curr.column,
            TokenType::True,
            String::from("true"),
            head.curr.found_in.clone(),
        )),
    };

    match head.curr.ttype {
        TokenType::LeftBrace => {
            utils::advance(head);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(Box::new(parse_scope_block(head)?)),
            })
        }
        TokenType::Semicolon => {
            utils::advance(head);
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            body: Arc::clone(head.curr)
        }),
    }
}

pub fn parse_while_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition: Expression = parse_expression(head)?;

    match head.curr.ttype {
        TokenType::LeftBrace => {
            utils::advance(head);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(Box::new(parse_scope_block(head)?)),
            })
        }
        TokenType::Semicolon => {
            utils::advance(head);
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            body: Arc::clone(head.curr)
        }),
    }
}

pub fn parse_conditional(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let parse_branch = |head: &mut ParserHead| -> Result<ScopeBoundStatement, ParseError> {
        utils::require_token_type(head.curr, TokenType::LeftBrace)?;
        utils::advance(head);

        parse_scope_block(head)
    };

    utils::advance(head);
    let condition: Expression = or_expression(head)?;
    let true_branch: ScopeBoundStatement = parse_branch(head)?;

    match head.curr.ttype {
        TokenType::Else => {
            utils::advance(head);

            Ok(ScopeBoundStatement::Conditional {
                condition,
                true_branch: Box::new(true_branch),
                false_branch: Some(match head.curr.ttype {
                    TokenType::If => Box::new(parse_conditional(head)?),
                    _ => Box::new(parse_branch(head)?),
                }),
            })
        }
        _ => Ok(ScopeBoundStatement::Conditional {
            condition,
            true_branch: Box::new(true_branch),
            false_branch: None,
        }),
    }
}

pub fn parse_variable_declaration(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let variable_name = Arc::clone(head.curr);
    utils::advance(head);

    let variable: ScopeBoundStatement = match head.curr.ttype {
        TokenType::Colon => {
            utils::advance(head);
            let datatype = utils::parse_datatype(head)?;

            match utils::require_token_type(head.curr, TokenType::Equal) {
                Ok(_) => {
                    utils::advance(head);

                    ScopeBoundStatement::VariableDeclaration(Variable::new(
                        variable_name,
                        Some(datatype),
                        or_expression(head)?,
                    ))
                }
                Err(_) => return Err(ParseError::InvalidVariableDeclaration {
                    line: head.curr.line,
                    column: head.curr.column,
                }),
            }
        }
        TokenType::DynamicDefinition => {
            utils::advance(head);
            ScopeBoundStatement::VariableDeclaration(Variable::new(
                variable_name,
                None,
                parse_expression(head)?,
            ))
        }
        _ => {
            return Err(ParseError::InvalidVariableDeclaration {
                line: head.curr.line,
                column: head.curr.column,
            })
        }
    };

    utils::require_token_type(head.curr, TokenType::Semicolon)?;
    utils::advance(head);

    Ok(variable)
}
