use std::rc::Rc;

use super::{error::Error, source::Source, token::Token, token_type::TokenType};

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

    pub fn get_token(&mut self) -> Result<Rc<Token<'lexer>>, Error> {
        self.skip_whitespace();

        {
            // Scope needed to delete this ↓ mut borrow.
            let current = self.files.last_mut().unwrap();
            current.start = current.current;
        }

        let ch: u8 = self.advance();

        match ch {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'[' => self.make_token(TokenType::LeftSquare),
            b']' => self.make_token(TokenType::RightSquare),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b'?' => self.make_token(TokenType::Question),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b';' => self.make_token(TokenType::Semicolon),
            b'%' => self.make_token(TokenType::Mod),
            b':' => self.make_token(TokenType::Colon),
            0x00 => self.make_token(TokenType::Eof),
            _ => self.make_token(TokenType::Unknown(ch as char)),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_finished() {
            let current = self.files.last().unwrap();

            match current.peek() {
                b' ' | b'\t' => self.advance(),
                _ => return,
            };
        }
    }

    fn advance(&mut self) -> u8 {
        let current = self.files.last_mut().unwrap();
        let peek = current.peek();

        if peek != 0x00 {
            current.update_current();
            current.update_column();
            peek
        } else if !self.is_finished() {
            self.advance()
        } else {
            peek
        }
    }

    fn is_finished(&mut self) -> bool {
        if !self.files.last().unwrap().is_at_eof() {
            false
        } else if self.files.len() > 1 {
            self.files.pop();

            false
        } else {
            true
        }
    }

    fn make_token(&self, ttype: TokenType) -> Result<Rc<Token<'lexer>>, Error> {
        let current_file = self.files.last().unwrap();
        let start: usize = current_file.start;
        let current: usize = current_file.current;

        let lexeme = String::from_utf8(current_file.mmap[start..current].to_vec()).unwrap();

        Ok(Rc::new(Token::new(
            current_file.line,
            current_file.column - lexeme.len(),
            ttype,
            lexeme,
            &current_file.name,
        )))
    }
}
