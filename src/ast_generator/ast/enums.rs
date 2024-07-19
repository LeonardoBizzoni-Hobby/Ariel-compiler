use std::{collections::HashMap, sync::Arc};

use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Enum {
    _name: Arc<Token>,
    _variants: HashMap<Arc<Token>, Option<DataType>>,
}

impl Enum {
    pub fn new(name: Arc<Token>, variants: HashMap<Arc<Token>, Option<DataType>>) -> Self {
        Self {
            _name: name,
            _variants: variants,
        }
    }
}
