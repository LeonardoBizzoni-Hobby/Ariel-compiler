use std::{collections::HashMap, sync::Arc};

use crate::{
    ast_generator::expression_parser,
    tokens::{error::ParseError, token::Token, token_type::TokenType},
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
        TokenType::LeftBrace => {
            utils::advance(head);
            parse_scope_block(head)
        }
        TokenType::Return => {
            utils::advance(head);

            let expr: Box<Expression> = or_expression(head)?;

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
        _ => parse_expression_statement(head),
    }
}

pub fn parse_scope_block(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
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

    Ok(ScopeBoundStatement::Scope(body))
}

fn parse_match(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);

    let on: Box<Expression> = parse_expression(head)?;

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    let mut cases: HashMap<Expression, ScopeBoundStatement> = HashMap::new();
    while !matches!(head.curr.ttype, TokenType::RightBrace) {
        let case: Expression = expression_parser::match_pattern_expression(head)?;

        utils::require_token_type(&head.curr, TokenType::Arrow)?;
        utils::advance(head);

        utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
        utils::advance(head);

        let value: ScopeBoundStatement = parse_scope_block(head)?;
        cases.insert(case, value);
    }

    Ok(ScopeBoundStatement::Match { on, cases })
}

fn parse_expression_statement(
    head: &mut ParserHead,
) -> Result<ScopeBoundStatement, ParseError> {
    let expr: Box<Expression> = parse_expression(head)?;

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
        }
        _ => Some(Box::new(parse_expression_statement(head)?)),
    };

    let condition: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            None
        }
        _ => Some(Box::new(parse_expression_statement(head)?)),
    };

    let increment: Option<Box<Expression>> = match head.curr.ttype {
        TokenType::RightParen => None,
        _ => Some(parse_expression(head)?),
    };

    utils::require_token_type(&head.curr, TokenType::RightParen)?;
    utils::advance(head);

    utils::require_token_type(&head.curr, TokenType::LeftBrace)?;
    utils::advance(head);
    let body: Box<ScopeBoundStatement> = Box::new(parse_scope_block(head)?);

    Ok(ScopeBoundStatement::For {
        initialization,
        condition,
        increment,
        body,
    })
}

pub fn parse_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition = Box::new(Expression::Literal {
        literal: Arc::new(Token::new(
            head.curr.line,
            head.curr.column,
            TokenType::True,
            String::from("true"),
            head.curr.found_in.clone(),
        )),
    });

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
            body: Arc::clone(head.curr),
        }),
    }
}

pub fn parse_while_loop(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition: Box<Expression> = parse_expression(head)?;

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
            body: Arc::clone(head.curr),
        }),
    }
}

