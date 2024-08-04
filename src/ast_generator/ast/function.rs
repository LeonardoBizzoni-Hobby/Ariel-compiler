use crate::{
    ast_generator::ast::{
        function_arg::Argument, scopebound_statements::ScopeBoundStatement, datatypes::DataType,
    },
    tokens::token::Token,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Function {
    is_main: bool,
    pub name: Box<Token>,
    pub args: Vec<Argument>,
    pub ret_type: Option<DataType>,
    body: Option<Vec<ScopeBoundStatement>>,
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

    #[inline]
    pub fn args(&mut self, args: Vec<Argument>) {
        self.args = args;
    }

    #[inline]
    pub fn body(&mut self, body: Option<Vec<ScopeBoundStatement>>) {
        self.body = body;
    }

    #[inline]
    pub fn ret_type(&mut self, ret_type: DataType) {
        self.ret_type = Some(ret_type);
    }
}
