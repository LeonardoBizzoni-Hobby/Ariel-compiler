use super::ast::{expressions::Expression, scopebound_statements::ScopeBoundStatement};
use super::parser_head::ParserHead;
use super::statement_parser::parse_scopebound_statement;

use crate::{
    test_util::{create_test_file, delete_test_file},
    tokens::{
        error::ParseError, source::SourceFile, token::Token, token_type::TokenType, tokenizer,
    },
};

fn parse(file_name: &str, content: &str) -> Result<ScopeBoundStatement, ParseError> {
    create_test_file(file_name, content);
    let mut file = SourceFile::new(file_name).unwrap();

    let mut head = ParserHead::new(tokenizer::get_token(&mut file), Box::new(Token::new()), &mut file);

    delete_test_file(file_name);
    parse_scopebound_statement(&mut head)
}

mod conditional;
mod loops;
mod scope;
mod simple_stmt;
mod let_stmt;
