use std::sync::Arc;

use crate::tokens::token::Token;

use super::DataType;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Argument {
    name: Arc<Token>,
    datatype: DataType,
}

#[allow(dead_code)]
impl Argument {
    pub fn new(name: Arc<Token>, datatype: DataType) -> Self {
        Self { name, datatype }
    }
}
