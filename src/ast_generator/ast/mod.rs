use structs::Struct;

use self::{enums::Enum, function::Function};

pub mod structs;
pub mod enums;
pub mod function;

pub mod function_arg;

pub mod expressions;
pub mod scopebound_statements;

pub mod variables;

#[derive(Debug)]
pub enum Ast {
    Fn(Function),
    Enum(Enum),
    Struct(Struct),
}
