use super::{source::SourceFile, token::Token, token_type::TokenType};
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Arc};

lazy_static! {
    static ref KEYWORD: HashMap<String, TokenType> = {
        HashMap::from([
            ("_".to_owned(), TokenType::DontCare),
            ("break".to_owned(), TokenType::Break),
            ("bool".to_owned(), TokenType::Bool),
            ("continue".to_owned(), TokenType::Continue),
            ("else".to_owned(), TokenType::Else),
            ("enum".to_owned(), TokenType::Enum),
            ("false".to_owned(), TokenType::False),
            ("fn".to_owned(), TokenType::Fn),
            ("for".to_owned(), TokenType::For),
            ("if".to_owned(), TokenType::If),
            ("import".to_owned(), TokenType::Import),
            ("let".to_owned(), TokenType::Let),
            ("loop".to_owned(), TokenType::Loop),
            ("main".to_owned(), TokenType::Main),
            ("match".to_owned(), TokenType::Match),
            ("nil".to_owned(), TokenType::Nil),
            ("return".to_owned(), TokenType::Return),
            ("struct".to_owned(), TokenType::Struct),
            ("true".to_owned(), TokenType::True),
            ("void".to_owned(), TokenType::Void),
            ("while".to_owned(), TokenType::While),
            ("u8".to_owned(), TokenType::U8),
            ("u16".to_owned(), TokenType::U16),
            ("u32".to_owned(), TokenType::U32),
            ("u64".to_owned(), TokenType::U64),
            ("i8".to_owned(), TokenType::I8),
            ("i16".to_owned(), TokenType::I16),
            ("i32".to_owned(), TokenType::I32),
            ("i64".to_owned(), TokenType::I64),
            ("f32".to_owned(), TokenType::F32),
            ("f64".to_owned(), TokenType::F64),
            ("str".to_owned(), TokenType::StringType),
        ])
    };
}

pub fn get_token(source: &mut SourceFile) -> Arc<Token> {
    skip_whitespace(source);

    source.start = source.current;

    match advance(source) {
        0 => make_token(TokenType::Eof, source),
        b'"' => make_string_token(source),
        b'0'..=b'9' => make_number_token(source),
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => make_identifier_token(source),
        b'(' => make_token(TokenType::LeftParen, source),
        b')' => make_token(TokenType::RightParen, source),
        b'[' => make_token(TokenType::LeftSquare, source),
        b']' => make_token(TokenType::RightSquare, source),
        b'{' => make_token(TokenType::LeftBrace, source),
        b'}' => make_token(TokenType::RightBrace, source),
        b'?' => make_token(TokenType::Question, source),
        b',' => make_token(TokenType::Comma, source),
        b';' => make_token(TokenType::Semicolon, source),
        b'%' => make_token(TokenType::Mod, source),
        b'.' => match source.peek() {
            b'.' => {
                advance(source);
                match source.peek() {
                    b'=' => {
                        advance(source);
                        make_token(TokenType::IterEqual, source)
                    }
                    _ => make_token(TokenType::Iter, source),
                }
            }
            _ => make_token(TokenType::Dot, source),
        },
        b':' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::DynamicDefinition, source)
            }
            b':' => {
                advance(source);
                make_token(TokenType::StaticScopeGetter, source)
            }
            _ => make_token(TokenType::Colon, source),
        },
        b'!' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::NotEqual, source)
            }
            _ => make_token(TokenType::Not, source),
        },
        b'=' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::EqualEqual, source)
            }
            _ => make_token(TokenType::Equal, source),
        },
        b'>' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::GreaterEqual, source)
            }
            b'>' => {
                advance(source);
                match source.peek() {
                    b'=' => {
                        advance(source);
                        make_token(TokenType::ShiftRightEqual, source)
                    }
                    _ => make_token(TokenType::ShiftRight, source),
                }
            }
            _ => make_token(TokenType::Greater, source),
        },
        b'<' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::LessEqual, source)
            }
            b'<' => {
                advance(source);
                match source.peek() {
                    b'=' => {
                        advance(source);
                        make_token(TokenType::ShiftLeftEqual, source)
                    }
                    _ => make_token(TokenType::ShiftLeft, source),
                }
            }
            _ => make_token(TokenType::Less, source),
        },
        b'&' => match source.peek() {
            b'&' => {
                advance(source);
                make_token(TokenType::And, source)
            }
            _ => make_token(TokenType::BitAnd, source),
        },
        b'|' => match source.peek() {
            b'|' => {
                advance(source);
                make_token(TokenType::Or, source)
            }
            _ => make_token(TokenType::BitOr, source),
        },
        b'-' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::MinusEquals, source)
            }
            b'>' => {
                advance(source);
                make_token(TokenType::Arrow, source)
            }
            _ => make_token(TokenType::Minus, source),
        },
        b'+' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::PlusEquals, source)
            }
            _ => make_token(TokenType::Plus, source),
        },
        b'/' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::SlashEquals, source)
            }
            b'/' => {
                advance(source);
                match source.peek() {
                    b'=' => {
                        advance(source);
                        make_token(TokenType::IntegerSlashEquals, source)
                    }
                    _ => make_token(TokenType::IntegerSlash, source),
                }
            }
            _ => make_token(TokenType::Slash, source),
        },
        b'*' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::StarEquals, source)
            }
            _ => make_token(TokenType::Star, source),
        },
        b'^' => match source.peek() {
            b'=' => {
                advance(source);
                make_token(TokenType::PowerEquals, source)
            }
            _ => make_token(TokenType::Power, source),
        },
        ch => make_token(TokenType::Unknown(ch as char), source),
    }
}

