use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Argument(pub Box<Token>, pub DataType);
