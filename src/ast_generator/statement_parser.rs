use std::{collections::HashMap, sync::Arc};

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
            utils::advance(head);
            Ok(ScopeBoundStatement::Scope(parse_scope_block(head)?))
        }
        TokenType::Defer => {
            utils::advance(head);

            let stmt: ScopeBoundStatement = parse_scopebound_statement(head)?;

            if head.prev.ttype != TokenType::Semicolon {
                utils::require_token_type(&head.curr, TokenType::Semicolon)?;
                utils::advance(head);
            }

            Ok(ScopeBoundStatement::Defer(Box::new(stmt)))
        }
        TokenType::Return => {
            utils::advance(head);

            let expr: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
                TokenType::Semicolon => None,
                _ => Some(Box::new(parse_assignable_stmt(head)?)),
            };

            utils::require_token_type(head.curr, TokenType::Semicolon)?;
            utils::advance(head);

            Ok(ScopeBoundStatement::Return(expr))
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
                    utils::advance(head);
                }

                if matches!(head.curr.ttype, TokenType::Semicolon) {
                    utils::advance(head);
                }
            }
        }
    }

    utils::require_token_type(&head.curr, TokenType::RightBrace)?;
    utils::advance(head);

    Ok(body)
}

fn parse_match(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);

    let on: Expression = *parse_expression(head)?;

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    let mut cases: HashMap<Expression, Vec<ScopeBoundStatement>> = HashMap::new();
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        let case: Expression = expression_parser::match_pattern_expression(head)?;

        utils::require_token_type(&head.curr, TokenType::Arrow)?;
        utils::advance(head);

        utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
        utils::advance(head);

        let value: Vec<ScopeBoundStatement> = parse_scope_block(head)?;
        cases.insert(case, value);

        match head.curr.ttype {
            TokenType::Comma => {
                utils::advance(head);
            }
            TokenType::RightBrace => {
                break;
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    line: head.curr.line,
                    col: head.curr.column,
                    found: head.curr.ttype.clone(),
                    expected: TokenType::RightParen,
                    msg: None,
                });
            }
        }
    }

    utils::advance(head);
    Ok(ScopeBoundStatement::Match { on, cases })
}

fn parse_expression_statement(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let expr: Expression = *parse_expression(head)?;

    match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            Ok(ScopeBoundStatement::Expression(expr))
        }
        TokenType::RightBrace => {
            Ok(ScopeBoundStatement::ImplicitReturn(expr))
        }
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(head.curr),
        }),
    }
}

fn parse_for(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let parse_value = |head: &mut ParserHead| -> Result<Expression, ParseError> {
        let res: Expression = *parse_expression(head)?;

        utils::require_token_type(&head.curr, TokenType::Semicolon)?;
        utils::advance(head);

        Ok(res)
    };

    // for -> (
    utils::advance(head);

    utils::require_token_type(&head.curr, TokenType::LeftParen)?;
    utils::advance(head);

    let initialization: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Let => Some(Box::new(parse_variable_declaration(head)?)),
        TokenType::Semicolon => {
            utils::advance(head);
            None
        }
        _ => Some(Box::new(ScopeBoundStatement::Expression(parse_value(
            head,
        )?))),
    };

    let condition: Option<Expression> = match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            None
        }
        _ => Some(parse_value(head)?),
    };

    let increment: Option<Expression> = match head.curr.ttype {
        TokenType::RightParen => None,
        _ => Some(*parse_expression(head)?),
    };

    utils::require_token_type(&head.curr, TokenType::RightParen)?;
    utils::advance(head);

    let body: Option<Vec<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            None
        }
        TokenType::LeftBrace => {
            utils::advance(head);
            Some(parse_scope_block(head)?)
        }
        _ => {
            return Err(ParseError::LoopBodyNotFound {
                body: Arc::clone(head.curr),
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
    utils::advance(head);

    match head.curr.ttype {
        TokenType::LeftBrace => {
            utils::advance(head);
            Ok(ScopeBoundStatement::Loop(Some(parse_scope_block(head)?)))
        }
        TokenType::Semicolon => {
            utils::advance(head);
            Ok(ScopeBoundStatement::Loop(None))
        }
        _ => Err(ParseError::LoopBodyNotFound {
            body: Arc::clone(head.curr),
        }),
    }
}

pub fn parse_while_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition: Expression = *parse_expression(head)?;

    match head.curr.ttype {
        TokenType::LeftBrace => {
            utils::advance(head);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(parse_scope_block(head)?),
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
            body: Arc::clone(head.curr),
        }),
    }
}

pub fn parse_conditional(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition: Expression = *or_expression(head)?;
    let true_branch: Vec<ScopeBoundStatement> = parse_conditional_branch(head)?;

    match head.curr.ttype {
        TokenType::Else => {
            utils::advance(head);

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
    utils::require_token_type(head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    parse_scope_block(head)
}

fn parse_assignable_stmt(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    match head.curr.ttype {
        TokenType::If => parse_conditional(head),
        TokenType::Match => parse_match(head),
        TokenType::LeftBrace => {
            utils::advance(head);
            Ok(ScopeBoundStatement::Scope(parse_scope_block(head)?))
        }
        TokenType::While
        | TokenType::Loop
        | TokenType::For
        | TokenType::Return
        | TokenType::Let
        | TokenType::Break
        | TokenType::Continue => Err(ParseError::InvalidVariableAssignment {
            value: Arc::clone(head.curr),
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
    utils::advance(head);

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
                        Box::new(parse_assignable_stmt(head)?),
                    ))
                }
                Err(_) => {
                    return Err(ParseError::InvalidVariableDeclaration {
                        line: head.curr.line,
                        column: head.curr.column,
                    })
                }
            }
        }
        TokenType::DynamicDefinition => {
            utils::advance(head);
            ScopeBoundStatement::VariableDeclaration(Variable::new(
                variable_name,
                None,
                Box::new(parse_assignable_stmt(head)?),
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
