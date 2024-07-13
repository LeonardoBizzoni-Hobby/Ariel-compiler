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
    let expr: Box<Expression> = parse_expression(head)?;

    match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            Ok(ScopeBoundStatement::Expression(expr))
        }
        TokenType::RightBrace => Ok(ScopeBoundStatement::ImplicitReturn(expr)),
        _ => Err(ParseError::InvalidExpression {
            token: Arc::clone(head.curr),
        }),
    }
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

    let body: Option<Box<ScopeBoundStatement>> = match head.curr.ttype {
        TokenType::Semicolon => {
            utils::advance(head);
            None
        }
        TokenType::LeftBrace => {
            utils::advance(head);
            Some(Box::new(parse_scope_block(head)?))
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
            Ok(ScopeBoundStatement::Loop(Some(Box::new(
                parse_scope_block(head)?,
            ))))
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
    let parse_value = |head: &mut ParserHead| -> Result<ScopeBoundStatement, ParseError> {
        match head.curr.ttype {
            TokenType::If => parse_conditional(head),
            TokenType::Match => parse_match(head),
            TokenType::LeftBrace => {
                utils::advance(head);
                parse_scope_block(head)
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
                expression_parser::parse_expression(head)?,
            )),
        }
    };

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
                        Box::new(parse_value(head)?),
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
                Box::new(parse_value(head)?),
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
        fn missing_condition() {
            let found: Result<ScopeBoundStatement, ParseError> =
                parse("missing_condition", "if { return 42; }");

            assert!(found.is_err());
            assert_eq!(
                ParseError::InvalidExpression {
                    token: Arc::new(Token {
                        line: 1,
                        column: 3,
                        ttype: TokenType::LeftBrace,
                        lexeme: "{".to_owned(),
                        found_in: "missing_condition".to_owned()
                    })
                },
                found.err().unwrap()
            );
        }

        #[test]
        fn missing_scope() {
            let found: Result<ScopeBoundStatement, ParseError> =
                parse("missing_scope", "if true return 42;");

            assert!(found.is_err());
            assert_eq!(
                ParseError::UnexpectedToken {
                    line: 1,
                    col: 8,
                    found: TokenType::Return,
                    expected: TokenType::LeftBrace,
                    msg: None
                },
                found.err().unwrap()
            );
        }

        #[test]
        fn missing_false_branch() {
            let found: Result<ScopeBoundStatement, ParseError> =
                parse("missing_false_branch", "if true { return 42; } else");

            assert!(found.is_err());
            assert_eq!(
                ParseError::UnexpectedToken {
                    line: 1,
                    col: 27,
                    found: TokenType::Eof,
                    expected: TokenType::LeftBrace,
                    msg: None
                },
                found.err().unwrap()
            );
        }

        #[test]
        fn empty_true_branch() {
            let found: Result<ScopeBoundStatement, ParseError> =
                parse("empty_true_branch", "if true {}");

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Conditional {
                    condition: Box::new(Expression::Literal {
                        literal: Arc::new(Token {
                            line: 1,
                            column: 3,
                            ttype: TokenType::True,
                            lexeme: "true".to_owned(),
                            found_in: "empty_true_branch".to_owned()
                        })
                    }),
                    true_branch: Box::new(ScopeBoundStatement::Scope(vec![])),
                    false_branch: None
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn empty_branches() {
            let found: Result<ScopeBoundStatement, ParseError> =
                parse("empty_branches", "if true {} else {}");

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Conditional {
                    condition: Box::new(Expression::Literal {
                        literal: Arc::new(Token {
                            line: 1,
                            column: 3,
                            ttype: TokenType::True,
                            lexeme: "true".to_owned(),
                            found_in: "empty_branches".to_owned()
                        })
                    }),
                    true_branch: Box::new(ScopeBoundStatement::Scope(vec![])),
                    false_branch: Some(Box::new(ScopeBoundStatement::Scope(vec![])))
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn else_with_conditional_missing_condition() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "else_with_conditional_missing_condition",
                "if true {} else if {} else {}",
            );

            assert!(found.is_err());
            assert_eq!(
                ParseError::InvalidExpression {
                    token: Arc::new(Token {
                        line: 1,
                        column: 19,
                        ttype: TokenType::LeftBrace,
                        lexeme: "{".to_owned(),
                        found_in: "else_with_conditional_missing_condition".to_owned()
                    })
                },
                found.err().unwrap()
            );
        }

        #[test]
        fn else_with_conditional() {
            let found: Result<ScopeBoundStatement, ParseError> =
                parse("else_with_conditional", "if true {} else if nil {}");

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Conditional {
                    condition: Box::new(Expression::Literal {
                        literal: Arc::new(Token {
                            line: 1,
                            column: 3,
                            ttype: TokenType::True,
                            lexeme: "true".to_owned(),
                            found_in: "else_with_conditional".to_owned()
                        })
                    }),
                    true_branch: Box::new(ScopeBoundStatement::Scope(vec![])),
                    false_branch: Some(Box::new(ScopeBoundStatement::Conditional {
                        condition: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 19,
                                ttype: TokenType::Nil,
                                lexeme: "nil".to_owned(),
                                found_in: "else_with_conditional".to_owned()
                            })
                        }),
                        true_branch: Box::new(ScopeBoundStatement::Scope(vec![])),
                        false_branch: None
                    }))
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn valid_if_stmt() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "valid_if",
                "if 23 == 23 { return 42; } else { return -42; }",
            );

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Conditional {
                    condition: Box::new(Expression::Binary {
                        left: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 3,
                                ttype: TokenType::Integer,
                                lexeme: "23".to_owned(),
                                found_in: "valid_if".to_owned(),
                            }),
                        }),
                        operation: Arc::new(Token {
                            line: 1,
                            column: 6,
                            ttype: TokenType::EqualEqual,
                            lexeme: "==".to_owned(),
                            found_in: "valid_if".to_owned(),
                        }),
                        right: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 9,
                                ttype: TokenType::Integer,
                                lexeme: "23".to_owned(),
                                found_in: "valid_if".to_owned(),
                            }),
                        }),
                    }),
                    true_branch: Box::new(ScopeBoundStatement::Scope(vec![
                        ScopeBoundStatement::Return(Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 21,
                                ttype: TokenType::Integer,
                                lexeme: "42".to_owned(),
                                found_in: "valid_if".to_owned(),
                            })
                        }))
                    ])),
                    false_branch: Some(Box::new(ScopeBoundStatement::Scope(vec![
                        ScopeBoundStatement::Return(Box::new(Expression::Unary {
                            operation: Arc::new(Token {
                                line: 1,
                                column: 41,
                                ttype: TokenType::Minus,
                                lexeme: "-".to_owned(),
                                found_in: "valid_if".to_owned()
                            }),
                            value: Box::new(Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 42,
                                    ttype: TokenType::Integer,
                                    lexeme: "42".to_owned(),
                                    found_in: "valid_if".to_owned(),
                                })
                            })
                        }))
                    ])))
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn valid_if_nested_condition() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "valid_if_grouping",
                "if (23 == 23) { return 42; } else { return -42; }",
            );

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Conditional {
                    condition: Box::new(Expression::Nested {
                        nested: Box::new(Expression::Binary {
                            left: Box::new(Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 4,
                                    ttype: TokenType::Integer,
                                    lexeme: "23".to_owned(),
                                    found_in: "valid_if_grouping".to_owned(),
                                }),
                            }),
                            operation: Arc::new(Token {
                                line: 1,
                                column: 7,
                                ttype: TokenType::EqualEqual,
                                lexeme: "==".to_owned(),
                                found_in: "valid_if_grouping".to_owned(),
                            }),
                            right: Box::new(Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 10,
                                    ttype: TokenType::Integer,
                                    lexeme: "23".to_owned(),
                                    found_in: "valid_if_grouping".to_owned(),
                                }),
                            }),
                        })
                    }),
                    true_branch: Box::new(ScopeBoundStatement::Scope(vec![
                        ScopeBoundStatement::Return(Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 23,
                                ttype: TokenType::Integer,
                                lexeme: "42".to_owned(),
                                found_in: "valid_if_grouping".to_owned(),
                            })
                        }))
                    ])),
                    false_branch: Some(Box::new(ScopeBoundStatement::Scope(vec![
                        ScopeBoundStatement::Return(Box::new(Expression::Unary {
                            operation: Arc::new(Token {
                                line: 1,
                                column: 43,
                                ttype: TokenType::Minus,
                                lexeme: "-".to_owned(),
                                found_in: "valid_if_grouping".to_owned()
                            }),
                            value: Box::new(Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 44,
                                    ttype: TokenType::Integer,
                                    lexeme: "42".to_owned(),
                                    found_in: "valid_if_grouping".to_owned(),
                                })
                            })
                        }))
                    ])))
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn valid_match() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "valid_match",
                "match 23 != 23 { true -> { 42 }, false -> { -42 } }",
            );

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Match {
                    on: Box::new(Expression::Binary {
                        left: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 6,
                                ttype: TokenType::Integer,
                                lexeme: "23".to_owned(),
                                found_in: "valid_match".to_owned()
                            })
                        }),
                        operation: Arc::new(Token {
                            line: 1,
                            column: 9,
                            ttype: TokenType::NotEqual,
                            lexeme: "!=".to_owned(),
                            found_in: "valid_match".to_owned()
                        }),
                        right: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 12,
                                ttype: TokenType::Integer,
                                lexeme: "23".to_owned(),
                                found_in: "valid_match".to_owned()
                            })
                        })
                    }),
                    cases: HashMap::from([
                        (
                            Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 17,
                                    ttype: TokenType::True,
                                    lexeme: "true".to_owned(),
                                    found_in: "valid_match".to_owned()
                                })
                            },
                            ScopeBoundStatement::Scope(vec![ScopeBoundStatement::ImplicitReturn(
                                Box::new(Expression::Literal {
                                    literal: Arc::new(Token {
                                        line: 1,
                                        column: 27,
                                        ttype: TokenType::Integer,
                                        lexeme: "42".to_owned(),
                                        found_in: "valid_match".to_owned()
                                    })
                                })
                            )])
                        ),
                        (
                            Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 33,
                                    ttype: TokenType::False,
                                    lexeme: "false".to_owned(),
                                    found_in: "valid_match".to_owned()
                                })
                            },
                            ScopeBoundStatement::Scope(vec![ScopeBoundStatement::ImplicitReturn(
                                Box::new(Expression::Unary {
                                    operation: Arc::new(Token {
                                        line: 1,
                                        column: 44,
                                        ttype: TokenType::Minus,
                                        lexeme: "-".to_owned(),
                                        found_in: "valid_match".to_owned()
                                    }),
                                    value: Box::new(Expression::Literal {
                                        literal: Arc::new(Token {
                                            line: 1,
                                            column: 45,
                                            ttype: TokenType::Integer,
                                            lexeme: "42".to_owned(),
                                            found_in: "valid_match".to_owned()
                                        })
                                    })
                                })
                            )])
                        )
                    ])
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn valid_match_nested_condition() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "valid_match_nested_condition",
                "match (23 != 23) { true -> { 42 }, false -> { -42 } }",
            );

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Match {
                    on: Box::new(Expression::Nested {
                        nested: Box::new(Expression::Binary {
                            left: Box::new(Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 7,
                                    ttype: TokenType::Integer,
                                    lexeme: "23".to_owned(),
                                    found_in: "valid_match_nested_condition".to_owned()
                                })
                            }),
                            operation: Arc::new(Token {
                                line: 1,
                                column: 10,
                                ttype: TokenType::NotEqual,
                                lexeme: "!=".to_owned(),
                                found_in: "valid_match_nested_condition".to_owned()
                            }),
                            right: Box::new(Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 13,
                                    ttype: TokenType::Integer,
                                    lexeme: "23".to_owned(),
                                    found_in: "valid_match_nested_condition".to_owned()
                                })
                            })
                        })
                    }),
                    cases: HashMap::from([
                        (
                            Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 19,
                                    ttype: TokenType::True,
                                    lexeme: "true".to_owned(),
                                    found_in: "valid_match_nested_condition".to_owned()
                                })
                            },
                            ScopeBoundStatement::Scope(vec![ScopeBoundStatement::ImplicitReturn(
                                Box::new(Expression::Literal {
                                    literal: Arc::new(Token {
                                        line: 1,
                                        column: 29,
                                        ttype: TokenType::Integer,
                                        lexeme: "42".to_owned(),
                                        found_in: "valid_match_nested_condition".to_owned()
                                    })
                                })
                            )])
                        ),
                        (
                            Expression::Literal {
                                literal: Arc::new(Token {
                                    line: 1,
                                    column: 35,
                                    ttype: TokenType::False,
                                    lexeme: "false".to_owned(),
                                    found_in: "valid_match_nested_condition".to_owned()
                                })
                            },
                            ScopeBoundStatement::Scope(vec![ScopeBoundStatement::ImplicitReturn(
                                Box::new(Expression::Unary {
                                    operation: Arc::new(Token {
                                        line: 1,
                                        column: 46,
                                        ttype: TokenType::Minus,
                                        lexeme: "-".to_owned(),
                                        found_in: "valid_match_nested_condition".to_owned()
                                    }),
                                    value: Box::new(Expression::Literal {
                                        literal: Arc::new(Token {
                                            line: 1,
                                            column: 47,
                                            ttype: TokenType::Integer,
                                            lexeme: "42".to_owned(),
                                            found_in: "valid_match_nested_condition".to_owned()
                                        })
                                    })
                                })
                            )])
                        )
                    ])
                },
                found.ok().unwrap()
            );
        }

        #[test]
        fn match_without_arrow() {
            let found: Result<ScopeBoundStatement, ParseError> = parse(
                "valid_match_nested_condition",
                "match 23 != 23 { true { 42 } }",
            );

            assert!(found.is_err());
            assert_eq!(
                ParseError::UnexpectedToken {
                    line: 1,
                    col: 22,
                    found: TokenType::LeftBrace,
                    expected: TokenType::Arrow,
                    msg: None,
                },
                found.err().unwrap()
            );
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
        fn inner_scope_implicit_return() {
            let found = parse("inner_scope_implicit_return", "{ {3 + 4} }");

            assert!(found.is_ok());
            assert_eq!(
                ScopeBoundStatement::Scope(vec![ScopeBoundStatement::Scope(vec![
                    ScopeBoundStatement::ImplicitReturn(Box::new(Expression::Binary {
                        left: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 3,
                                ttype: TokenType::Integer,
                                lexeme: "3".to_owned(),
                                found_in: "inner_scope_implicit_return".to_owned()
                            })
                        }),
                        operation: Arc::new(Token {
                            line: 1,
                            column: 5,
                            ttype: TokenType::Plus,
                            lexeme: "+".to_owned(),
                            found_in: "inner_scope_implicit_return".to_owned()
                        }),
                        right: Box::new(Expression::Literal {
                            literal: Arc::new(Token {
                                line: 1,
                                column: 7,
                                ttype: TokenType::Integer,
                                lexeme: "4".to_owned(),
                                found_in: "inner_scope_implicit_return".to_owned()
                            })
                        })
                    }))
                ])]),
                found.ok().unwrap()
            );
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
