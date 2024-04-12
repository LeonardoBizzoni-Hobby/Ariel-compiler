use super::{source::Source, token::Token, token_type::TokenType};
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
            ("foreach".to_owned(), TokenType::ForEach),
            ("if".to_owned(), TokenType::If),
            ("in".to_owned(), TokenType::In),
            ("import".to_owned(), TokenType::Import),
            ("let".to_owned(), TokenType::Let),
            ("loop".to_owned(), TokenType::Loop),
            ("main".to_owned(), TokenType::Main),
            ("match".to_owned(), TokenType::Match),
            ("nil".to_owned(), TokenType::Nil),
            ("namespace".to_owned(), TokenType::Namespace),
            ("pub".to_owned(), TokenType::Public),
            ("return".to_owned(), TokenType::Return),
            ("struct".to_owned(), TokenType::Struct),
            ("template".to_owned(), TokenType::Template),
            ("this".to_owned(), TokenType::This),
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

pub fn get_token(source: &mut Source) -> Arc<Token> {
    skip_whitespace(source);

    source.start = source.current;

    match advance(source) {
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
                    b'=' => make_token(TokenType::IterEqual, source),
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
                make_token(TokenType::BangEqual, source)
            }
            _ => make_token(TokenType::Bang, source),
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
                make_token(TokenType::IntegerSlash, source)
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
        b'^' => {
            advance(source);
            match source.peek() {
                b'=' => {
                    advance(source);
                    make_token(TokenType::PowerEquals, source)
                }
                _ => make_token(TokenType::Power, source),
            }
        }
        0x00 => make_token(TokenType::Eof, source),
        ch => make_token(TokenType::Unknown(ch as char), source),
    }
}

fn make_token(ttype: TokenType, source: &mut Source) -> Arc<Token> {
    let start: usize = source.start;
    let current: usize = source.current;

    let binding: Vec<u8> = source.mmap[start..current].to_vec();
    let lexeme = String::from_utf8_lossy(&binding);

    Arc::new(Token::new(
        source.line,
        source.column - lexeme.len(),
        ttype,
        lexeme.to_string(),
        source.name.clone(),
    ))
}

fn make_number_token(source: &mut Source) -> Arc<Token> {
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

fn make_string_token(source: &mut Source) -> Arc<Token> {
    let mut lexeme = String::new();
    while source.peek() != b'"' {
        if source.peek() == b'\\' {
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
                        source
                    )
                }
            }
        } else {
            lexeme += &String::from_utf8(vec![advance(source)]).unwrap();
        }
    }

    if source.is_at_eof() {
        make_token_from(
            TokenType::Unknown(source.peek() as char),
            "Unterminated string.",
            source,
        )
    } else {
        advance(source);
        make_token_from(TokenType::String, &lexeme, source)
    }
}

fn make_identifier_token(source: &mut Source) -> Arc<Token> {
    while source.peek().is_ascii_alphanumeric() || source.peek() == b'_' {
        advance(source);
    }

    let start: usize = source.start;
    let current: usize = source.current;
    let binding: Vec<u8> = source.mmap[start..current].to_vec();
    let id = String::from_utf8_lossy(&binding);

    (|lexeme: &str| -> Arc<Token> {
        if let Some(token_type) = KEYWORD.get(lexeme) {
            make_token(token_type.clone(), source)
        } else {
            make_token(TokenType::Identifier, source)
        }
    })(&id)
}

fn make_token_from(ttype: TokenType, lexeme: &str, source: &mut Source) -> Arc<Token> {
    Arc::new(Token::new(
        source.line,
        {
            match ttype {
                TokenType::Unknown(ch) => source.column - ch.len_utf8(),
                _ => source.column - lexeme.len(),
            }
        },
        ttype,
        lexeme.to_string(),
        source.name.clone(),
    ))
}

fn skip_whitespace(source: &mut Source) {
    while !source.is_at_eof() {
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
                if source.peek_next() == b'#' {
                    advance(source);

                    while !(source.peek() == b'#' && source.peek_next() == b'#') {
                        if matches!(advance(source), b'\n' | b'\r') {
                            source.line += 1;
                            source.column = 0;
                        }
                    }

                    advance(source);
                    advance(source);
                } else {
                    while source.peek() != b'\n' {
                        advance(source);
                    }
                }
            }
            _ => return,
        };
    }
}

fn advance(source: &mut Source) -> u8 {
    let peek = source.peek();

    if peek != 0x00 {
        source.current += 1;
        source.column += 1;
        peek
    } else {
        peek
    }
}


