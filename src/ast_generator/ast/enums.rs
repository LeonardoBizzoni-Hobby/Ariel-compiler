use std::collections::HashMap;

use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Enum {
    _name: Box<Token>,
    _variants: HashMap<Box<Token>, Option<DataType>>,
}

impl Enum {
    pub fn new(name: Box<Token>, variants: HashMap<Box<Token>, Option<DataType>>) -> Self {
        Self {
            _name: name,
            _variants: variants,
        }
    }
}
