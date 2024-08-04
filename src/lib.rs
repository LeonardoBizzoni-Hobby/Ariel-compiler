use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ast_generator::ast::{
    datatypes::DataType, enums::Enum, function::Function,
    scopebound_statements::ScopeBoundStatement, structs::Struct, ASTs,
};
use ast_walker::{env::Environment, value::Value};

use crate::ast_generator::parser;

mod ast_generator;
mod ast_walker;
mod test_util;
mod tokens;

#[macro_export]
macro_rules! measure {
    ($task:expr) => {{
        let start_timer = std::time::Instant::now();
        let res = $task;
        let elapsed = start_timer.elapsed();

        println!(
            "Task took: {}ns ({}ms)",
            elapsed.as_nanos(),
            elapsed.as_millis()
        );

        res
    }};
}

pub fn compile(source: &str) {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let ast: ASTs = measure!(parser::parse(source, imported_files));

    let mut global_env: Environment = HashMap::new();
    let env_ptr: *mut Environment = &mut global_env;

    macro_rules! check_and_insert {
        ($ast2check:expr, $value2insert:expr) => {
            unsafe {
                if (*env_ptr).contains_key($ast2check.name.lexeme.as_str()) {
                    eprintln!("`{}` is defined more then once.", $ast2check.name.lexeme);
                    return;
                }

                (*env_ptr).insert(&$ast2check.name.lexeme, $value2insert);
            }
        };
    }

    let _ = measure!({
        ast.fns.iter().for_each(|func| {
            check_and_insert!(
                func,
                Value::Function {
                    arity: func.args.len(),
                    returns: &func.ret_type,
                    closure_env: &global_env,
                    ast: func,
                }
            )
        });

        ast.enums.iter().for_each(|enumeration| {
            check_and_insert!(enumeration, Value::Enum { ast: enumeration })
        });

        ast.structs
            .iter()
            .for_each(|class| check_and_insert!(class, Value::Struct { ast: class }));
    });

    println!("{global_env:#?}");

    if !valid_structs(&global_env, &ast.structs)
        | !valid_enums(&global_env, &ast.enums)
        | !valid_fn(&global_env, &ast.fns)
    {
        return;
    }

    println!("All good!");
}

fn valid_field(env: &Environment, datatype: &DataType) -> bool {
    match datatype {
        DataType::Compound { name } => env.contains_key(name.lexeme.as_str()),
        DataType::Pointer(of) | DataType::Array(of) => valid_field(env, of),
        _ => true,
    }
}

fn valid_enums(env: &Environment, enums: &[Enum]) -> bool {
    let mut res = true;

    for myenum in enums.iter() {
        for (variant_name, datatype) in myenum.variants.iter() {
            if let Some(dt) = datatype {
                if !valid_field(env, dt) {
                    res = false;
                    eprintln!(
                        "[{} {}:{}] Enum `{}` defines field `{}` of type `{}` which doesn't exists.",
                        myenum.name.found_in, myenum.name.line, myenum.name.column, myenum.name.lexeme, variant_name.lexeme, dt
                    );
                }
            }
        }
    }

    res
}

fn valid_structs(env: &Environment, structs: &[Struct]) -> bool {
    let mut res = true;

    for mystruct in structs.iter() {
        for (fieldname, datatype) in mystruct.fields.iter() {
            if !valid_field(env, datatype) {
                res = false;
                eprintln!(
                    "[{} {}:{}] Struct `{}` defines field `{}` of type `{datatype}` which doesn't exists.",
                    mystruct.name.found_in, mystruct.name.line, mystruct.name.column, mystruct.name.lexeme, fieldname.lexeme
                );
            }
        }
    }

    res
}

fn valid_fn(env: &HashMap<&str, Value>, fns: &[Function]) -> bool {
    let mut res = true;

    for myfn in fns.iter() {
        for (arg, datatype) in myfn.args.iter() {
            if !valid_field(env, datatype) {
                res = false;
                eprintln!(
                    "[{} {}:{}] Function `{}` expects argument `{}` of type `{datatype}` which doesn't exists.",
                    myfn.name.found_in, myfn.name.line, myfn.name.column, myfn.name.lexeme, arg.lexeme
                );
            }
        }

        if let Some(datatype) = &myfn.ret_type {
            if !valid_field(env, datatype) {
                res = false;
                eprintln!(
                    "[{} {}:{}] Function `{}` returns `{datatype}` but this type isn't defined.",
                    myfn.name.found_in, myfn.name.line, myfn.name.column, myfn.name.lexeme
                );
            }
        }

        if let Some(body) = &myfn.body {
            for stmt in body.iter() {
                match *stmt {
                    ScopeBoundStatement::Scope(_) => todo!(),
                    ScopeBoundStatement::VariableDeclaration(_) => todo!(),
                    ScopeBoundStatement::Return(_) => todo!(),
                    ScopeBoundStatement::ImplicitReturn(_) => todo!(),
                    ScopeBoundStatement::Expression(_) => todo!(),
                    ScopeBoundStatement::Defer(_) => todo!(),
                    ScopeBoundStatement::Conditional { .. } => todo!(),
                    ScopeBoundStatement::Match { .. } => todo!(),
                    ScopeBoundStatement::Loop(_) => todo!(),
                    ScopeBoundStatement::While { .. } => todo!(),
                    ScopeBoundStatement::For { .. } => todo!(),
                    ScopeBoundStatement::Break => todo!(),
                    ScopeBoundStatement::Continue => todo!(),
                }
            }
        }
    }

    res
}
