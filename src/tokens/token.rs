use std::fmt::Display;

use super::token_type::TokenType;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub ttype: TokenType,
    pub lexeme: String,
    pub found_in: String,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            line: Default::default(),
            column: Default::default(),
            ttype: Default::default(),
            lexeme: Default::default(),
            found_in: Default::default(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

impl Token {
    pub fn new() -> Self {
        Self::default()
    }
}
