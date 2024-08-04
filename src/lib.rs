use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ast_generator::ast::{datatypes::DataType, enums::Enum, structs::Struct, ASTs};
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

    if !valid_structs(&global_env, &ast.structs) || !valid_enums(&global_env, &ast.enums) {
        return;
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
                        "Enum `{}` defines field `{}` of type `{}` which doesn't exists.",
                        myenum.name.lexeme, variant_name.lexeme, dt
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
                    "Struct `{}` defines field `{}` of type `{datatype}` which doesn't exists.",
                    mystruct.name.lexeme, fieldname.lexeme
                );
            }
        }
    }

    res
}

fn valid_field(env: &Environment, datatype: &DataType) -> bool {
    match datatype {
        DataType::Compound { name } => env.contains_key(name.lexeme.as_str()),
        DataType::Pointer(of) | DataType::Array(of) => valid_field(env, of),
        _ => true,
    }
}