pub fn parse_conditional(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::advance(head);
    let condition: Box<Expression> = or_expression(head)?;
    let true_branch: Box<ScopeBoundStatement> = Box::new(parse_conditional_branch(head)?);

    match head.curr.ttype {
        TokenType::Else => {
            utils::advance(head);

            Ok(ScopeBoundStatement::Conditional {
                condition,
                true_branch,
                false_branch: Some(match head.curr.ttype {
                    TokenType::If => Box::new(parse_conditional(head)?),
                    _ => Box::new(parse_conditional_branch(head)?),
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

fn parse_conditional_branch(head: &mut ParserHead) -> Result<ScopeBoundStatement, ParseError> {
    utils::require_token_type(head.curr, TokenType::LeftBrace)?;
    utils::advance(head);

    parse_scope_block(head)
}

pub fn parse_variable_declaration(
    head: &mut ParserHead,
) -> Result<ScopeBoundStatement, ParseError> {
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

#[allow(unused_imports)]
#[allow(dead_code)]
mod tests {
    use crate::{
        test_util::{create_test_file, delete_test_file},
        tokens::{source::SourceFile, tokenizer},
    };

    use super::*;
    use std::{fs::File, io::Write, ops::Deref};

    fn parse(file_name: &str, content: &str) -> Result<ScopeBoundStatement, ParseError> {
        create_test_file(file_name, content);
        let mut file = SourceFile::new(file_name).unwrap();

        let mut curr = tokenizer::get_token(&mut file);
        let mut prev = Arc::new(Token::empty());
        let mut head = ParserHead::new(&mut curr, &mut prev, &mut file);

        delete_test_file(file_name);
        parse_scopebound_statement(&mut head)
    }

    mod conditional {
        use super::*;

        #[test]
        fn valid_if_stmt() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "valid_if",
                "if 23 == 23 { return 42; } else { return -42; }",
            );
            assert!(found.is_ok());

            match found.ok().unwrap() {
                ScopeBoundStatement::Conditional {
                    condition,
                    true_branch: _,
                    false_branch: _,
                } => {
                    let expected_left = Token {
                        line: 1,
                        column: 3,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_if".to_owned(),
                    };
                    let expected_op = Token {
                        line: 1,
                        column: 6,
                        ttype: TokenType::EqualEqual,
                        lexeme: "==".to_owned(),
                        found_in: "valid_if".to_owned(),
                    };
                    let expected_right = Token {
                        line: 1,
                        column: 9,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_if".to_owned(),
                    };

                    match *condition {
                        Expression::Binary {
                            left,
                            operation,
                            right,
                        } => {
                            match *left {
                                Expression::Literal { literal } => {
                                    assert_eq!(expected_left, *literal);
                                }
                                _ => panic!(),
                            }
                            match *right {
                                Expression::Literal { literal } => {
                                    assert_eq!(expected_right, *literal);
                                }
                                _ => panic!(),
                            }

                            assert_eq!(expected_op, *operation);
                        }
                        _ => panic!(),
                    }
                }
                _ => panic!(),
            }
        }
    }

    mod scope {
        use super::*;

        #[test]
        fn scope_not_closed() {
            let found = parse("invalid_scope", "{ 3 + 4;");
            assert!(found.is_err());

            match found.err().unwrap() {
                ParseError::UnexpectedToken {
                    line,
                    col,
                    found,
                    expected,
                    ..
                } => {
                    assert_eq!(1, line);
                    assert_eq!(8, col);
                    assert_eq!(TokenType::Eof, found);
                    assert_eq!(TokenType::RightBrace, expected);
                }
                _ => panic!(),
            }
        }

        #[test]
        fn invalid_stmt_in_invalid_subscope() {
            let found = parse("invalid_stmt_in_invalid_subscope", "{ {3 + 4 }");
            assert!(found.is_err());

            match found.err().unwrap() {
                ParseError::UnexpectedToken {
                    line,
                    col,
                    found,
                    expected,
                    ..
                } => {
                    assert_eq!(1, line);
                    assert_eq!(10, col);
                    assert_eq!(TokenType::Eof, found);
                    assert_eq!(TokenType::RightBrace, expected);
                }
                _ => panic!(),
            }
        }

        #[test]
        fn invalid_stmt_in_scope() {
            let found = parse("invalid_stmt_in_scope", "{ {3 + 4} }");
            assert!(found.is_ok());

            if let ScopeBoundStatement::Scope(body) = found.ok().unwrap() {
                assert!(!body.is_empty());
                assert!(body.get(0).is_some());

                if let ScopeBoundStatement::Scope(inner) = &body[0] {
                    assert!(inner.is_empty());
                    return;
                }
            }

            panic!()
        }

        #[test]
        fn valid_scope() {
            let found = parse("valid_scope", "{ { 3 + 4; } }");
            assert!(found.is_ok());

            if let ScopeBoundStatement::Scope(body) = found.ok().unwrap() {
                assert!(!body.is_empty());
                assert!(body.get(0).is_some());

                if let ScopeBoundStatement::Scope(inner_body) = &body[0] {
                    assert!(!inner_body.is_empty());
                    assert!(inner_body.get(0).is_some());

                    if let ScopeBoundStatement::Expression(expr) = &inner_body[0] {
                        if let Expression::Binary {
                            left,
                            operation,
                            right,
                        } = expr.as_ref()
                        {
                            if let (
                                Expression::Literal { literal: left_lit },
                                Expression::Literal { literal: right_lit },
                            ) = (left.as_ref(), right.as_ref())
                            {
                                assert_eq!("3", left_lit.lexeme);
                                assert_eq!(TokenType::Plus, operation.ttype);
                                assert_eq!("4", right_lit.lexeme);
                                return;
                            }
                        }
                    }
                }
            }

            panic!()
        }
    }

    mod loop_flow_modifiers {
        use super::*;

        #[test]
        fn invalid_continue() {
            let found = parse("invalid_continue", "continue");
            assert!(found.is_err());

            match found.err().unwrap() {
                ParseError::UnexpectedToken {
                    line,
                    col,
                    found,
                    expected,
                    ..
                } => {
                    assert_eq!(1, line);
                    assert_eq!(8, col);
                    assert_eq!(TokenType::Eof, found);
                    assert_eq!(TokenType::Semicolon, expected);
                }
                _ => panic!(),
            }
        }

        #[test]
        fn valid_continue() {
            let found = parse("valid_continue", "continue;");
            assert!(found.is_ok());

            match found.ok().unwrap() {
                ScopeBoundStatement::Continue => {}
                _ => panic!(),
            }
        }

        #[test]
        fn invalid_break() {
            let found = parse("invalid_break", "break");
            assert!(found.is_err());

            if let ParseError::UnexpectedToken {
                line,
                col,
                found,
                expected,
                ..
            } = found.err().unwrap()
            {
                assert_eq!(1, line);
                assert_eq!(5, col);
                assert_eq!(TokenType::Eof, found);
                assert_eq!(TokenType::Semicolon, expected);
                return;
            }

            panic!()
        }

        #[test]
        fn valid_break() {
            let found = parse("valid_break", "break;");
            assert!(found.is_ok());

            match found.ok().unwrap() {
                ScopeBoundStatement::Break => {}
                _ => panic!(),
            }
        }
    }
}
