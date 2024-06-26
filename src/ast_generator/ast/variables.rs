use std::sync::Arc;

use crate::tokens::token::Token;

use super::expressions::Expression;

#[derive(Debug)]
pub struct Variable {
    _name: Arc<Token>,
    _datatype: Option<DataType>,
    _value: Expression,
}

impl Variable {
    pub fn new(name: Arc<Token>, datatype: Option<DataType>, value: Expression) -> Self {
        Self {
            _name: name,
            _datatype: datatype,
            _value: value,
        }
    }
}

#[derive(Debug)]
pub enum DataType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bool,
    Void,
    Array(Box<DataType>),
    Pointer(Box<DataType>),
}
