use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use crate::ast_generator::parser;

mod test_util;
mod ast_generator;
mod tokens;

pub fn compile(source: &str) {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    #[cfg(debug_assertions)]
    let start_timer = std::time::Instant::now();

    let _ast = parser::parse(source, Arc::clone(&imported_files));

    #[cfg(debug_assertions)]
    {
        let elapsed = start_timer.elapsed();

        println!(
            "Parsing took: {}ns ({}ms)",
            elapsed.as_nanos(),
            elapsed.as_millis()
        );

        // println!("{:#?}", ast);
    }
}