fn make_token(ttype: TokenType, source: &mut SourceFile) -> Arc<Token> {
    let start: usize = source.start;
    let current: usize = source.current;

    let binding: Vec<u8> = source.mmap[start..current].to_vec();
    let lexeme: String = String::from_utf8_lossy(&binding).to_string();

    Arc::new(Token::new(
        source.line,
        source.column - lexeme.len(),
        ttype,
        lexeme,
        source.name.clone(),
    ))
}

fn make_number_token(source: &mut SourceFile) -> Arc<Token> {
    while source.peek().is_ascii_digit() {
        advance(source);
    }

    if source.peek() == b'.' && source.peek_next().is_ascii_digit() {
        advance(source);

        while source.peek().is_ascii_digit() {
            advance(source);
        }

        make_token(TokenType::Double, source)
    } else {
        make_token(TokenType::Integer, source)
    }
}

fn make_string_token(source: &mut SourceFile) -> Arc<Token> {
    let mut lexeme: String = String::new();
    while source.peek() != b'"' {
        match source.peek() {
            0 => break,
            b'\\' => {
                advance(source);
                match advance(source) {
                    b'n' => lexeme += "\n",
                    b't' => lexeme += "\t",
                    b'r' => lexeme += "\r",
                    b'\'' => lexeme += "\'",
                    b'"' => lexeme += "\"",
                    b'\\' => lexeme += "\\",
                    ch => {
                        return make_token_from(
                            TokenType::Unknown(ch as char),
                            "Invalid escape sequence.",
                            source,
                        )
                    }
                }
            }
            _ => {
                lexeme += &{
                    match String::from_utf8(vec![advance(source)]) {
                        Ok(t) => t,
                        Err(_) => {
                            return make_token_from(
                                TokenType::InvalidByteSequenceToString,
                                &lexeme,
                                source,
                            )
                        }
                    }
                }
            }
        }
    }

    match source.peek() {
        b'"' => {
            advance(source);
            make_token_from(TokenType::String, &lexeme, source)
        }
        // i don't think this can assume any other value than 0
        unexpected_char => make_token_from(
            TokenType::Unknown(unexpected_char as char),
            "Unterminated string.",
            source,
        ),
    }
}

fn make_identifier_token(source: &mut SourceFile) -> Arc<Token> {
    while source.peek().is_ascii_alphanumeric() || source.peek() == b'_' {
        advance(source);
    }

    let start: usize = source.start;
    let current: usize = source.current;
    let identifier: String = String::from_utf8_lossy(&source.mmap[start..current]).to_string();

    match KEYWORD.get(&identifier) {
        Some(token_type) => make_token(token_type.clone(), source),
        None => make_token(TokenType::Identifier, source),
    }
}

fn make_token_from(ttype: TokenType, lexeme: &str, source: &mut SourceFile) -> Arc<Token> {
    Arc::new(Token::new(
        source.line,
        {
            match ttype {
                TokenType::Unknown(_) => source.column - 1,
                _ => source.column - lexeme.len(),
            }
        },
        ttype,
        lexeme.to_string(),
        source.name.clone(),
    ))
}

fn skip_whitespace(source: &mut SourceFile) {
    loop {
        match source.peek() {
            b'\n' | b'\r' => {
                advance(source);

                source.line += 1;
                source.column = 0;
            }
            b' ' | b'\t' => {
                advance(source);
            }
            b'#' => {
                match source.peek_next() {
                    // Multi-line comment
                    b'#' => {
                        advance(source);

                        while !(source.peek() == b'#' && source.peek_next() == b'#') {
                            if matches!(advance(source), b'\n' | b'\r') {
                                source.line += 1;
                                source.column = 0;
                            }
                        }

                        advance(source);
                        advance(source);
                    }
                    // Single line comment
                    _ => {
                        while source.peek() != b'\n' {
                            advance(source);
                        }
                    }
                }
            }
            _ => return,
        };
    }
}

fn advance(source: &mut SourceFile) -> u8 {
    match source.peek() {
        0 => 0,
        value => {
            source.current += 1;
            source.column += 1;
            value
        }
    }
}

