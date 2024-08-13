use crate::ast_generator::ast::datatypes::DataType;

pub enum TypeError {
    InvalidReturnValue {
        line: usize,
        column: usize,
        expected: DataType,
        got: DataType,
    },
    UnexpectedType {
        line: usize,
        column: usize,
        expected: DataType,
        got: DataType,
    },
    InvalidStmt {
        line: usize,
        column: usize,
    },
    InvalidTypeConversion {
        line: usize,
        column: usize,
        from: DataType,
        to: DataType,
    },
}