// #[allow(dead_code)]
// pub struct Tokenizer {
//     file: Source,
// }

// #[allow(dead_code)]
// impl Tokenizer {
//     pub fn new(source: &str) -> Result<Self, Error> {
//         Ok(Self {
//             file: Source::new(source)?,
//         })
//     }

//     pub fn get_all_tokens(&mut self, source: &mut Source) -> Vec<Arc<Token>> {
//         let mut res: Vec<Arc<Token>> = vec![];

//         loop {
//             let tk = self.get_token(source);
//             res.push(tk.clone());

//             if tk.ttype == TokenType::Eof {
//                 return res;
//             }
//         }
//     }

//     pub fn get_token(&mut self, source: &mut Source) -> Arc<Token> {
//         self.skip_whitespace(source);

//         source.start = source.current;

//         match self.advance(source) {
//             b'"' => self.make_string_token(source),
//             b'0'..=b'9' => self.make_number_token(source),
//             b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.make_identifier_token(source),
//             b'(' => self.make_token(TokenType::LeftParen, source),
//             b')' => self.make_token(TokenType::RightParen, source),
//             b'[' => self.make_token(TokenType::LeftSquare, source),
//             b']' => self.make_token(TokenType::RightSquare, source),
//             b'{' => self.make_token(TokenType::LeftBrace, source),
//             b'}' => self.make_token(TokenType::RightBrace, source),
//             b'?' => self.make_token(TokenType::Question, source),
//             b',' => self.make_token(TokenType::Comma, source),
//             b';' => self.make_token(TokenType::Semicolon, source),
//             b'%' => self.make_token(TokenType::Mod, source),
//             b'.' => match source.peek() {
//                 b'.' => {
//                     self.advance(source);
//                     match source.peek() {
//                         b'=' => self.make_token(TokenType::IterEqual, source),
//                         _ => self.make_token(TokenType::Iter, source),
//                     }
//                 }
//                 _ => self.make_token(TokenType::Dot, source),
//             },
//             b':' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::DynamicDefinition, source)
//                 }
//                 b':' => {
//                     self.advance(source);
//                     self.make_token(TokenType::StaticScopeGetter, source)
//                 }
//                 _ => self.make_token(TokenType::Colon, source),
//             },
//             b'!' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::BangEqual, source)
//                 }
//                 _ => self.make_token(TokenType::Bang, source),
//             },
//             b'=' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::EqualEqual, source)
//                 }
//                 _ => self.make_token(TokenType::Equal, source),
//             },
//             b'>' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::GreaterEqual, source)
//                 }
//                 b'>' => {
//                     self.advance(source);
//                     match source.peek() {
//                         b'=' => {
//                             self.advance(source);
//                             self.make_token(TokenType::ShiftRightEqual, source)
//                         }
//                         _ => self.make_token(TokenType::ShiftRight, source),
//                     }
//                 }
//                 _ => self.make_token(TokenType::Greater, source),
//             },
//             b'<' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::LessEqual, source)
//                 }
//                 b'<' => {
//                     self.advance(source);
//                     match source.peek() {
//                         b'=' => {
//                             self.advance(source);
//                             self.make_token(TokenType::ShiftLeftEqual, source)
//                         }
//                         _ => self.make_token(TokenType::ShiftLeft, source),
//                     }
//                 }
//                 _ => self.make_token(TokenType::Less, source),
//             },
//             b'&' => match source.peek() {
//                 b'&' => {
//                     self.advance(source);
//                     self.make_token(TokenType::And, source)
//                 }
//                 _ => self.make_token(TokenType::BitAnd, source),
//             },
//             b'|' => match source.peek() {
//                 b'|' => {
//                     self.advance(source);
//                     self.make_token(TokenType::Or, source)
//                 }
//                 _ => self.make_token(TokenType::BitOr, source),
//             },
//             b'-' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::MinusEquals, source)
//                 }
//                 b'>' => {
//                     self.advance(source);
//                     self.make_token(TokenType::Arrow, source)
//                 }
//                 _ => self.make_token(TokenType::Minus, source),
//             },
//             b'+' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::PlusEquals, source)
//                 }
//                 _ => self.make_token(TokenType::Plus, source),
//             },
//             b'/' => match source.peek() {
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::SlashEquals, source)
//                 }
//                 b'/' => {
//                     self.advance(source);
//                     self.make_token(TokenType::IntegerSlash, source)
//                 }
//                 _ => self.make_token(TokenType::Slash, source),
//             },
//             b'*' => match source.peek() {
//                 b'*' => {
//                     self.advance(source);
//                     match source.peek() {
//                         b'=' => {
//                             self.advance(source);
//                             self.make_token(TokenType::PowerEquals, source)
//                         }
//                         _ => self.make_token(TokenType::Power, source),
//                     }
//                 }
//                 b'=' => {
//                     self.advance(source);
//                     self.make_token(TokenType::StarEquals, source)
//                 }
//                 _ => self.make_token(TokenType::Star, source),
//             },
//             0x00 => self.make_token(TokenType::Eof, source),
//             ch => self.make_token(TokenType::Unknown(ch as char), source),
//         }
//     }

