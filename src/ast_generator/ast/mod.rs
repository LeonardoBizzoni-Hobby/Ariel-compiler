use structs::Struct;

use self::{enums::Enum, function::Function};

pub mod enums;
pub mod function;
pub mod structs;

pub mod expressions;
pub mod scopebound_statements;

pub mod variables;
pub mod datatypes;

pub struct ASTs {
    pub fns: Vec<Function>,
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
}

impl ASTs {
    pub fn new() -> Self {
        Self {
            fns: vec![],
            enums: vec![],
            structs: vec![],
        }
    }

    #[inline]
    pub fn merge(&mut self, other: ASTs) {
        self.fns.extend(other.fns);
        self.enums.extend(other.enums);
        self.structs.extend(other.structs);
    }
}
