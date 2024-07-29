use std::{collections::HashSet, sync::{Arc, Mutex}};

use ast_generator::ast::Ast;

use crate::ast_generator::parser;

mod ast_generator;
mod ast_walker;
mod test_util;
mod tokens;

pub fn compile(source: &str) {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    #[cfg(debug_assertions)]
    let start_timer = std::time::Instant::now();

    let _ast_forest: Vec<Ast> = parser::parse(source, imported_files);

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

    // let glob_env: HashMap<Arc<Token>, Ast> = HashMap::new();
    // for ast in ast_forest.iter() {
    //     match ast {
    //         Ast::Fn(func) => {
    //             if glob_env.contains_key(&func.name) {
    //                 eprintln!("`{}` is defined more then once.", func.name.lexeme);
    //                 return;
    //             }

    //             for arg in func.args.iter() {
    //                 match arg.1 {
    //                     DataType::Array(..) => todo!(),
    //                     DataType::Pointer(..) => todo!(),
    //                     DataType::Compound { .. } => {}
    //                     _ => {}
    //                 }
    //             }

    //             // glob_env.insert(Arc::clone(&func.name), *ast);
    //         }
    //         Ast::Enum(..) => todo!(),
    //         Ast::Struct(..) => todo!(),
    //     }
    // }
}
