use crate::tokens::token::Token;

use super::{datatypes::DataType, scopebound_statements::ScopeBoundStatement};

#[derive(Debug, Eq, PartialEq)]
pub struct Variable {
    _name: Box<Token>,
    _datatype: Option<DataType>,
    _value: Box<ScopeBoundStatement>,
}

impl Variable {
    pub fn new(name: Box<Token>, datatype: Option<DataType>, value: Box<ScopeBoundStatement>) -> Self {
        Self {
            _name: name,
            _datatype: datatype,
            _value: value,
        }
    }
}
