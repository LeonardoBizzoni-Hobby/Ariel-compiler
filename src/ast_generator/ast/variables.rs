use std::sync::Arc;

use crate::tokens::token::Token;

use super::scopebound_statements::ScopeBoundStatement;

#[derive(Debug, Eq, PartialEq)]
pub struct Variable {
    _name: Arc<Token>,
    _datatype: Option<DataType>,
    _value: Box<ScopeBoundStatement>,
}

impl Variable {
    pub fn new(name: Arc<Token>, datatype: Option<DataType>, value: Box<ScopeBoundStatement>) -> Self {
        Self {
            _name: name,
            _datatype: datatype,
            _value: value,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DataType {
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
    F32,
    F64,
    String,
    Bool,
    Void,
    Array(Box<DataType>),
    Pointer(Box<DataType>),
    Compound { name: Arc<Token> },
}
