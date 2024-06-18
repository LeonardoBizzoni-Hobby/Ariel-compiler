use std::sync::Arc;

use crate::tokens::token::Token;

use super::variables::DataType;

#[derive(Debug)]
pub struct Argument(pub Arc<Token>, pub DataType);
