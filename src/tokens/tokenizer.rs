use std::{cell::RefCell, rc::Rc};

use super::{error::Error, source::Source, token::Token, token_type::TokenType};

#[allow(dead_code)]
pub struct Tokenizer {
    current: Rc<RefCell<Source>>,
    files: Vec<Rc<RefCell<Source>>>,
}

#[allow(dead_code)]
impl Tokenizer {
    pub fn new(source: &str) -> Result<Self, Error> {
        let file = Rc::new(RefCell::new(Source::new(source)?));

        Ok(Self {
            current: Rc::clone(&file),
            files: vec![Rc::clone(&file)],
        })
    }

    pub fn scan<'a>(&'a mut self, new_file: &'a str) -> Result<(), Error> {
        let new_file = Rc::new(RefCell::new(Source::new(new_file)?));

        self.files.push(Rc::clone(&new_file));
        self.current = Rc::clone(&new_file);
        Ok(())
    }

    pub fn get_token(&mut self) -> Token {
        self.skip_whitespace();
        let _ch = self.advance();

        Token::new(
            self.current.borrow().line,
            self.current.borrow().column,
            TokenType::Eof,
            "",
        )
    }

    fn skip_whitespace(&mut self) {
        while !self.is_finished() {
            match self.current.clone().borrow().peek() {
                b' ' | b'\t' => self.advance(),
                _ => return,
            };
        }
    }

    fn advance(&mut self) -> u8 {
        let peek = self.current.borrow().peek();

        if peek != 0x00 {
            self.current.borrow_mut().update_line(|val: usize| -> usize { val + 1 });
            self.current.borrow_mut().update_column(|val: usize| -> usize { val + 1 });
        }

        peek
    }

    fn is_finished(&mut self) -> bool {
        if !self.current.borrow().is_at_eof() {
            false
        } else if self.files.len() > 1 {
            self.files.pop();
            self.current = Rc::clone(self.files.last().as_ref().unwrap());

            false
        } else {
            true
        }
    }
}
