use std::collections::HashMap;

use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Enum {
    pub name: Box<Token>,
    pub variants: HashMap<Box<Token>, Option<DataType>>,
}

impl Enum {
    pub fn new(name: Box<Token>, variants: HashMap<Box<Token>, Option<DataType>>) -> Self {
        Self { name, variants }
    }
}
