use super::*;

#[test]
fn scope_not_closed() {
    let found = parse("invalid_scope", "{ 3 + 4;");
    assert!(found.is_err());

    match found.err().unwrap() {
        ParseError::UnexpectedToken {
            token, expected, ..
        } => {
            assert_eq!(1, token.line);
            assert_eq!(8, token.column);
            assert_eq!(TokenType::Eof, token.ttype);
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
            token, expected, ..
        } => {
            assert_eq!(1, token.line);
            assert_eq!(10, token.column);
            assert_eq!(TokenType::Eof, token.ttype);
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
            ScopeBoundStatement::ImplicitReturn(Expression::Binary {
                left: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 3,
                        ttype: TokenType::Integer,
                        lexeme: "3".to_owned(),
                        found_in: "inner_scope_implicit_return".to_owned()
                    })
                }),
                operation: Box::new(Token {
                    line: 1,
                    column: 5,
                    ttype: TokenType::Plus,
                    lexeme: "+".to_owned(),
                    found_in: "inner_scope_implicit_return".to_owned()
                }),
                right: Box::new(Expression::Literal {
                    literal: Box::new(Token {
                        line: 1,
                        column: 7,
                        ttype: TokenType::Integer,
                        lexeme: "4".to_owned(),
                        found_in: "inner_scope_implicit_return".to_owned()
                    })
                })
            })
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
                } = expr
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
