use std::fmt::Debug;

use crate::ast_generator::ast::{
    enums::Enum, function::Function, structs::Struct, datatypes::DataType,
};

use super::env::Environment;

#[allow(dead_code)]
pub enum Value<'a> {
    Function {
        arity: usize,
        returns: &'a Option<DataType>,
        closure_env: &'a Environment<'a>,
        ast: &'a Function,
    },
    Enum {
        ast: &'a Enum,
    },
    Struct {
        ast: &'a Struct,
    },
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function { arity, returns, .. } => f
                .debug_struct("Fn")
                .field("arity", arity)
                .field("returns", returns)
                .finish(),
            Self::Enum { .. } => f.debug_struct("Enum").finish(),
            Self::Struct { .. } => f.debug_struct("Struct").finish(),
        }
    }
}
