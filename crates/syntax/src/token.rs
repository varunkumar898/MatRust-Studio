use matlab_diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    StringLit(String),
    Ident(String),

    // Arithmetic operators
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Caret,    // ^
    DotStar,  // .*
    DotSlash, // ./
    DotCaret, // .^
    DotQuote, // .'
    Quote,    // '

    // Assignment & Comparison
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // ~=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Delimiters & separators
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,
    Semicolon, // ;
    Colon,     // :
    Newline,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
