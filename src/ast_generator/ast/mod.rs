use self::function::Function;

pub mod function;
pub mod function_arg;

pub mod scopebound_statements;
pub mod expressions;

pub mod variables;

#[derive(Debug)]
pub enum Ast {
    Fn(Function),
}
