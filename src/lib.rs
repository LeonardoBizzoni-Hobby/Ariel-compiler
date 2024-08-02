use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ast_generator::ast::Ast;
use ast_walker::{global_env::Environment, value::Value};

use crate::ast_generator::parser;

mod ast_generator;
mod ast_walker;
mod test_util;
mod tokens;

pub fn compile(source: &str) {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    #[cfg(debug_assertions)]
    let start_timer = std::time::Instant::now();

    let ast_forest: Vec<Ast> = parser::parse(source, imported_files);

    #[cfg(debug_assertions)]
    {
        let elapsed = start_timer.elapsed();

        println!(
            "Parsing took: {}ns ({}ms)",
            elapsed.as_nanos(),
            elapsed.as_millis()
        );

        // println!("{:#?}", ast_forest);
    }

    let mut glob_env: Environment = HashMap::new();
    macro_rules! check_and_insert {
        ($ast2check:expr, $value2insert:expr) => {
            if glob_env.contains_key($ast2check.name.lexeme.as_str()) {
                eprintln!("`{}` is defined more then once.", $ast2check.name.lexeme);
                return;
            }

            glob_env.insert(&$ast2check.name.lexeme, $value2insert);
        };
    }

    for ast in ast_forest.iter() {
        match ast {
            Ast::Fn(func) => {
                check_and_insert!(
                    func,
                    Value::Function {
                        arity: func.args.len(),
                        closure_env: &glob_env as *const Environment,
                        ast: func,
                    }
                );
            }
            Ast::Enum(enumeration) => {
                check_and_insert!(enumeration, Value::Enum { ast: enumeration });
            }
            Ast::Struct(class) => {
                check_and_insert!(class, Value::Struct { ast: class });
            }
        }
    }

    println!("{glob_env:#?}");
}
