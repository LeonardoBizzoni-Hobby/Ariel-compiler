use super::token_type::TokenType;

#[derive(Debug)]
pub enum Error {
    FileNotFound(String, String),
    MemoryMapFiled(String, String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    UnexpectedToken {
        line: usize,
        col: usize,
        found: TokenType,
        expected: TokenType,
        msg: Option<String>,
    },
    InvalidDataType {
        line: usize,
        col: usize,
        found: TokenType,
        msg: Option<String>,
    },
    InvalidVariableDeclaration {
        line: usize,
        column: usize,
    },
}
