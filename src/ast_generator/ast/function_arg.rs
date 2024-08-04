use crate::tokens::token::Token;

use super::datatypes::DataType;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Argument {
    pub name: Box<Token>,
    pub arg_type: DataType,
}
