use std::sync::Arc;

use crate::tokens::token::Token;

use super::{function_field::Argument, DataType, ScopeBoundStatement};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Function {
    is_main: bool,
    name: Arc<Token>,
    args: Vec<Argument>,
    ret_type: Option<DataType>,
    body: Vec<ScopeBoundStatement>,
}

#[allow(dead_code)]
impl Function {
    pub fn make_main(token: Arc<Token>) -> Self {
        Self {
            is_main: true,
            name: token,
            args: vec![],
            ret_type: None,
            body: vec![],
        }
    }

    pub fn make_func(token: Arc<Token>) -> Self {
        Self {
            is_main: false,
            name: token,
            args: vec![],
            ret_type: None,
            body: vec![],
        }
    }

    pub fn args(&mut self, args: Vec<Argument>) {
        self.args = args;
    }

    pub fn body(&mut self, body: Vec<ScopeBoundStatement>) {
        self.body = body;
    }

    pub fn ret_type(&mut self, ret_type: DataType) {
        self.ret_type = Some(ret_type);
    }
}