#[allow(unused_imports)]
#[allow(dead_code)]
mod tests {
    use crate::test_util::{create_test_file, delete_test_file};
    use super::*;
    use std::{fs::File, io::Write};

    fn scan_file(path: &str) -> Vec<Arc<Token>> {
        let mut scanned: Vec<Arc<Token>> = vec![];
        let mut source = SourceFile::new(path).unwrap();
        loop {
            let tk = get_token(&mut source);
            scanned.push(Arc::clone(&tk));

            if tk.ttype == TokenType::Eof {
                break;
            }
        }

        return scanned;
    }

    fn test_tokentype_equality(expected: Vec<TokenType>, found: Vec<Arc<Token>>) {
        assert_eq!(
            expected,
            found
                .iter()
                .map(|tk| tk.ttype.clone())
                .collect::<Vec<TokenType>>()
        );
    }

    #[test]
    fn single_file_tokenization() {
        create_test_file("single.file", "a b c d e");

        let scanned: Vec<Arc<Token>> = scan_file("single.file");
        let expected = vec![
            Arc::new(Token::new(
                1,
                0,
                TokenType::Identifier,
                String::from("a"),
                "single.file".to_string(),
            )),
            Arc::new(Token::new(
                1,
                2,
                TokenType::Identifier,
                String::from("b"),
                "single.file".to_string(),
            )),
            Arc::new(Token::new(
                1,
                4,
                TokenType::Identifier,
                String::from("c"),
                "single.file".to_string(),
            )),
            Arc::new(Token::new(
                1,
                6,
                TokenType::Identifier,
                String::from("d"),
                "single.file".to_string(),
            )),
            Arc::new(Token::new(
                1,
                8,
                TokenType::Identifier,
                String::from("e"),
                "single.file".to_string(),
            )),
            Arc::new(Token::new(
                1,
                9,
                TokenType::Eof,
                String::new(),
                "single.file".to_string(),
            )),
        ];

        assert_eq!(expected.len(), scanned.len());
        for x in 0..scanned.len() {
            assert_eq!(expected[x], scanned[x]);
        }

        delete_test_file("single.file");
    }

    #[test]
    fn keyword_tokenization_test() {
        create_test_file("keyword.test", "_ break continue else enum false fn for if import let loop main match nil return struct true void while u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 str");

        let scanned: Vec<Arc<Token>> = scan_file("keyword.test");
        let expected_types = vec![
            TokenType::DontCare,
            TokenType::Break,
            TokenType::Continue,
            TokenType::Else,
            TokenType::Enum,
            TokenType::False,
            TokenType::Fn,
            TokenType::For,
            TokenType::If,
            TokenType::Import,
            TokenType::Let,
            TokenType::Loop,
            TokenType::Main,
            TokenType::Match,
            TokenType::Nil,
            TokenType::Return,
            TokenType::Struct,
            TokenType::True,
            TokenType::Void,
            TokenType::While,
            TokenType::U8,
            TokenType::U16,
            TokenType::U32,
            TokenType::U64,
            TokenType::I8,
            TokenType::I16,
            TokenType::I32,
            TokenType::I64,
            TokenType::F32,
            TokenType::F64,
            TokenType::StringType,
            TokenType::Eof,
        ];

        test_tokentype_equality(expected_types, scanned);
        delete_test_file("keyword.test");
    }

    #[test]
    fn symbol_tokenization_test() {
        create_test_file("symbol.test", "([{}]) . .. ..=, -> : := :: ?%! != = == > >= >> >>= < <= << <<= && & | || - -= + += / /=  // //= * *= ^ ^=");

        let scanned: Vec<Arc<Token>> = scan_file("symbol.test");
        let expected_types = vec![
            TokenType::LeftParen,
            TokenType::LeftSquare,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::RightSquare,
            TokenType::RightParen,
            TokenType::Dot,
            TokenType::Iter,
            TokenType::IterEqual,
            TokenType::Comma,
            TokenType::Arrow,
            TokenType::Colon,
            TokenType::DynamicDefinition,
            TokenType::StaticScopeGetter,
            TokenType::Question,
            TokenType::Mod,
            TokenType::Not,
            TokenType::NotEqual,
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::ShiftRight,
            TokenType::ShiftRightEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::ShiftLeft,
            TokenType::ShiftLeftEqual,
            TokenType::And,
            TokenType::BitAnd,
            TokenType::BitOr,
            TokenType::Or,
            TokenType::Minus,
            TokenType::MinusEquals,
            TokenType::Plus,
            TokenType::PlusEquals,
            TokenType::Slash,
            TokenType::SlashEquals,
            TokenType::IntegerSlash,
            TokenType::IntegerSlashEquals,
            TokenType::Star,
            TokenType::StarEquals,
            TokenType::Power,
            TokenType::PowerEquals,
            TokenType::Eof,
        ];

        test_tokentype_equality(expected_types, scanned);
        delete_test_file("symbol.test");
    }
}
