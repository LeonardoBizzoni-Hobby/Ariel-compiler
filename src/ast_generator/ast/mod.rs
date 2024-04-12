use self::function::Function;

pub mod function_field;
pub mod function;

#[derive(Debug)]
pub enum ScopeBoundStatement {}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Ast {
    Integer(i32),
    Fn(Function),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DataType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bool,
    Void,
    Array(Box<DataType>),
    Pointer(Box<DataType>),
}
