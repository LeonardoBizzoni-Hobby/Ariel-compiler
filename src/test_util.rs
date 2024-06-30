use std::{fs::File, io::Write};

pub fn create_test_file(name: &str, content: &str) {
    let mut file = File::create(name).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

pub fn delete_test_file(path: &str) {
    std::fs::remove_file(path).unwrap();
}
