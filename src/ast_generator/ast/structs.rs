use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Struct {
    pub name: Box<Token>,
    pub fields: Vec<(Box<Token>, DataType)>,
}

impl Struct {
    pub fn new(name: Box<Token>, fields: Vec<(Box<Token>, DataType)>) -> Self {
        Self { name, fields }
    }
}
