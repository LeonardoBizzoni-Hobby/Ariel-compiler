use std::sync::Arc;

use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Struct {
    _name: Arc<Token>,
    _fields: Vec<(Arc<Token>, DataType)>,
}

impl Struct {
    pub fn new(name: Arc<Token>, fields: Vec<(Arc<Token>, DataType)>) -> Self {
        Self { _name: name, _fields: fields }
    }
}
