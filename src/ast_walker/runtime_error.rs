#[derive(Debug)]
pub enum RuntimeError {
    NameRedefinition(String),
}
