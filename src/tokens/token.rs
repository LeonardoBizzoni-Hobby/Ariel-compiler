use super::token_type::TokenType;

pub struct Token<'ctx> {
    line: usize,
    column: usize,
    ttype: TokenType,
    lexeme: &'ctx str,
}

impl<'ctx> Token<'ctx> {
    pub fn new(line: usize, column: usize, ttype: TokenType, lexeme: &'ctx str) -> Self {
        Self {
            line,
            column,
            ttype,
            lexeme,
        }
    }
}
