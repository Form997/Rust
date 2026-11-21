use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Zero-based character offset in the source.
    pub start: usize,
    /// Exclusive zero-based character offset in the source.
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Program,
    Version,
    Import,
    Data,
    Rule,
    Function,
    Task,
    Output,
    End,

    Identifier(String),
    StringLiteral(String),
    Number(String),

    Colon,
    Equal,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: Span { start, end },
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program => write!(f, "PROGRAM"),
            Self::Version => write!(f, "VERSION"),
            Self::Import => write!(f, "IMPORT"),
            Self::Data => write!(f, "DATA"),
            Self::Rule => write!(f, "RULE"),
            Self::Function => write!(f, "FUNCTION"),
            Self::Task => write!(f, "TASK"),
            Self::Output => write!(f, "OUTPUT"),
            Self::End => write!(f, "END"),
            Self::Identifier(value) => write!(f, "IDENTIFIER({value})"),
            Self::StringLiteral(value) => write!(f, "STRING({value:?})"),
            Self::Number(value) => write!(f, "NUMBER({value})"),
            Self::Colon => write!(f, ":"),
            Self::Equal => write!(f, "="),
            Self::Newline => write!(f, "NEWLINE"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}