//     fn skip_whitespace(&mut self, source: &mut Source) {
//         while !source.is_at_eof() {
//             match source.peek() {
//                 b'\n' | b'\r' => {
//                     self.advance(source);

//                     source.line += 1;
//                     source.column = 0;
//                 }
//                 b' ' | b'\t' => {
//                     self.advance(source);
//                 }
//                 b'#' => {
//                     if source.peek_next() == b'#' {
//                         self.advance(source);

//                         while !(source.peek() == b'#' && source.peek_next() == b'#') {
//                             if matches!(self.advance(source), b'\n' | b'\r') {
//                                 source.line += 1;
//                                 source.column = 0;
//                             }
//                         }

//                         self.advance(source);
//                         self.advance(source);
//                     } else {
//                         while source.peek() != b'\n' {
//                             self.advance(source);
//                         }
//                     }
//                 }
//                 _ => return,
//             };
//         }
//     }

//     fn advance(&mut self, source: &mut Source) -> u8 {
//         let peek = source.peek();

//         if peek != 0x00 {
//             source.current += 1;
//             source.column += 1;
//             peek
//         } else {
//             peek
//         }
//     }

//     fn make_token(&self, ttype: TokenType, source: &mut Source) -> Arc<Token> {
//         let start: usize = source.start;
//         let current: usize = source.current;

//         let binding: Vec<u8> = source.mmap[start..current].to_vec();
//         let lexeme = String::from_utf8_lossy(&binding);

//         Arc::new(Token::new(
//             source.line,
//             source.column - lexeme.len(),
//             ttype,
//             lexeme.to_string(),
//             source.name.clone(),
//         ))
//     }

//     fn make_token_from(&self, ttype: TokenType, lexeme: &str) -> Arc<Token> {
//         Arc::new(Token::new(
//             self.file.line,
//             {
//                 match ttype {
//                     TokenType::Unknown(ch) => self.file.column - ch.len_utf8(),
//                     _ => self.file.column - lexeme.len(),
//                 }
//             },
//             ttype,
//             lexeme.to_string(),
//             self.file.name.clone(),
//         ))
//     }

//     fn make_number_token(&mut self, source: &mut Source) -> Arc<Token> {
//         while self.file.peek().is_ascii_digit() {
//             self.advance(source);
//         }

//         if self.file.peek() == b'.' && self.file.peek_next().is_ascii_digit() {
//             self.advance(source);

//             while self.file.peek().is_ascii_digit() {
//                 self.advance(source);
//             }

//             self.make_token(TokenType::Double, source)
//         } else {
//             self.make_token(TokenType::Integer, source)
//         }
//     }

//     fn make_string_token(&mut self, source: &mut Source) -> Arc<Token> {
//         let mut lexeme = String::new();
//         while source.peek() != b'"' {
//             if source.peek() == b'\\' {
//                 self.advance(source);
//                 match self.advance(source) {
//                     b'n' => lexeme += "\n",
//                     b't' => lexeme += "\t",
//                     b'r' => lexeme += "\r",
//                     b'\'' => lexeme += "\'",
//                     b'"' => lexeme += "\"",
//                     b'\\' => lexeme += "\\",
//                     ch => {
//                         return self.make_token_from(
//                             TokenType::Unknown(ch as char),
//                             "Invalid escape sequence.",
//                         )
//                     }
//                 }
//             } else {
//                 lexeme += &String::from_utf8(vec![self.advance(source)]).unwrap();
//             }
//         }

//         if source.is_at_eof() {
//             self.make_token_from(
//                 TokenType::Unknown(source.peek() as char),
//                 "Unterminated string.",
//             )
//         } else {
//             self.advance(source);
//             self.make_token_from(TokenType::String, &lexeme)
//         }
//     }

//     fn make_identifier_token(&mut self, source: &mut Source) -> Arc<Token> {
//         while source.peek().is_ascii_alphanumeric() || source.peek() == b'_' {
//             self.advance(source);
//         }

