use std::sync::Arc;

use crate::tokens::token::Token;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FunctionField {
    name: Arc<Token>,
    ftype: Arc<Token>,
}

#[allow(dead_code)]
impl FunctionField {
    pub fn new(name: Arc<Token>, ftype: Arc<Token>) -> Self {
        Self { name, ftype }
    }
}
