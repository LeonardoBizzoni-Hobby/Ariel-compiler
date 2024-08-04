use crate::{
    ast_generator::ast::{
        scopebound_statements::ScopeBoundStatement, datatypes::DataType,
    },
    tokens::token::Token,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Function {
    is_main: bool,
    pub name: Box<Token>,
    pub args: Vec<(Box<Token>, DataType)>,
    pub ret_type: Option<DataType>,
    pub body: Option<Vec<ScopeBoundStatement>>,
}

impl Function {
    pub fn make_main(token: Box<Token>) -> Self {
        Self {
            is_main: true,
            name: token,
            args: vec![],
            ret_type: None,
            body: None,
        }
    }

    pub fn make_func(token: Box<Token>) -> Self {
        Self {
            is_main: false,
            name: token,
            args: vec![],
            ret_type: None,
            body: None,
        }
    }
}
