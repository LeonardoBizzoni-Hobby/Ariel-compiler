use super::token_type::TokenType;

#[derive(Debug)]
pub struct Token<'lexer> {
    line: usize,
    column: usize,
    ttype: TokenType,
    lexeme: String,
    found_in: &'lexer str,
}

impl<'lexer> Token<'lexer> {
    pub fn new(line: usize, column: usize, ttype: TokenType, lexeme: String, found_in: &'lexer str) -> Self {
        Self {
            line,
            column,
            ttype,
            lexeme,
            found_in,
        }
    }
}
