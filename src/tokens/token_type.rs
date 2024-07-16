use std::fmt::Display;

use colored::Colorize;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TokenType {
    Eof,
    Unknown(char),

    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    LeftBrace,
    RightBrace,

    Dot,
    SequenceUpTo,
    SequenceUpToIncluding,
    Comma,
    Arrow,
    Colon,
    Semicolon,
    DynamicDefinition,
    StaticScopeGetter,

    Question,

    Mod,
    Not,
    NotEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    ShiftRight,
    ShiftRightEqual,
    Less,
    LessEqual,
    ShiftLeft,
    ShiftLeftEqual,
    And,
    BitAnd,
    Or,
    BitOr,
    Match,
    Minus,
    MinusEquals,
    Plus,
    PlusEquals,
    Slash,
    IntegerSlash,
    IntegerSlashEquals,
    SlashEquals,
    Star,
    StarEquals,
    Power,
    PowerEquals,

    Double,
    Integer,
    String,
    True,
    False,
    Identifier,
    Nil,

    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    StringType,
    Bool,
    Void,

    DontCare,
    Break,
    Continue,
    Else,
    Enum,
    Fn,
    For,
    If,
    Import,
    Let,
    Loop,
    Main,
    Return,
    Struct,
    While,
    InvalidByteSequenceToString,
    Defer,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Eof => write!(f, "{}", "EOF".red()),
            TokenType::Unknown(ch) => write!(f, "{} {ch}", "[Unknown Symbol]".red()),
            TokenType::InvalidByteSequenceToString => write!(f, "{}", "[InvalidByteSequenceToString]".red()),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftSquare => write!(f, "["),
            TokenType::RightSquare => write!(f, "]"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Dot => write!(f, "."),
            TokenType::SequenceUpTo => write!(f, ".."),
            TokenType::SequenceUpToIncluding => write!(f, "..="),
            TokenType::Comma => write!(f, ","),
            TokenType::Arrow => write!(f, "->"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::DynamicDefinition => write!(f, ":="),
            TokenType::StaticScopeGetter => write!(f, "::"),
            TokenType::Question => write!(f, "?"),
            TokenType::Mod => write!(f, "mod"),
            TokenType::Not => write!(f, "!"),
            TokenType::NotEqual => write!(f, "!="),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::ShiftRight => write!(f, ">>"),
            TokenType::ShiftRightEqual => write!(f, ">>="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::ShiftLeft => write!(f, "<<"),
            TokenType::ShiftLeftEqual => write!(f, "<<="),
            TokenType::And => write!(f, "&&"),
            TokenType::BitAnd => write!(f, "&"),
            TokenType::Or => write!(f, "||"),
            TokenType::BitOr => write!(f, "|"),
            TokenType::Match => write!(f, "match"),
            TokenType::Minus => write!(f, "-"),
            TokenType::MinusEquals => write!(f, "-="),
            TokenType::Plus => write!(f, "+"),
            TokenType::PlusEquals => write!(f, "+="),
            TokenType::Slash => write!(f, "/"),
            TokenType::IntegerSlash => write!(f, "//"),
            TokenType::IntegerSlashEquals => write!(f, "//="),
            TokenType::SlashEquals => write!(f, "/="),
            TokenType::Star => write!(f, "*"),
            TokenType::StarEquals => write!(f, "*="),
            TokenType::Power => write!(f, "**"),
            TokenType::PowerEquals => write!(f, "**="),
            TokenType::Double => write!(f, "double precision number"),
            TokenType::Integer => write!(f, "integer number"),
            TokenType::String => write!(f, "string"),
            TokenType::True => write!(f, "boolean true"),
            TokenType::False => write!(f, "boolean false"),
            TokenType::Identifier => write!(f, "identifier"),
            TokenType::Nil => write!(f, "nil"),
            TokenType::U8 => write!(f, "u8"),
            TokenType::U16 => write!(f, "u16"),
            TokenType::U32 => write!(f, "u32"),
            TokenType::U64 => write!(f, "u64"),
            TokenType::I8 => write!(f, "i8"),
            TokenType::I16 => write!(f, "i16"),
            TokenType::I32 => write!(f, "i32"),
            TokenType::I64 => write!(f, "i64"),
            TokenType::F32 => write!(f, "f32"),
            TokenType::F64 => write!(f, "f64"),
            TokenType::StringType => write!(f, "string type"),
            TokenType::Void => write!(f, "void type"),
            TokenType::DontCare => write!(f, "_"),
            TokenType::Break => write!(f, "break"),
            TokenType::Continue => write!(f, "continue"),
            TokenType::Else => write!(f, "else"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::Fn => write!(f, "fn"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::Import => write!(f, "import"),
            TokenType::Let => write!(f, "let"),
            TokenType::Loop => write!(f, "loop"),
            TokenType::Main => write!(f, "main"),
            TokenType::Return => write!(f, "return"),
            TokenType::Struct => write!(f, "struct"),
            TokenType::While => write!(f, "while"),
            TokenType::Bool => write!(f, "boolean"),
            TokenType::Defer => write!(f, "defer"),
        }
    }
}
