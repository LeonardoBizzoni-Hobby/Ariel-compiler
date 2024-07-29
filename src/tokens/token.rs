use super::token_type::TokenType;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
            line: 0,
            column: 0,
            ttype: TokenType::Unknown('\0'),
            lexeme: "\0".to_string(),
            found_in: "\0".to_string(),
        }
    }
}

impl Token {
    pub fn new(
        line: usize,
        column: usize,
        ttype: TokenType,
        lexeme: String,
        found_in: String,
    ) -> Self {
        Self {
            line,
            column,
            ttype,
            lexeme,
            found_in,
        }
    }
}
