use std::collections::HashMap;

use super::*;
use crate::ast_generator::ast::variables::{DataType, Variable};

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
fn let_while_value() {
    let found = parse("let_while_value", "let name : u8 = while true;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::While,
                lexeme: "while".to_owned(),
                found_in: "let_while_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_loop_value() {
    let found = parse("let_loop_value", "let name : u8 = loop;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::Loop,
                lexeme: "loop".to_owned(),
                found_in: "let_loop_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_for_value() {
    let found = parse(
        "let_for_value",
        "let name : u8 = for (let i := 0; i < 100; i += 1);",
    );

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::For,
                lexeme: "for".to_owned(),
                found_in: "let_for_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_return_value() {
    let found = parse("let_return_value", "let name : u8 = return;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::Return,
                lexeme: "return".to_owned(),
                found_in: "let_return_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_let_value() {
    let found = parse("let_let_value", "let name : u8 = let _ := 0;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::Let,
                lexeme: "let".to_owned(),
                found_in: "let_let_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_break_value() {
    let found = parse("let_break_value", "let name : u8 = break;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::Break,
                lexeme: "break".to_owned(),
                found_in: "let_break_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_continue_value() {
    let found = parse("let_continue_value", "let name : u8 = continue;");

    assert!(found.is_err());
    assert_eq!(
        ParseError::InvalidVariableAssignment {
            value: Arc::new(Token {
                line: 1,
                column: 16,
                ttype: TokenType::Continue,
                lexeme: "continue".to_owned(),
                found_in: "let_continue_value".to_owned()
            })
        },
        found.err().unwrap()
    );
}

#[test]
fn let_if_value() {
    let found = parse("let_if_value", "let name : isize = if true {};");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::VariableDeclaration(Variable::new(
            Arc::new(Token {
                line: 1,
                column: 4,
                ttype: TokenType::Identifier,
                lexeme: "name".to_owned(),
                found_in: "let_if_value".to_owned()
            }),
            Some(DataType::Isize),
            Box::new(ScopeBoundStatement::Conditional {
                condition: Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 22,
                        ttype: TokenType::True,
                        lexeme: "true".to_owned(),
                        found_in: "let_if_value".to_owned()
                    })
                },
                true_branch: vec![],
                false_branch: None
            })
        )),
        found.ok().unwrap()
    );
}

#[test]
fn let_match_value() {
    let found = parse("let_match_value", "let name : isize = match 42 {};");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::VariableDeclaration(Variable::new(
            Arc::new(Token {
                line: 1,
                column: 4,
                ttype: TokenType::Identifier,
                lexeme: "name".to_owned(),
                found_in: "let_match_value".to_owned()
            }),
            Some(DataType::Isize),
            Box::new(ScopeBoundStatement::Match {
                on: Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 25,
                        ttype: TokenType::Integer,
                        lexeme: "42".to_owned(),
                        found_in: "let_match_value".to_owned()
                    })
                },
                cases: HashMap::new()
            })
        )),
        found.ok().unwrap()
    );
}

#[test]
fn let_scope_value() {
    let found = parse("let_scope_value", "let name : isize = {};");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::VariableDeclaration(Variable::new(
            Arc::new(Token {
                line: 1,
                column: 4,
                ttype: TokenType::Identifier,
                lexeme: "name".to_owned(),
                found_in: "let_scope_value".to_owned()
            }),
            Some(DataType::Isize),
            Box::new(ScopeBoundStatement::Scope(vec![]))
        )),
        found.ok().unwrap()
    );
}

#[test]
fn let_expr_value() {
    let found = parse("let_expr_value", "let name : isize = 12 + 30;");

    assert!(found.is_ok());
    assert_eq!(
        ScopeBoundStatement::VariableDeclaration(Variable::new(
            Arc::new(Token {
                line: 1,
                column: 4,
                ttype: TokenType::Identifier,
                lexeme: "name".to_owned(),
                found_in: "let_expr_value".to_owned()
            }),
            Some(DataType::Isize),
            Box::new(ScopeBoundStatement::Expression(Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 19,
                        ttype: TokenType::Integer,
                        lexeme: "12".to_owned(),
                        found_in: "let_expr_value".to_owned()
                    })
                }),
                operation: Arc::new(Token {
                    line: 1,
                    column: 22,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "let_expr_value".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Arc::new(Token {
                        line: 1,
                        column: 24,
                        ttype: TokenType::Integer,
                        lexeme: "30".to_owned(),
                        found_in: "let_expr_value".to_owned()
                    })
                })
            }))
        )),
        found.ok().unwrap()
    );
}
