use std::sync::Arc;

use crate::tokens::{source::SourceFile, token::Token};

pub struct ParserHead<'a>{
    pub curr: &'a mut Arc<Token>,
    pub prev: &'a mut Arc<Token>,
    pub source: &'a mut SourceFile
}

impl<'a> ParserHead<'a> {
    pub fn new(curr: &'a mut Arc<Token>, prev: &'a mut Arc<Token>, source: &'a mut SourceFile) -> Self {
        Self { curr, prev, source }
    }
}
