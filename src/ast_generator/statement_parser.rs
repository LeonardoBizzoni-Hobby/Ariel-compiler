use std::sync::Arc;

use crate::tokens::{error::ParseError, source::Source, token::Token, token_type::TokenType};

use super::{
    ast::{
        expressions::Expression, scopebound_statements::ScopeBoundStatement, variables::Variable,
    },
    expression_parser::{or_expression, parse_expression},
    utils,
};

pub fn parse_scopebound_statement(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    match curr.ttype {
        TokenType::If => parse_conditional(curr, prev, source),
        TokenType::Match => todo!(),
        TokenType::While => parse_while_loop(curr, prev, source),
        TokenType::Loop => parse_loop(curr, prev, source),
        TokenType::For => todo!(),
        TokenType::LeftBrace => {
            utils::advance(curr, prev, source);
            parse_scopebound_statement(curr, prev, source)
        }
        TokenType::Return => {
            utils::advance(curr, prev, source);

            let expr: Expression = or_expression(curr, prev, source)?;

            utils::require_token_type(curr, TokenType::Semicolon)?;
            utils::advance(curr, prev, source);

            Ok(ScopeBoundStatement::Return(expr))
        }
        TokenType::Let => {
            utils::advance(curr, prev, source);

            let variable: Box<Variable> = parse_variable_declaration(curr, prev, source)?;

            utils::require_token_type(curr, TokenType::Semicolon)?;
            utils::advance(curr, prev, source);

            Ok(ScopeBoundStatement::VariableDeclaration(variable))
        }
        TokenType::Break => {
            utils::advance(curr, prev, source);

            utils::require_token_type(curr, TokenType::Semicolon)?;
            utils::advance(curr, prev, source);

            Ok(ScopeBoundStatement::Break)
        }
        TokenType::Continue => {
            utils::advance(curr, prev, source);

            utils::require_token_type(curr, TokenType::Semicolon)?;
            utils::advance(curr, prev, source);

            Ok(ScopeBoundStatement::Continue)
        }
        _ => {
            let expr: Expression = parse_expression(curr, prev, source)?;

            utils::require_token_type(curr, TokenType::Semicolon)?;
            utils::advance(curr, prev, source);

            Ok(ScopeBoundStatement::Expression(expr))
        }
    }
}

pub fn parse_scope_block(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Vec<ScopeBoundStatement>, ParseError> {
    let mut body: Vec<ScopeBoundStatement> = vec![];

    while !matches!(curr.ttype, TokenType::RightBrace | TokenType::Eof) {
        body.push(match parse_scopebound_statement(curr, prev, source) {
            Ok(stmt) => stmt,
            Err(e) => {
                utils::print_error(&curr.found_in, &prev.lexeme, e);

                while !matches!(curr.ttype, TokenType::Semicolon | TokenType::Eof) {
                    utils::advance(curr, prev, source);
                }

                // Consumes the `;`
                utils::advance(curr, prev, source);
                continue;
            }
        })
    }

    utils::require_token_type(&curr, TokenType::RightBrace)?;
    utils::advance(curr, prev, source);

    Ok(body)
}

pub fn parse_loop(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(curr, prev, source);
    let condition = Expression::Literal {
        literal: Arc::new(Token::new(
            curr.line,
            curr.column,
            TokenType::True,
            "true".to_owned(),
            curr.found_in.clone(),
        )),
    };

    match curr.ttype {
        TokenType::LeftBrace => {
            utils::advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(parse_scope_block(curr, prev, source)?),
            })
        }
        TokenType::Semicolon => {
            utils::advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            line: curr.line,
            column: curr.column,
        }),
    }
}

pub fn parse_while_loop(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(curr, prev, source);
    let condition = parse_expression(curr, prev, source)?;

    match curr.ttype {
        TokenType::LeftBrace => {
            utils::advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: Some(parse_scope_block(curr, prev, source)?),
            })
        }
        TokenType::Semicolon => {
            utils::advance(curr, prev, source);
            Ok(ScopeBoundStatement::While {
                condition,
                body: None,
            })
        }
        _ => Err(ParseError::LoopBodyNotFound {
            line: curr.line,
            column: curr.column,
        }),
    }
}

pub fn parse_conditional(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<ScopeBoundStatement, ParseError> {
    let parse_branch = |curr: &mut Arc<Token>,
                        prev: &mut Arc<Token>,
                        source: &mut Source|
     -> Result<Vec<ScopeBoundStatement>, ParseError> {
        utils::require_token_type(curr, TokenType::LeftBrace)?;
        utils::advance(curr, prev, source);

        parse_scope_block(curr, prev, source)
    };

    utils::advance(curr, prev, source);
    let condition: Expression = or_expression(curr, prev, source)?;
    let true_branch = parse_branch(curr, prev, source)?;

    match curr.ttype {
        TokenType::Else => {
            utils::advance(curr, prev, source);

            Ok(ScopeBoundStatement::Conditional {
                condition,
                true_branch,
                false_branch: Some(match curr.ttype {
                    TokenType::If => vec![parse_conditional(curr, prev, source)?],
                    _ => parse_branch(curr, prev, source)?,
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

pub fn parse_variable_declaration(
    curr: &mut Arc<Token>,
    prev: &mut Arc<Token>,
    source: &mut Source,
) -> Result<Box<Variable>, ParseError> {
    let variable_name = Arc::clone(curr);
    utils::advance(curr, prev, source);

    match curr.ttype {
        TokenType::Colon => {
            utils::advance(curr, prev, source);
            let datatype = utils::parse_datatype(curr, prev, source)?;

            match utils::require_token_type(curr, TokenType::Equal) {
                Ok(_) => {
                    utils::advance(curr, prev, source);

                    Ok(Box::new(Variable::new(
                        variable_name,
                        Some(datatype),
                        or_expression(curr, prev, source)?,
                    )))
                }
                Err(_) => Err(ParseError::InvalidVariableDeclaration {
                    line: curr.line,
                    column: curr.column,
                }),
            }
        }
        TokenType::DynamicDefinition => {
            utils::advance(curr, prev, source);
            Ok(Box::new(Variable::new(
                variable_name,
                None,
                parse_expression(curr, prev, source)?,
            )))
        }
        _ => {
            return Err(ParseError::InvalidVariableDeclaration {
                line: curr.line,
                column: curr.column,
            })
        }
    }
}
