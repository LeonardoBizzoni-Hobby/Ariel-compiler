use crate::{
    ast_generator::ast::{
        function_arg::Argument, scopebound_statements::ScopeBoundStatement, variables::DataType,
    },
    tokens::token::Token,
};

use std::sync::Arc;

#[derive(Debug)]
pub struct Function {
    _is_main: bool,
    _name: Arc<Token>,
    args: Vec<Argument>,
    ret_type: Option<DataType>,
    body: Option<ScopeBoundStatement>,
}

impl Function {
    pub fn make_main(token: Arc<Token>) -> Self {
        Self {
            _is_main: true,
            _name: token,
            args: vec![],
            ret_type: None,
            body: None,
        }
    }

    pub fn make_func(token: Arc<Token>) -> Self {
        Self {
            _is_main: false,
            _name: token,
            args: vec![],
            ret_type: None,
            body: None,
        }
    }

    pub fn args(&mut self, args: Vec<Argument>) {
        self.args = args;
    }

    pub fn body(&mut self, body: Option<ScopeBoundStatement>) {
        self.body = body;
    }

    pub fn ret_type(&mut self, ret_type: DataType) {
        self.ret_type = Some(ret_type);
    }
}
