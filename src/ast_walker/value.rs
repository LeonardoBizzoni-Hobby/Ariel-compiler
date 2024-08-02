use crate::ast_generator::ast::{enums::Enum, function::Function, structs::Struct};

use super::global_env::Environment;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Value<'a> {
    Function {
        arity: usize,
        closure_env: *const Environment<'a>,
        ast: &'a Function,
    },
    Enum {
        ast: &'a Enum,
    },
    Struct {
        ast: &'a Struct,
    },
}
