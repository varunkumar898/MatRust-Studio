//! Token definitions and classifications for the MATLAB lexer.

use crate::span::Span;
use std::fmt;

/// Discriminant of a lexical token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    Imaginary(f64),
    CharVector(String),
    StringLiteral(String),

    // Identifier
    Ident(String),

    // Keywords
    If,
    Else,
    Elseif,
    End,
    For,
    While,
    Break,
    Continue,
    Return,
    Switch,
    Case,
    Otherwise,
    Try,
    Catch,
    Function,
    Global,
    Persistent,

    // Operators
    Plus,               // +
    Minus,              // -
    Star,               // *
    DotStar,            // .*
    Slash,              // /
    DotSlash,           // ./
    Backslash,          // \
    DotBackslash,       // .\
    Caret,              // ^
    DotCaret,           // .^
    Transpose,          // .'
    ConjugateTranspose, // '
    Equal,              // =
    EqEq,               // ==
    NotEq,              // ~=
    Lt,                 // <
    LtEq,               // <=
    Gt,                 // >
    GtEq,               // >=
    And,                // &
    Or,                 // |
    AndAnd,             // &&
    OrOr,               // ||
    Not,                // ~
    Colon,              // :

    // Delimiters & Punctuation
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .
    At,        // @
    Newline,   // \n

    // End of file
    Eof,
}

impl TokenKind {
    /// Returns true if this token can be followed by a transpose operator (`'` or `.'`).
    /// In MATLAB, `'` is a transpose operator only if preceded by an operand (variable, number,
    /// closing bracket/paren/brace, or another transpose). Otherwise, it begins a character literal.
    pub fn can_precede_transpose(&self) -> bool {
        matches!(
            self,
            TokenKind::Ident(_)
                | TokenKind::Number(_)
                | TokenKind::Imaginary(_)
                | TokenKind::CharVector(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::End
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Transpose
                | TokenKind::ConjugateTranspose
        )
    }

    /// Returns a human-friendly display name for diagnostics.
    pub fn display_name(&self) -> &'static str {
        match self {
            TokenKind::Number(_) => "number",
            TokenKind::Imaginary(_) => "imaginary number",
            TokenKind::CharVector(_) => "character vector",
            TokenKind::StringLiteral(_) => "string",
            TokenKind::Ident(_) => "identifier",
            TokenKind::If => "'if'",
            TokenKind::Else => "'else'",
            TokenKind::Elseif => "'elseif'",
            TokenKind::End => "'end'",
            TokenKind::For => "'for'",
            TokenKind::While => "'while'",
            TokenKind::Break => "'break'",
            TokenKind::Continue => "'continue'",
            TokenKind::Return => "'return'",
            TokenKind::Switch => "'switch'",
            TokenKind::Case => "'case'",
            TokenKind::Otherwise => "'otherwise'",
            TokenKind::Try => "'try'",
            TokenKind::Catch => "'catch'",
            TokenKind::Function => "'function'",
            TokenKind::Global => "'global'",
            TokenKind::Persistent => "'persistent'",
            TokenKind::Plus => "'+'",
            TokenKind::Minus => "'-'",
            TokenKind::Star => "'*'",
            TokenKind::DotStar => "'.*'",
            TokenKind::Slash => "'/'",
            TokenKind::DotSlash => "'./'",
            TokenKind::Backslash => "'\\'",
            TokenKind::DotBackslash => "'.\\'",
            TokenKind::Caret => "'^'",
            TokenKind::DotCaret => "'.^'",
            TokenKind::Transpose => "'.''",
            TokenKind::ConjugateTranspose => "'''",
            TokenKind::Equal => "'='",
            TokenKind::EqEq => "'=='",
            TokenKind::NotEq => "'~='",
            TokenKind::Lt => "'<'",
            TokenKind::LtEq => "'<='",
            TokenKind::Gt => "'>'",
            TokenKind::GtEq => "'>='",
            TokenKind::And => "'&'",
            TokenKind::Or => "'|'",
            TokenKind::AndAnd => "'&&'",
            TokenKind::OrOr => "'||'",
            TokenKind::Not => "'~'",
            TokenKind::Colon => "':'",
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::LBracket => "'['",
            TokenKind::RBracket => "']'",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::Semicolon => "';'",
            TokenKind::Comma => "','",
            TokenKind::Dot => "'.'",
            TokenKind::At => "'@'",
            TokenKind::Newline => "newline",
            TokenKind::Eof => "end of input",
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A token with its source location and whitespace context.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// True if whitespace separated this token from the preceding token.
    pub preceded_by_whitespace: bool,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, preceded_by_whitespace: bool) -> Self {
        Self {
            kind,
            span,
            preceded_by_whitespace,
        }
    }
}
