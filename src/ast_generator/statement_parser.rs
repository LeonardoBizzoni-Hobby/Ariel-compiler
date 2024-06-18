use std::sync::Arc;

use crate::tokens::{error::ParseError, token::Token, token_type::TokenType};

use super::{
    ast::{
        expressions::Expression, scopebound_statements::ScopeBoundStatement, variables::Variable,
    },
    expression_parser::{or_expression, parse_expression},
    parser_head::ParserHead,
    utils,
};

#[allow(dead_code)]
enum Iterators {
    Variable(Arc<Token>),
    Seq(Arc<Token>, Arc<Token>),
    Array,
}

pub fn parse_scopebound_statement(
    head: &mut ParserHead,
) -> Result<ScopeBoundStatement, ParseError> {
    match head.curr.ttype {
        TokenType::If => parse_conditional(head),
        TokenType::Match => todo!(),
        TokenType::While => parse_while_loop(head),
        TokenType::Loop => parse_loop(head),
        TokenType::For => todo!(),
        TokenType::LeftBrace => {
            utils::advance(head);
            parse_scopebound_statement(head)
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

            let variable: Box<Variable> = parse_variable_declaration(head)?;

            utils::require_token_type(head.curr, TokenType::Semicolon)?;
            utils::advance(head);

            Ok(ScopeBoundStatement::VariableDeclaration(variable))
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
        _ => {
            let expr: Expression = parse_expression(head)?;

            utils::require_token_type(head.curr, TokenType::Semicolon)?;
            utils::advance(head);

            Ok(ScopeBoundStatement::Expression(expr))
        }
    }
}

pub fn parse_scope_block(head: &mut ParserHead) -> Result<Vec<ScopeBoundStatement>, ParseError> {
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

    Ok(body)
}

pub fn parse_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition = Expression::Literal {
        literal: Arc::new(Token::new(
            head.curr.line,
            head.curr.column,
            TokenType::True,
            "true".to_owned(),
            head.curr.found_in.clone(),
        )),
    };

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
            line: head.curr.line,
            column: head.curr.column,
        }),
    }
}

pub fn parse_while_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition = parse_expression(head)?;

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
            line: head.curr.line,
            column: head.curr.column,
        }),
    }
}

pub fn parse_conditional(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    let parse_branch = |head: &mut ParserHead| -> Result<Vec<ScopeBoundStatement>, ParseError> {
        utils::require_token_type(head.curr, TokenType::LeftBrace)?;
        utils::advance(head);

        parse_scope_block(head)
    };

    utils::advance(head);
    let condition: Expression = or_expression(head)?;
    let true_branch = parse_branch(head)?;

    match head.curr.ttype {
        TokenType::Else => {
            utils::advance(head);

            Ok(ScopeBoundStatement::Conditional {
                condition,
                true_branch,
                false_branch: Some(match head.curr.ttype {
                    TokenType::If => vec![parse_conditional(head)?],
                    _ => parse_branch(head)?,
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

pub fn parse_variable_declaration(head: &mut ParserHead) -> Result<Box<Variable>, ParseError> {
    let variable_name = Arc::clone(head.curr);
    utils::advance(head);

    match head.curr.ttype {
        TokenType::Colon => {
            utils::advance(head);
            let datatype = utils::parse_datatype(head)?;

            match utils::require_token_type(head.curr, TokenType::Equal) {
                Ok(_) => {
                    utils::advance(head);

                    Ok(Box::new(Variable::new(
                        variable_name,
                        Some(datatype),
                        or_expression(head)?,
                    )))
                }
                Err(_) => Err(ParseError::InvalidVariableDeclaration {
                    line: head.curr.line,
                    column: head.curr.column,
                }),
            }
        }
        TokenType::DynamicDefinition => {
            utils::advance(head);
            Ok(Box::new(Variable::new(
                variable_name,
                None,
                parse_expression(head)?,
            )))
        }
        _ => {
            return Err(ParseError::InvalidVariableDeclaration {
                line: head.curr.line,
                column: head.curr.column,
            })
        }
    }
}
