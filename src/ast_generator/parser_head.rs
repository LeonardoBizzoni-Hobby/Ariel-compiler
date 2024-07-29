use std::mem;

use crate::tokens::{
    error::ParseError, source::SourceFile, token::Token, token_type::TokenType, tokenizer,
};

use super::ast::{function_arg::Argument, variables::DataType};

pub struct ParserHead<'a> {
    pub curr: Box<Token>,
    pub prev: Box<Token>,
    pub source: &'a mut SourceFile,
}

impl<'a> ParserHead<'a> {
    pub fn new(curr: Box<Token>, prev: Box<Token>, source: &'a mut SourceFile) -> Self {
        Self { curr, prev, source }
    }

    #[inline]
    pub fn advance(&mut self) -> &Token {
        self.prev = mem::replace(&mut self.curr, tokenizer::get_token(&mut self.source));
        &self.curr
    }

    pub fn require_current_is(&mut self, expected: TokenType) -> Result<(), ParseError> {
        if (*self.curr).ttype == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                token: std::mem::take(&mut self.curr),
                expected,
                msg: None,
            })
        }
    }

    pub fn parse_datatype(&mut self) -> Result<DataType, ParseError> {
        match self.curr.ttype {
            TokenType::U8 => Ok(self.handle_pointer_datatype(DataType::U8)),
            TokenType::U16 => Ok(self.handle_pointer_datatype(DataType::U16)),
            TokenType::U32 => Ok(self.handle_pointer_datatype(DataType::U32)),
            TokenType::U64 => Ok(self.handle_pointer_datatype(DataType::U64)),
            TokenType::Usize => Ok(self.handle_pointer_datatype(DataType::Usize)),
            TokenType::I8 => Ok(self.handle_pointer_datatype(DataType::I8)),
            TokenType::I16 => Ok(self.handle_pointer_datatype(DataType::I16)),
            TokenType::I32 => Ok(self.handle_pointer_datatype(DataType::I32)),
            TokenType::I64 => Ok(self.handle_pointer_datatype(DataType::I64)),
            TokenType::Isize => Ok(self.handle_pointer_datatype(DataType::Isize)),
            TokenType::F32 => Ok(self.handle_pointer_datatype(DataType::F32)),
            TokenType::F64 => Ok(self.handle_pointer_datatype(DataType::F64)),
            TokenType::Bool => Ok(self.handle_pointer_datatype(DataType::Bool)),
            TokenType::StringType => Ok(self.handle_pointer_datatype(DataType::String)),
            TokenType::Void => {
                let datatype = self.handle_pointer_datatype(DataType::Void);
                if matches!(datatype, DataType::Pointer(_)) {
                    Ok(datatype)
                } else {
                    Err(ParseError::InvalidDataType {
                        token: std::mem::take(&mut self.curr),
                        msg: Some(
                            String::from("`void` by itself isn't a valid datatype, it should have been a void pointer `void*`."),
                        ),
                    })
                }
            }
            TokenType::LeftSquare => {
                self.advance();
                let array_of: DataType = self.parse_datatype()?;

                self.require_current_is(TokenType::RightSquare)?;
                Ok(self.handle_pointer_datatype(DataType::Array(Box::new(array_of))))
            }
            TokenType::Identifier => {
                let datatype = DataType::Compound {
                    name: std::mem::take(&mut self.curr),
                };
                Ok(self.handle_pointer_datatype(datatype))
            }
            _ => Err(ParseError::InvalidDataType {
                token: std::mem::take(&mut self.curr),
                msg: None,
            }),
        }
    }

    #[inline]
    fn handle_pointer_datatype(&mut self, mut datatype: DataType) -> DataType {
        match self.advance().ttype {
            TokenType::Star => {
                while matches!(self.curr.ttype, TokenType::Star) {
                    self.advance();
                    datatype = DataType::Pointer(Box::new(datatype));
                }

                datatype
            }
            _ => datatype,
        }
    }

    pub fn parse_argument(&mut self) -> Result<Argument, ParseError> {
        self.require_current_is(TokenType::Identifier)?;
        let field_name = mem::take(&mut self.curr);

        // arg_name -> :
        self.advance();
        self.require_current_is(TokenType::Colon)?;

        // : -> datatype
        self.advance();

        Ok(Argument(field_name, self.parse_datatype()?))
    }

    pub fn synchronize(&mut self) {
        loop {
            self.advance();

            match self.curr.ttype {
                TokenType::Import | TokenType::Struct | TokenType::Fn | TokenType::Eof => break,
                _ => {}
            }
        }
    }
}
