use std::fmt::Display;

use crate::tokens::token::Token;

#[derive(Debug, Eq, PartialEq, Clone)]
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
    Compound { name: Box<Token> },
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::U8 => write!(f, "u8"),
            DataType::U16 => write!(f, "u16"),
            DataType::U32 => write!(f, "u32"),
            DataType::U64 => write!(f, "u64"),
            DataType::Usize => write!(f, "usize"),
            DataType::I8 => write!(f, "i8"),
            DataType::I16 => write!(f, "i16"),
            DataType::I32 => write!(f, "i32"),
            DataType::I64 => write!(f, "i64"),
            DataType::Isize => write!(f, "isize"),
            DataType::F32 => write!(f, "f32"),
            DataType::F64 => write!(f, "f64"),
            DataType::String => write!(f, "string"),
            DataType::Bool => write!(f, "bool"),
            DataType::Void => write!(f, "void"),
            DataType::Array(of) => write!(f, "[{of}]"),
            DataType::Pointer(of) => write!(f, "{of}*"),
            DataType::Compound { name } => write!(f, "{name}"),
        }
    }
}
