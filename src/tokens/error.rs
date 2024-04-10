use std::any::Any;

#[derive(Debug)]
pub enum Error {
    FileNotFound(String, String),
    MemoryMapFiled(String, String),

    Parser(ParseError),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidImport,
    ParseThreadJoin(Box<dyn Any + Send>),
}
