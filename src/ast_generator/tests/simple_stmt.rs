use crate::ast_generator::ast::variables::{DataType, Variable};

#[allow(unused_imports)]
use super::*;

#[test]
fn defer_without_stmt() {
    let found = parse("defer_without_stmt", "defer;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Arc::new(Token {
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
                    literal: Arc::new(Token {
                        line: 1,
                        column: 6,
                        ttype: TokenType::Integer,
                        lexeme: "2".to_owned(),
                        found_in: "defer_stmt".to_owned()
                    })
                }),
                operation: Arc::new(Token {
                    line: 1,
                    column: 8,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "defer_stmt".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
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
fn let_without_value() {
    let found = parse("let_without_value", "let _ : u8;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableDeclaration {
            line: 1,
            column: 10
        },
        found.err().unwrap()
    );
}

#[test]
fn let_builtin_type() {
    let found = parse("let_builtin_type", "let _ : u8 = 10;");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::VariableDeclaration(Variable::new(
            Arc::new(Token {
                line: 1,
                column: 4,
                ttype: TokenType::DontCare,
                lexeme: "_".to_owned(),
                found_in: "let_builtin_type".to_owned()
            }),
            Some(crate::ast_generator::ast::variables::DataType::U8),
            Box::new(ScopeBoundStatement::Expression(Expression::Literal {
                literal: Arc::new(Token {
                    line: 1,
                    column: 13,
                    ttype: TokenType::Integer,
                    lexeme: "10".to_owned(),
                    found_in: "let_builtin_type".to_owned()
                })
            }))
        )),
        found.ok().unwrap()
    );
}

#[test]
fn let_custom_type() {
    let found = parse("let_custom_type", "let _ : Hello = 10;");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::VariableDeclaration(Variable::new(
            Arc::new(Token {
                line: 1,
                column: 4,
                ttype: TokenType::DontCare,
                lexeme: "_".to_owned(),
                found_in: "let_custom_type".to_owned()
            }),
            Some(DataType::Compound {
                name: Arc::new(Token {
                    line: 1,
                    column: 8,
                    ttype: TokenType::Identifier,
                    lexeme: "Hello".to_owned(),
                    found_in: "let_custom_type".to_owned()
                })
            }),
            Box::new(ScopeBoundStatement::Expression(Expression::Literal {
                literal: Arc::new(Token {
                    line: 1,
                    column: 16,
                    ttype: TokenType::Integer,
                    lexeme: "10".to_owned(),
                    found_in: "let_custom_type".to_owned()
                })
            }))
        )),
        found.ok().unwrap()
    );
}

#[test]
fn let_invalid_type() {
    let found = parse("let_invalid_type", "let _ : 8 = 10;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidDataType {
            line: 1,
            col: 8,
            found: TokenType::Integer,
            msg: None
        },
        found.err().unwrap()
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
                    literal: Arc::new(Token {
                        line: 1,
                        column: 7,
                        ttype: TokenType::Integer,
                        lexeme: "2".to_owned(),
                        found_in: "return_expr".to_owned()
                    })
                }),
                operation: Arc::new(Token {
                    line: 1,
                    column: 9,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "return_expr".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
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
                literal: Arc::new(Token {
                    line: 1,
                    column: 10,
                    ttype: TokenType::Nil,
                    lexeme: "nil".to_owned(),
                    found_in: "return_stmt".to_owned()
                })
            },
            true_branch: vec![ScopeBoundStatement::ImplicitReturn(Expression::Literal {
                literal: Arc::new(Token {
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
            line: 1,
            col: 20,
            found: TokenType::Eof,
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
            line: 1,
            col: 12,
            found: TokenType::Eof,
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
    assert_eq!(
        ParseError::UnexpectedToken {
            line: 1,
            col: 5,
            found: TokenType::Eof,
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
