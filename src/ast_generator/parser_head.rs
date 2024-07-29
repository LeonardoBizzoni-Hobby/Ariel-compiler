use std::mem;

use crate::tokens::{
    error::ParseError, source::SourceFile, token::Token, token_type::TokenType, tokenizer,
};

pub struct ParserHead<'a> {
    pub curr: Box<Token>,
    pub prev: Box<Token>,
    pub source: &'a mut SourceFile,
}

impl<'a> ParserHead<'a> {
    pub fn new(curr: Box<Token>, prev: Box<Token>, source: &'a mut SourceFile) -> Self {
        Self { curr, prev, source }
    }

    #[inline]
    pub fn advance(&mut self) {
        self.prev = mem::replace(&mut self.curr, tokenizer::get_token(&mut self.source));
    }

    pub fn require_current_is(&mut self, expected: TokenType) -> Result<(), ParseError> {
        if (*self.curr).ttype == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                token: std::mem::take(&mut self.curr),
                expected,
                msg: None,
            })
        }
    }
}
