use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Struct {
    _name: Box<Token>,
    _fields: Vec<(Box<Token>, DataType)>,
}

impl Struct {
    pub fn new(name: Box<Token>, fields: Vec<(Box<Token>, DataType)>) -> Self {
        Self { _name: name, _fields: fields }
    }
}