//         let start: usize = source.start;
//         let current: usize = source.current;
//         let binding: Vec<u8> = source.mmap[start..current].to_vec();
//         let id = String::from_utf8_lossy(&binding);

//         (|lexeme: &str| -> Arc<Token> {
//             if let Some(token_type) = KEYWORD.get(lexeme) {
//                 self.make_token(token_type.clone(), source)
//             } else {
//                 self.make_token(TokenType::Identifier, source)
//             }
//         })(&id)
//     }
// }

// #[allow(unused_imports)]
// mod tests {
//     use crate::tokens::{token::Token, token_type::TokenType, tokenizer::Tokenizer};
//     use std::{fs::File as StdFile, io::Write, sync::Arc};

//     #[test]
//     fn single_file_tokenization() {
//         let _ = StdFile::create("single.file")
//             .unwrap()
//             .write_all(b"a b c d e");

//         let mut lexer =
//             Tokenizer::new("single.file").expect("Found error variant in Tokenizer init.");

//         let scanned = lexer.get_all_tokens();
//         let expected = vec![
//             Arc::new(Token::new(
//                 1,
//                 0,
//                 TokenType::Identifier,
//                 "a".to_string(),
//                 "single.file".to_string(),
//             )),
//             Arc::new(Token::new(
//                 1,
//                 2,
//                 TokenType::Identifier,
//                 "b".to_string(),
//                 "single.file".to_string(),
//             )),
//             Arc::new(Token::new(
//                 1,
//                 4,
//                 TokenType::Identifier,
//                 "c".to_string(),
//                 "single.file".to_string(),
//             )),
//             Arc::new(Token::new(
//                 1,
//                 6,
//                 TokenType::Identifier,
//                 "d".to_string(),
//                 "single.file".to_string(),
//             )),
//             Arc::new(Token::new(
//                 1,
//                 8,
//                 TokenType::Identifier,
//                 "e".to_string(),
//                 "single.file".to_string(),
//             )),
//             Arc::new(Token::new(
//                 1,
//                 9,
//                 TokenType::Eof,
//                 "".to_string(),
//                 "single.file".to_string(),
//             )),
//         ];

//         assert_eq!(expected.len(), scanned.len());
//         for x in 0..scanned.len() {
//             assert_eq!(expected[x], scanned[x]);
//         }

//         std::fs::remove_file("single.file").unwrap();
//     }

//     #[test]
//     fn keyword_tokenization_test() {
//         StdFile::create("keyword.test")
//             .unwrap()
//             .write_all(
//                 b"_
// break
// continue
// else
// enum
// false
// fn
// for
// foreach
// if
// in
// import
// let
// loop
// main
// match
// nil
// namespace
// pub
// return
// struct
// super
// template
// this
// true
// void
// while
// u8
// u16
// u32
// u64
// i8
// i16
// i32
// i64
// f32
// f64
// str",
//             )
//             .unwrap();
//         let expected_types = vec![
//             TokenType::DontCare,
//             TokenType::Break,
//             TokenType::Continue,
//             TokenType::Else,
//             TokenType::Enum,
//             TokenType::False,
//             TokenType::Fn,
//             TokenType::For,
//             TokenType::ForEach,
//             TokenType::If,
//             TokenType::In,
//             TokenType::Import,
//             TokenType::Let,
//             TokenType::Loop,
//             TokenType::Main,
//             TokenType::Match,
//             TokenType::Nil,
//             TokenType::Namespace,
//             TokenType::Public,
//             TokenType::Return,
//             TokenType::Struct,
//             TokenType::Super,
//             TokenType::Template,
//             TokenType::This,
//             TokenType::True,
//             TokenType::Void,
//             TokenType::While,
//             TokenType::U8,
//             TokenType::U16,
//             TokenType::U32,
//             TokenType::U64,
//             TokenType::I8,
//             TokenType::I16,
//             TokenType::I32,
//             TokenType::I64,
//             TokenType::F32,
//             TokenType::F64,
//             TokenType::StringType,
//             TokenType::Eof,
//         ];

//         let mut lexer =
//             Tokenizer::new("keyword.test").expect("Found error variant in Tokenizer init.");

//         assert_eq!(
//             expected_types,
//             lexer
//                 .get_all_tokens()
//                 .iter()
//                 .map(|tk| tk.ttype.clone())
//                 .collect::<Vec<TokenType>>()
//         );

//         std::fs::remove_file("keyword.test").unwrap();
//     }
// }
