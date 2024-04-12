use std::sync::Arc;

use crate::tokens::token::Token;

use super::DataType;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FunctionField {
    name: Arc<Token>,
    ftype: DataType,
}

#[allow(dead_code)]
impl FunctionField {
    pub fn new(name: Arc<Token>, ftype: DataType) -> Self {
        Self { name, ftype }
    }
}
