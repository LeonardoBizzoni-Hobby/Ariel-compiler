use std::collections::HashMap;

use super::*;

#[test]
fn missing_condition() {
    let found: Result<ScopeBoundStatement, ParseError> =
        parse("missing_condition", "if { return 42; }");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidExpression {
            token: Box::new(Token {
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
            token: Box::new(Token {
                line: 1,
                column: 8,
                ttype: TokenType::Return,
                lexeme: "return".to_string(),
                found_in: "missing_scope".to_string()
            }),
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
            token: Box::new(Token {
                line: 1,
                column: 27,
                ttype: TokenType::Eof,
                lexeme: String::new(),
                found_in: "missing_false_branch".to_string()
            }),
            expected: TokenType::LeftBrace,
            msg: None
        },
        found.err().unwrap()
    );
}

#[test]
fn empty_true_branch() {
    let found: Result<ScopeBoundStatement, ParseError> = parse("empty_true_branch", "if true {}");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::Conditional {
            condition: Expression::Literal {
                literal: Box::new(Token {
                    line: 1,
                    column: 3,
                    ttype: TokenType::True,
                    lexeme: "true".to_owned(),
                    found_in: "empty_true_branch".to_owned()
                })
            },
            true_branch: vec![],
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
            condition: Expression::Literal {
                literal: Box::new(Token {
                    line: 1,
                    column: 3,
                    ttype: TokenType::True,
                    lexeme: "true".to_owned(),
                    found_in: "empty_branches".to_owned()
                })
            },
            true_branch: vec![],
            false_branch: Some(vec![])
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
            token: Box::new(Token {
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
            condition: Expression::Literal {
                literal: Box::new(Token {
                    line: 1,
                    column: 3,
                    ttype: TokenType::True,
                    lexeme: "true".to_owned(),
                    found_in: "else_with_conditional".to_owned()
                })
            },
            true_branch: vec![],
            false_branch: Some(vec![ScopeBoundStatement::Conditional {
                condition: Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 19,
                        ttype: TokenType::Nil,
                        lexeme: "nil".to_owned(),
                        found_in: "else_with_conditional".to_owned()
                    })
                },
                true_branch: vec![],
                false_branch: None
            }])
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
            condition: Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 3,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_if".to_owned(),
                    }),
                }),
                operation: Box::new(Token {
                    line: 1,
                    column: 6,
                    ttype: TokenType::EqualEqual,
                    lexeme: "==".to_owned(),
                    found_in: "valid_if".to_owned(),
                }),
                right: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 9,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_if".to_owned(),
                    }),
                }),
            },
            true_branch: vec![ScopeBoundStatement::Return(Some(Box::new(
                ScopeBoundStatement::Expression(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 21,
                        ttype: TokenType::Integer,
                        lexeme: "42".to_owned(),
                        found_in: "valid_if".to_owned(),
                    })
                })
            )))],
            false_branch: Some(vec![ScopeBoundStatement::Return(Some(Box::new(
                ScopeBoundStatement::Expression(Expression::Unary {
                    operation: Box::new(Token {
                        line: 1,
                        column: 41,
                        ttype: TokenType::Minus,
                        lexeme: "-".to_owned(),
                        found_in: "valid_if".to_owned()
                    }),
                    value: Box::new(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 42,
                            ttype: TokenType::Integer,
                            lexeme: "42".to_owned(),
                            found_in: "valid_if".to_owned(),
                        })
                    })
                })
            )))])
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
            condition: Expression::Nested {
                nested: Box::new(Expression::Binary {
                    left: Box::new(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 4,
                            ttype: TokenType::Integer,
                            lexeme: "23".to_owned(),
                            found_in: "valid_if_grouping".to_owned(),
                        }),
                    }),
                    operation: Box::new(Token {
                        line: 1,
                        column: 7,
                        ttype: TokenType::EqualEqual,
                        lexeme: "==".to_owned(),
                        found_in: "valid_if_grouping".to_owned(),
                    }),
                    right: Box::new(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 10,
                            ttype: TokenType::Integer,
                            lexeme: "23".to_owned(),
                            found_in: "valid_if_grouping".to_owned(),
                        }),
                    }),
                })
            },
            true_branch: vec![ScopeBoundStatement::Return(Some(Box::new(
                ScopeBoundStatement::Expression(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 23,
                        ttype: TokenType::Integer,
                        lexeme: "42".to_owned(),
                        found_in: "valid_if_grouping".to_owned(),
                    })
                })
            )))],
            false_branch: Some(vec![ScopeBoundStatement::Return(Some(Box::new(
                ScopeBoundStatement::Expression(Expression::Unary {
                    operation: Box::new(Token {
                        line: 1,
                        column: 43,
                        ttype: TokenType::Minus,
                        lexeme: "-".to_owned(),
                        found_in: "valid_if_grouping".to_owned()
                    }),
                    value: Box::new(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 44,
                            ttype: TokenType::Integer,
                            lexeme: "42".to_owned(),
                            found_in: "valid_if_grouping".to_owned(),
                        })
                    })
                })
            )))])
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
            on: Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 6,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_match".to_owned()
                    })
                }),
                operation: Box::new(Token {
                    line: 1,
                    column: 9,
                    ttype: TokenType::NotEqual,
                    lexeme: "!=".to_owned(),
                    found_in: "valid_match".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 12,
                        ttype: TokenType::Integer,
                        lexeme: "23".to_owned(),
                        found_in: "valid_match".to_owned()
                    })
                })
            },
            cases: HashMap::from([
                (
                    Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 17,
                            ttype: TokenType::True,
                            lexeme: "true".to_owned(),
                            found_in: "valid_match".to_owned()
                        })
                    },
                    vec![ScopeBoundStatement::ImplicitReturn(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 27,
                            ttype: TokenType::Integer,
                            lexeme: "42".to_owned(),
                            found_in: "valid_match".to_owned()
                        })
                    })]
                ),
                (
                    Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 33,
                            ttype: TokenType::False,
                            lexeme: "false".to_owned(),
                            found_in: "valid_match".to_owned()
                        })
                    },
                    vec![ScopeBoundStatement::ImplicitReturn(Expression::Unary {
                        operation: Box::new(Token {
                            line: 1,
                            column: 44,
                            ttype: TokenType::Minus,
                            lexeme: "-".to_owned(),
                            found_in: "valid_match".to_owned()
                        }),
                        value: Box::new(Expression::Literal {
                            literal: Box::new(Token {
                                line: 1,
                                column: 45,
                                ttype: TokenType::Integer,
                                lexeme: "42".to_owned(),
                                found_in: "valid_match".to_owned()
                            })
                        })
                    })]
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
            on: Expression::Nested {
                nested: Box::new(Expression::Binary {
                    left: Box::new(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 7,
                            ttype: TokenType::Integer,
                            lexeme: "23".to_owned(),
                            found_in: "valid_match_nested_condition".to_owned()
                        })
                    }),
                    operation: Box::new(Token {
                        line: 1,
                        column: 10,
                        ttype: TokenType::NotEqual,
                        lexeme: "!=".to_owned(),
                        found_in: "valid_match_nested_condition".to_owned()
                    }),
                    right: Box::new(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 13,
                            ttype: TokenType::Integer,
                            lexeme: "23".to_owned(),
                            found_in: "valid_match_nested_condition".to_owned()
                        })
                    })
                })
            },
            cases: HashMap::from([
                (
                    Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 19,
                            ttype: TokenType::True,
                            lexeme: "true".to_owned(),
                            found_in: "valid_match_nested_condition".to_owned()
                        })
                    },
                    vec![ScopeBoundStatement::ImplicitReturn(Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 29,
                            ttype: TokenType::Integer,
                            lexeme: "42".to_owned(),
                            found_in: "valid_match_nested_condition".to_owned()
                        })
                    })]
                ),
                (
                    Expression::Literal {
                        literal: Box::new(Token {
                            line: 1,
                            column: 35,
                            ttype: TokenType::False,
                            lexeme: "false".to_owned(),
                            found_in: "valid_match_nested_condition".to_owned()
                        })
                    },
                    vec![ScopeBoundStatement::ImplicitReturn(Expression::Unary {
                        operation: Box::new(Token {
                            line: 1,
                            column: 46,
                            ttype: TokenType::Minus,
                            lexeme: "-".to_owned(),
                            found_in: "valid_match_nested_condition".to_owned()
                        }),
                        value: Box::new(Expression::Literal {
                            literal: Box::new(Token {
                                line: 1,
                                column: 47,
                                ttype: TokenType::Integer,
                                lexeme: "42".to_owned(),
                                found_in: "valid_match_nested_condition".to_owned()
                            })
                        })
                    })]
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
            token: Box::new(Token {
                line: 1,
                column: 22,
                ttype: TokenType::LeftBrace,
                lexeme: "{".to_owned(),
                found_in: "valid_match_nested_condition".to_owned()
            }),
            expected: TokenType::Arrow,
            msg: None,
        },
        found.err().unwrap()
    );
}
