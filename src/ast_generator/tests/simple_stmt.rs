use super::*;

#[test]
fn defer_without_stmt() {
    let found = parse("defer_without_stmt", "defer;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Box::new(Token {
                line: 1,
                column: 5,
                ttype: TokenType::Semicolon,
                lexeme: ";".to_owned(),
                found_in: "defer_without_stmt".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn defer_empty_scope_stmt() {
    let found = parse("defer_empty_scope_stmt", "defer {};");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::Defer(Box::new(ScopeBoundStatement::Scope(vec![]))),
        found.ok().unwrap()
    );
}

#[test]
fn defer_stmt() {
    let found = parse("defer_stmt", "defer 2 + 40;");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::Defer(Box::new(ScopeBoundStatement::Expression(
            Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 6,
                        ttype: TokenType::Integer,
                        lexeme: "2".to_owned(),
                        found_in: "defer_stmt".to_owned()
                    })
                }),
                operation: Box::new(Token {
                    line: 1,
                    column: 8,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "defer_stmt".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 10,
                        ttype: TokenType::Integer,
                        lexeme: "40".to_owned(),
                        found_in: "defer_stmt".to_owned()
                    })
                })
            }
        ))),
        found.ok().unwrap()
    );
}

#[test]
fn return_nothing() {
    let found = parse("return_nothing", "return;");

    assert!(found.is_ok());
    assert_eq!(ScopeBoundStatement::Return(None), found.ok().unwrap());
}

#[test]
fn return_expr() {
    let found = parse("return_expr", "return 2 + 3;");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::Return(Some(Box::new(ScopeBoundStatement::Expression(
            Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 7,
                        ttype: TokenType::Integer,
                        lexeme: "2".to_owned(),
                        found_in: "return_expr".to_owned()
                    })
                }),
                operation: Box::new(Token {
                    line: 1,
                    column: 9,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "return_expr".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 11,
                        ttype: TokenType::Integer,
                        lexeme: "3".to_owned(),
                        found_in: "return_expr".to_owned()
                    })
                })
            }
        )))),
        found.ok().unwrap()
    );
}

#[test]
fn return_stmt() {
    let found = parse("return_stmt", "return if nil { 23 };");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::Return(Some(Box::new(ScopeBoundStatement::Conditional {
            condition: Expression::Literal {
                literal: Box::new(Token {
                    line: 1,
                    column: 10,
                    ttype: TokenType::Nil,
                    lexeme: "nil".to_owned(),
                    found_in: "return_stmt".to_owned()
                })
            },
            true_branch: vec![ScopeBoundStatement::ImplicitReturn(Expression::Literal {
                literal: Box::new(Token {
                    line: 1,
                    column: 16,
                    ttype: TokenType::Integer,
                    lexeme: "23".to_owned(),
                    found_in: "return_stmt".to_owned()
                })
            })],
            false_branch: None
        }))),
        found.ok().unwrap()
    );
}

#[test]
fn return_stmt_no_semicolon() {
    let found = parse("return_stmt_no_semicolon", "return if nil { 23 }");

    assert!(found.is_err());
    assert_eq!(
        ParseError::UnexpectedToken {
            token: Box::new(Token {
                line: 1,
                column: 20,
                ttype: TokenType::Eof,
                lexeme: "".to_owned(),
                found_in: "return_stmt_no_semicolon".to_owned()
            }),
            expected: TokenType::Semicolon,
            msg: None
        },
        found.err().unwrap()
    );
}

#[test]
fn return_expr_no_semicolon() {
    let found = parse("return_expr_no_semicolon", "return 2 + 3");

    assert!(found.is_err());
    assert_eq!(
        ParseError::UnexpectedToken {
            token: Box::new(Token {
                line: 1,
                column: 12,
                ttype: TokenType::Eof,
                lexeme: "".to_owned(),
                found_in: "return_expr_no_semicolon".to_owned()
            }),
            expected: TokenType::Semicolon,
            msg: None
        },
        found.err().unwrap()
    );
}

#[test]
fn invalid_continue() {
    let found = parse("invalid_continue", "continue");
    assert!(found.is_err());

    match found.err().unwrap() {
        ParseError::UnexpectedToken {
            token, expected, ..
        } => {
            assert_eq!(1, token.line);
            assert_eq!(8, token.column);
            assert_eq!(TokenType::Eof, token.ttype);
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
    assert_eq!(
        ParseError::UnexpectedToken {
            token: Box::new(Token {
                line: 1,
                column: 5,
                ttype: TokenType::Eof,
                lexeme: "".to_owned(),
                found_in: "invalid_break".to_owned()
            }),
            expected: TokenType::Semicolon,
            msg: None
        },
        found.err().unwrap()
    );
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
