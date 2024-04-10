use lazy_static::lazy_static;
use std::collections::HashMap;

use super::{error::Error, source::Source, token::Token, token_type::TokenType};

#[allow(dead_code)]
pub struct Tokenizer {
    file: Source,
}

#[allow(dead_code)]
impl Tokenizer {
    pub fn new(source: &str) -> Result<Self, Error> {
        Ok(Self {
            file: Source::new(source)?,
        })
    }

    #[cfg(test)]
    pub fn get_all_tokens(&mut self) -> Vec<Box<Token>> {
        let mut res: Vec<Box<Token>> = vec![];

        loop {
            res.push(self.get_token());

            if res.last().unwrap().ttype == TokenType::Eof {
                return res;
            }
        }
    }

    pub fn get_token(&mut self) -> Box<Token> {
        self.skip_whitespace();

        self.file.start = self.file.current;

        match self.advance() {
            b'"' => self.make_string_token(),
            b'0'..=b'9' => self.make_number_token(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.make_identifier_token(),
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'[' => self.make_token(TokenType::LeftSquare),
            b']' => self.make_token(TokenType::RightSquare),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b'?' => self.make_token(TokenType::Question),
            b',' => self.make_token(TokenType::Comma),
            b';' => self.make_token(TokenType::Semicolon),
            b'%' => self.make_token(TokenType::Mod),
            b'.' => match self.file.peek() {
                b'.' => {
                    self.advance();
                    match self.file.peek() {
                        b'=' => self.make_token(TokenType::IterEqual),
                        _ => self.make_token(TokenType::Iter),
                    }
                }
                _ => self.make_token(TokenType::Dot),
            },
            b':' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::DynamicDefinition)
                }
                b':' => {
                    self.advance();
                    self.make_token(TokenType::StaticScopeGetter)
                }
                _ => self.make_token(TokenType::Colon),
            },
            b'!' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::BangEqual)
                }
                _ => self.make_token(TokenType::Bang),
            },
            b'=' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::EqualEqual)
                }
                _ => self.make_token(TokenType::Equal),
            },
            b'>' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::GreaterEqual)
                }
                b'>' => {
                    self.advance();
                    match self.file.peek() {
                        b'=' => {
                            self.advance();
                            self.make_token(TokenType::ShiftRightEqual)
                        }
                        _ => self.make_token(TokenType::ShiftRight),
                    }
                }
                _ => self.make_token(TokenType::Greater),
            },
            b'<' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::LessEqual)
                }
                b'<' => {
                    self.advance();
                    match self.file.peek() {
                        b'=' => {
                            self.advance();
                            self.make_token(TokenType::ShiftLeftEqual)
                        }
                        _ => self.make_token(TokenType::ShiftLeft),
                    }
                }
                _ => self.make_token(TokenType::Less),
            },
            b'&' => match self.file.peek() {
                b'&' => {
                    self.advance();
                    self.make_token(TokenType::And)
                }
                _ => self.make_token(TokenType::BitAnd),
            },
            b'|' => match self.file.peek() {
                b'|' => {
                    self.advance();
                    self.make_token(TokenType::Or)
                }
                _ => self.make_token(TokenType::BitOr),
            },
            b'-' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::MinusEquals)
                }
                b'>' => {
                    self.advance();
                    self.make_token(TokenType::Arrow)
                }
                _ => self.make_token(TokenType::Minus),
            },
            b'+' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::PlusEquals)
                }
                _ => self.make_token(TokenType::Plus),
            },
            b'/' => match self.file.peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::SlashEquals)
                }
                b'/' => {
                    self.advance();
                    self.make_token(TokenType::IntegerSlash)
                }
                _ => self.make_token(TokenType::Slash),
            },
            b'*' => match self.file.peek() {
                b'*' => {
                    self.advance();
                    match self.file.peek() {
                        b'=' => {
                            self.advance();
                            self.make_token(TokenType::PowerEquals)
                        }
                        _ => self.make_token(TokenType::Power),
                    }
                }
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::StarEquals)
                }
                _ => self.make_token(TokenType::Star),
            },
            0x00 => self.make_token(TokenType::Eof),
            ch => self.make_token(TokenType::Unknown(ch as char)),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.file.is_at_eof() {
            match self.file.peek() {
                b'\n' | b'\r' => {
                    self.advance();

                    self.file.line += 1;
                    self.file.column = 0;
                }
                b' ' | b'\t' => {
                    self.advance();
                }
                b'#' => {
                    if self.file.peek_ahead(1) == b'#' {
                        self.advance();

                        while {
                            !self.file.is_at_eof()
                                && !(self.file.peek() == b'#' && self.file.peek_ahead(1) == b'#')
                        } {
                            if matches!(self.advance(), b'\n' | b'\r') {
                                self.file.line += 1;
                                self.file.column = 0;
                            }
                        }

                        self.advance();
                        self.advance();
                    } else {
                        while !self.file.is_at_eof() && self.file.peek() != b'\n' {
                            self.advance();
                        }
                    }
                }
                _ => return,
            };
        }
    }

    fn advance(&mut self) -> u8 {
        let peek = self.file.peek();

        if peek != 0x00 {
            self.file.current += 1;
            self.file.column += 1;
            peek
        } else {
            peek
        }
    }

    fn make_token(&self, ttype: TokenType) -> Box<Token> {
        let start: usize = self.file.start;
        let current: usize = self.file.current;

        let lexeme = String::from_utf8(self.file.mmap[start..current].to_vec()).unwrap();

        Box::new(Token::new(
            self.file.line,
            self.file.column - lexeme.len(),
            ttype,
            lexeme,
            self.file.name.clone(),
        ))
    }

    fn make_token_from(&self, ttype: TokenType, lexeme: &str) -> Box<Token> {
        Box::new(Token::new(
            self.file.line,
            {
                match ttype {
                    TokenType::Unknown(ch) => self.file.column - ch.len_utf8(),
                    _ => self.file.column - lexeme.len(),
                }
            },
            ttype,
            lexeme.to_owned(),
            self.file.name.clone(),
        ))
    }

    fn make_number_token(&mut self) -> Box<Token> {
        while !self.file.is_at_eof() && self.file.peek().is_ascii_digit() {
            self.advance();
        }

        if {
            !self.file.is_at_eof()
                && self.file.peek() == b'.'
                && self.file.peek_ahead(1).is_ascii_digit()
        } {
            self.advance();

            while !self.file.is_at_eof() && self.file.peek().is_ascii_digit() {
                self.advance();
            }

            self.make_token(TokenType::Double)
        } else {
            self.make_token(TokenType::Integer)
        }
    }

    fn make_string_token(&mut self) -> Box<Token> {
        let mut lexeme = String::new();
        while !self.file.is_at_eof() && self.file.peek() != b'"' {
            if self.file.peek() == b'\\' {
                self.advance();
                match self.advance() {
                    b'n' => lexeme += "\n",
                    b't' => lexeme += "\t",
                    b'r' => lexeme += "\r",
                    b'\'' => lexeme += "\'",
                    b'"' => lexeme += "\"",
                    b'\\' => lexeme += "\\",
                    ch => {
                        return self.make_token_from(
                            TokenType::Unknown(ch as char),
                            "Invalid escape sequence.",
                        )
                    }
                }
            } else {
                lexeme += &String::from_utf8(vec![self.advance()]).unwrap();
            }
        }

        if self.file.is_at_eof() {
            self.make_token_from(
                TokenType::Unknown(self.file.peek() as char),
                "Unterminated string.",
            )
        } else {
            self.advance();
            self.make_token_from(TokenType::String, &lexeme)
        }
    }

    fn make_identifier_token(&mut self) -> Box<Token> {
        while !self.file.is_at_eof() && self.file.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        let start: usize = self.file.start;
        let current: usize = self.file.current;
        let binding: Vec<u8> = self.file.mmap[start..current].to_vec();
        let id = String::from_utf8_lossy(&binding);

        (|lexeme: &str| -> Box<Token> {
            lazy_static! {
                static ref KEYWORD: HashMap<String, TokenType> = {
                    HashMap::from([
                        ("_".to_owned(), TokenType::DontCare),
                        ("break".to_owned(), TokenType::Break),
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
                        ("super".to_owned(), TokenType::Super),
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
            };

            if let Some(token_type) = KEYWORD.get(lexeme) {
                self.make_token(token_type.clone())
            } else {
                self.make_token(TokenType::Identifier)
            }
        })(&id)
    }
}

#[allow(unused_imports)]
mod tests {
    use crate::tokens::{token::Token, token_type::TokenType, tokenizer::Tokenizer};
    use std::{fs::File as StdFile, io::Write};

    #[test]
    fn single_file_tokenization() {
        let _ = StdFile::create("single.file")
            .unwrap()
            .write_all(b"a b c d e");

        let mut lexer =
            Tokenizer::new("single.file").expect("Found error variant in Tokenizer init.");

        let scanned = lexer.get_all_tokens();
        let expected = vec![
            Box::new(Token::new(
                1,
                0,
                TokenType::Identifier,
                "a".to_string(),
                "single.file".to_owned(),
            )),
            Box::new(Token::new(
                1,
                2,
                TokenType::Identifier,
                "b".to_string(),
                "single.file".to_owned(),
            )),
            Box::new(Token::new(
                1,
                4,
                TokenType::Identifier,
                "c".to_string(),
                "single.file".to_owned(),
            )),
            Box::new(Token::new(
                1,
                6,
                TokenType::Identifier,
                "d".to_string(),
                "single.file".to_owned(),
            )),
            Box::new(Token::new(
                1,
                8,
                TokenType::Identifier,
                "e".to_string(),
                "single.file".to_owned(),
            )),
            Box::new(Token::new(
                1,
                9,
                TokenType::Eof,
                "".to_string(),
                "single.file".to_owned(),
            )),
        ];

        assert_eq!(expected.len(), scanned.len());
        for x in 0..scanned.len() {
            assert_eq!(expected[x], scanned[x]);
        }

        std::fs::remove_file("single.file").unwrap();
    }

    #[test]
    fn keyword_tokenization_test() {
        StdFile::create("keyword.test")
            .unwrap()
            .write_all(
                b"_
break
continue
else
enum
false
fn
for
foreach
if
in
import
let
loop
main
match
nil
namespace
pub
return
struct
super
template
this
true
void
while
u8
u16
u32
u64
i8
i16
i32
i64
f32
f64
str",
            )
            .unwrap();
        let expected_types = vec![
            TokenType::DontCare,
            TokenType::Break,
            TokenType::Continue,
            TokenType::Else,
            TokenType::Enum,
            TokenType::False,
            TokenType::Fn,
            TokenType::For,
            TokenType::ForEach,
            TokenType::If,
            TokenType::In,
            TokenType::Import,
            TokenType::Let,
            TokenType::Loop,
            TokenType::Main,
            TokenType::Match,
            TokenType::Nil,
            TokenType::Namespace,
            TokenType::Public,
            TokenType::Return,
            TokenType::Struct,
            TokenType::Super,
            TokenType::Template,
            TokenType::This,
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

        let mut lexer =
            Tokenizer::new("keyword.test").expect("Found error variant in Tokenizer init.");

        assert_eq!(
            expected_types,
            lexer
                .get_all_tokens()
                .iter()
                .map(|tk| tk.ttype.clone())
                .collect::<Vec<TokenType>>()
        );

        std::fs::remove_file("keyword.test").unwrap();
    }
}
