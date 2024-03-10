use lazy_static::lazy_static;
use std::{collections::HashMap, rc::Rc};

use super::{error::Error, source::Source, token::Token, token_type::TokenType};

macro_rules! last_mut {
    ($vec:expr) => {
        $vec.last_mut().expect("Vector is empty")
    };
}

macro_rules! last {
    ($vec:expr) => {
        $vec.last().expect("Vector is empty")
    };
}

#[allow(dead_code)]
pub struct Tokenizer<'lexer> {
    files: Vec<Source<'lexer>>,
}

#[allow(dead_code)]
impl<'lexer> Tokenizer<'lexer> {
    pub fn new(source: &'lexer str) -> Result<Self, Error> {
        Ok(Self {
            files: vec![Source::new(source)?],
        })
    }

    pub fn scan(&mut self, new_file: &'lexer str) -> Result<(), Error> {
        self.files.push(Source::new(new_file)?);
        Ok(())
    }

    pub fn get_all_tokens(&mut self) -> Vec<Rc<Token<'lexer>>> {
        let mut res: Vec<Rc<Token>> = vec![];

        loop {
            let tk = self.get_token();
            res.push(Rc::clone(&tk));

            if tk.ttype == TokenType::Eof {
                return res;
            }
        }
    }

    pub fn get_token(&mut self) -> Rc<Token<'lexer>> {
        self.skip_whitespace();

        {
            // Scope needed to delete this ↓ mut borrow.
            let current = last_mut!(self.files);
            current.start = current.current;
        }

        let ch: u8 = self.advance();

        match ch {
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
            b'.' => match last!(self.files).peek() {
                b'.' => {
                    self.advance();
                    match last!(self.files).peek() {
                        b'=' => self.make_token(TokenType::IterEqual),
                        _ => self.make_token(TokenType::Iter),
                    }
                },
                _ => self.make_token(TokenType::Dot),
            },
            b':' => match last!(self.files).peek() {
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
            b'!' => match last!(self.files).peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::BangEqual)
                }
                _ => self.make_token(TokenType::Bang),
            },
            b'=' => match last!(self.files).peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::EqualEqual)
                }
                _ => self.make_token(TokenType::Equal),
            },
            b'>' => match last!(self.files).peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::GreaterEqual)
                }
                b'>' => {
                    self.advance();
                    match last!(self.files).peek() {
                        b'=' => {
                            self.advance();
                            self.make_token(TokenType::ShiftRightEqual)
                        }
                        _ => self.make_token(TokenType::ShiftRight),
                    }
                }
                _ => self.make_token(TokenType::Greater),
            },
            b'<' => match last!(self.files).peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::LessEqual)
                }
                b'<' => {
                    self.advance();
                    match last!(self.files).peek() {
                        b'=' => {
                            self.advance();
                            self.make_token(TokenType::ShiftLeftEqual)
                        }
                        _ => self.make_token(TokenType::ShiftLeft),
                    }
                }
                _ => self.make_token(TokenType::Less),
            },
            b'&' => match last!(self.files).peek() {
                b'&' => {
                    self.advance();
                    self.make_token(TokenType::And)
                }
                _ => self.make_token(TokenType::BitAnd),
            },
            b'|' => match last!(self.files).peek() {
                b'|' => {
                    self.advance();
                    self.make_token(TokenType::Or)
                }
                _ => self.make_token(TokenType::BitOr),
            },
            b'-' => match last!(self.files).peek() {
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
            b'+' => match last!(self.files).peek() {
                b'=' => {
                    self.advance();
                    self.make_token(TokenType::PlusEquals)
                }
                _ => self.make_token(TokenType::Plus),
            },
            b'/' => match last!(self.files).peek() {
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
            b'*' => match last!(self.files).peek() {
                b'*' => {
                    self.advance();
                    match last!(self.files).peek() {
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
            _ => self.make_token(TokenType::Unknown(ch as char)),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_finished() {
            let current = last!(self.files);

            match current.peek() {
                b'\n' | b'\r' => {
                    self.advance();

                    let current = last_mut!(self.files);
                    current.line += 1;
                    current.column = 0;
                }
                b' ' | b'\t' => {
                    self.advance();
                }
                b'#' => {
                    if last!(self.files).peek_ahead(1) == b'#' {
                        self.advance();

                        while {
                            let current = last!(self.files);
                            !current.is_at_eof()
                                && !(current.peek() == b'#' && current.peek_ahead(1) == b'#')
                        } {
                            if matches!(self.advance(), b'\n' | b'\r') {
                                let current = last_mut!(self.files);
                                current.line += 1;
                                current.column = 0;
                            }
                        }

                        self.advance();
                        self.advance();
                    } else {
                        while {
                            let curr = last!(self.files);
                            !curr.is_at_eof() && curr.peek() != b'\n'
                        } {
                            self.advance();
                        }
                    }
                }
                _ => return,
            };
        }
    }

    fn advance(&mut self) -> u8 {
        let current = last_mut!(self.files);
        let peek = current.peek();

        if peek != 0x00 {
            current.current += 1;
            current.column += 1;
            peek
        } else if !self.is_finished() {
            self.advance()
        } else {
            peek
        }
    }

    fn is_finished(&mut self) -> bool {
        if self.files.len() == 0 {
            true
        } else if !last!(self.files).is_at_eof() {
            false
        } else if self.files.len() > 1 {
            self.files.pop();

            false
        } else {
            true
        }
    }

    fn make_token(&self, ttype: TokenType) -> Rc<Token<'lexer>> {
        let current_file = last!(self.files);
        let start: usize = current_file.start;
        let current: usize = current_file.current;

        let lexeme = String::from_utf8(current_file.mmap[start..current].to_vec()).unwrap();

        Rc::new(Token::new(
            current_file.line,
            current_file.column - lexeme.len(),
            ttype,
            lexeme,
            &current_file.name,
        ))
    }

    fn make_token_from(&self, ttype: TokenType, lexeme: &str) -> Rc<Token<'lexer>> {
        let current_file = last!(self.files);

        Rc::new(Token::new(
            current_file.line,
            {
                match ttype {
                    TokenType::Unknown(ch) => current_file.column - ch.len_utf8(),
                    _ => current_file.column - lexeme.len(),
                }
            },
            ttype,
            lexeme.to_owned(),
            &current_file.name,
        ))
    }

    fn make_number_token(&mut self) -> Rc<Token<'lexer>> {
        while {
            let curr = last!(self.files);
            !curr.is_at_eof() && curr.peek().is_ascii_digit()
        } {
            self.advance();
        }

        if {
            let curr = last!(self.files);
            !curr.is_at_eof() && curr.peek() == b'.' && curr.peek_ahead(1).is_ascii_digit()
        } {
            self.advance();

            while {
                let curr = last!(self.files);
                !curr.is_at_eof() && curr.peek().is_ascii_digit()
            } {
                self.advance();
            }

            self.make_token(TokenType::Double)
        } else {
            self.make_token(TokenType::Integer)
        }
    }

    fn make_string_token(&mut self) -> Rc<Token<'lexer>> {
        let mut lexeme = String::new();
        while {
            let current = last!(self.files);
            !current.is_at_eof() && current.peek() != b'"'
        } {
            if last!(self.files).peek() == b'\\' {
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

        if last!(self.files).is_at_eof() {
            self.make_token_from(
                TokenType::Unknown(last!(self.files).peek() as char),
                "Unterminated string.",
            )
        } else {
            self.advance();
            self.make_token_from(TokenType::String, &lexeme)
        }
    }

    fn make_identifier_token(&mut self) -> Rc<Token<'lexer>> {
        while {
            let current = last!(self.files);
            !current.is_at_eof() && current.peek().is_ascii_alphanumeric()
        } {
            self.advance();
        }

        let start: usize = last!(self.files).start;
        let current: usize = last!(self.files).current;
        let binding: Vec<u8> = last!(self.files).mmap[start..current].to_vec();
        let id = String::from_utf8_lossy(&binding);

        (|lexeme: &str| -> Rc<Token<'lexer>> {
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
