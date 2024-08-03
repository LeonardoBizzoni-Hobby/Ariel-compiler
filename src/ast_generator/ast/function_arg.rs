use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Argument {
    pub name: Box<Token>,
    pub arg_type: DataType,
}
