use super::*;
use crate::ast_generator::ast::variables::Variable;

#[test]
fn while_no_condition() {
    let found = parse("while_no_condition", "while { 23 + 19; }");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Arc::new(Token {
                line: 1,
                column: 6,
                ttype: TokenType::LeftBrace,
                lexeme: "{".to_owned(),
                found_in: "while_no_condition".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn while_no_condition_body() {
    let found = parse("while_no_condition_body", "while;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Arc::new(Token {
                line: 1,
                column: 5,
                ttype: TokenType::Semicolon,
                lexeme: ";".to_owned(),
                found_in: "while_no_condition_body".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn valid_while_no_body() {
    let found = parse("valid_while_no_body", "while true;");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::While {
            condition: Expression::Literal {
                literal: Arc::new(Token {
                    line: 1,
                    column: 6,
                    ttype: TokenType::True,
                    lexeme: "true".to_owned(),
                    found_in: "valid_while_no_body".to_owned()
                })
            },
            body: None
        },
        found.ok().unwrap()
    );
}

#[test]
fn valid_while() {
    let found = parse("valid_while", "while nil { 23 + 19; }");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::While {
            condition: Expression::Literal {
                literal: Arc::new(Token {
                    line: 1,
                    column: 6,
                    ttype: TokenType::Nil,
                    lexeme: "nil".to_owned(),
                    found_in: "valid_while".to_owned()
                })
            },
            body: Some(vec![ScopeBoundStatement::Expression(Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 12,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_while".to_owned()
                    })
                }),
                operation: Arc::new(Token {
                    line: 1,
                    column: 15,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "valid_while".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 17,
                        ttype: TokenType::Integer,
                        lexeme: "19".to_owned(),
                        found_in: "valid_while".to_owned()
                    })
                })
            })])
        },
        found.ok().unwrap()
    );
}

#[test]
fn empty_for_no_body() {
    let found = parse("empty_for_no_body", "for ( ; ; );");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::For {
            initialization: None,
            condition: None,
            increment: None,
            body: None
        },
        found.ok().unwrap()
    );
}

#[test]
fn empty_for() {
    let found = parse("empty_for_body", "for ( ; ; ) {}");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::For {
            initialization: None,
            condition: None,
            increment: None,
            body: Some(vec![])
        },
        found.ok().unwrap()
    );
}

#[test]
fn for_with_init() {
    let found = parse("for_with_init", "for (let a := 0; ; );");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::For {
            initialization: Some(Box::new(ScopeBoundStatement::VariableDeclaration(
                Variable::new(
                    Arc::new(Token {
                        line: 1,
                        column: 9,
                        ttype: TokenType::Identifier,
                        lexeme: "a".to_owned(),
                        found_in: "for_with_init".to_owned()
                    }),
                    None,
                    Box::new(ScopeBoundStatement::Expression(Expression::Literal {
                        literal: Arc::new(Token {
                            line: 1,
                            column: 14,
                            ttype: TokenType::Integer,
                            lexeme: "0".to_owned(),
                            found_in: "for_with_init".to_owned()
                        })
                    }))
                )
            ))),
            condition: None,
            increment: None,
            body: None
        },
        found.ok().unwrap()
    );
}

#[test]
fn for_with_condition() {
    let found = parse("for_with_condition", "for (; true; );");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::For {
            initialization: None,
            condition: Some(Expression::Literal {
                literal: Arc::new(Token {
                    line: 1,
                    column: 7,
                    ttype: TokenType::True,
                    lexeme: "true".to_owned(),
                    found_in: "for_with_condition".to_owned()
                })
            }),
            increment: None,
            body: None
        },
        found.ok().unwrap()
    );
}

#[test]
fn for_with_invalid_condition() {
    let found = parse(
        "for_with_invalid_condition",
        "for (; if true { a += 10 }; );",
    );

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Arc::new(Token {
                line: 1,
                column: 7,
                ttype: TokenType::If,
                lexeme: "if".to_owned(),
                found_in: "for_with_invalid_condition".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn for_with_increment() {
    let found = parse("for_with_increment", "for (; ; a += 10);");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::For {
            initialization: None,
            condition: None,
            increment: Some(Expression::Binary {
                left: Box::new(Expression::Name {
                    name: Arc::new(Token {
                        line: 1,
                        column: 9,
                        ttype: TokenType::Identifier,
                        lexeme: "a".to_owned(),
                        found_in: "for_with_increment".to_owned()
                    })
                }),
                operation: Arc::new(Token {
                    line: 1,
                    column: 11,
                    ttype: TokenType::PlusEquals,
                    lexeme: "+=".to_owned(),
                    found_in: "for_with_increment".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 14,
                        ttype: TokenType::Integer,
                        lexeme: "10".to_owned(),
                        found_in: "for_with_increment".to_owned()
                    })
                })
            }),
            body: None
        },
        found.ok().unwrap()
    );
}

#[test]
fn for_with_invalid_increment() {
    let found = parse(
        "for_with_invalid_increment",
        "for (; ; if true { a += 10 });",
    );

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Arc::new(Token {
                line: 1,
                column: 9,
                ttype: TokenType::If,
                lexeme: "if".to_owned(),
                found_in: "for_with_invalid_increment".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn full_for() {
    let found = parse("full_for", "for (let a:= 0; true; a += 10);");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::For {
            initialization: Some(Box::new(ScopeBoundStatement::VariableDeclaration(
                Variable::new(
                    Arc::new(Token {
                        line: 1,
                        column: 9,
                        ttype: TokenType::Identifier,
                        lexeme: "a".to_owned(),
                        found_in: "full_for".to_owned()
                    }),
                    None,
                    Box::new(ScopeBoundStatement::Expression(Expression::Literal {
                        literal: Arc::new(Token {
                            line: 1,
                            column: 13,
                            ttype: TokenType::Integer,
                            lexeme: "0".to_owned(),
                            found_in: "full_for".to_owned()
                        })
                    }))
                )
            ))),
            condition: Some(Expression::Literal {
                literal: Arc::new(Token {
                    line: 1,
                    column: 16,
                    ttype: TokenType::True,
                    lexeme: "true".to_owned(),
                    found_in: "full_for".to_owned()
                })
            }),
            increment: Some(Expression::Binary {
                left: Box::new(Expression::Name {
                    name: Arc::new(Token {
                        line: 1,
                        column: 22,
                        ttype: TokenType::Identifier,
                        lexeme: "a".to_owned(),
                        found_in: "full_for".to_owned()
                    })
                }),
                operation: Arc::new(Token {
                    line: 1,
                    column: 24,
                    ttype: TokenType::PlusEquals,
                    lexeme: "+=".to_owned(),
                    found_in: "full_for".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 27,
                        ttype: TokenType::Integer,
                        lexeme: "10".to_owned(),
                        found_in: "full_for".to_owned()
                    })
                })
            }),
            body: None
        },
        found.ok().unwrap()
    );
}
