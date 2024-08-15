use std::fmt::Display;

use crate::tokens::token::Token;

#[derive(Debug, Eq, Clone)]
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

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataType::U8, DataType::U8)
            | (DataType::U16, DataType::U16)
            | (DataType::U32, DataType::U32)
            | (DataType::U64, DataType::U64 | DataType::Usize)
            | (DataType::Usize, DataType::Usize)
            | (DataType::I8, DataType::I8)
            | (DataType::I16, DataType::I16)
            | (DataType::I32, DataType::I32)
            | (DataType::I64, DataType::I64 | DataType::Isize)
            | (DataType::Isize, DataType::Isize)
            | (DataType::F32, DataType::F32)
            | (DataType::F64, DataType::F64)
            | (DataType::String, DataType::String)
            | (DataType::Bool, DataType::Bool)
            | (DataType::Void, DataType::Void) => true,

            (DataType::Array(of), DataType::Array(other_of))
            | (DataType::Pointer(of), DataType::Pointer(other_of)) => of.eq(other_of),

            (DataType::Compound { name }, DataType::Compound { name: other_name }) => {
                name.lexeme == other_name.lexeme
            }

            _ => false,
        }
    }
}

impl PartialOrd for DataType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (DataType::U8, DataType::U8)
            | (DataType::U16, DataType::U16)
            | (DataType::U32, DataType::U32)
            | (DataType::U64, DataType::U64 | DataType::Usize)
            | (DataType::Usize, DataType::Usize)
            | (DataType::I8, DataType::I8)
            | (DataType::I16, DataType::I16)
            | (DataType::I32, DataType::I32)
            | (DataType::I64, DataType::I64 | DataType::Isize)
            | (DataType::Isize, DataType::Isize)
            | (DataType::F32, DataType::F32)
            | (DataType::F64, DataType::F64)
            | (DataType::String, DataType::String)
            | (DataType::Bool, DataType::Bool)
            | (DataType::Void, DataType::Void) => Some(std::cmp::Ordering::Equal),

            (DataType::Array(of), DataType::Array(other_of))
            | (DataType::Pointer(of), DataType::Pointer(other_of)) => of.partial_cmp(other_of),

            (DataType::Compound { name }, DataType::Compound { name: other_name }) => {
                if name.lexeme == other_name.lexeme {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    None
                }
            }

            (DataType::U8, DataType::I8)
            | (DataType::U16, DataType::U8 | DataType::I16)
            | (DataType::U32, DataType::U8 | DataType::U16 | DataType::I32)
            | (DataType::U64, DataType::U8 | DataType::U16 | DataType::U32 | DataType::I64)
            | (DataType::F64, DataType::F32) => Some(std::cmp::Ordering::Greater),

            _ => None,
        }
    }
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
