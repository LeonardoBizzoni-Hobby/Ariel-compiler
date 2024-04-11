use self::function::LbFunction;

pub mod function_field;
pub mod function;

#[derive(Debug)]
pub enum ScopeBoundStatement {}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Ast {
    Integer(i32),
    Fn(LbFunction),
}
