use crate::token::{Token, TokenKind};
use matlab_diagnostics::{Diagnostic, Span};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            cursor: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|&(_, c)| c)
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|&(_, c)| c)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, c)) = self.chars.get(self.cursor) {
            self.cursor += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn current_byte_pos(&self) -> usize {
        self.chars
            .get(self.cursor)
            .map(|&(pos, _)| pos)
            .unwrap_or(self.source.len())
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut tokens = Vec::new();

        while self.cursor < self.chars.len() {
            let start_byte = self.current_byte_pos();
            let start_line = self.line;
            let start_col = self.column;

            let c = self.advance().unwrap();

            // Skip spaces and tabs
            if c == ' ' || c == '\t' || c == '\r' {
                continue;
            }

            // Newline
            if c == '\n' {
                let span = Span::new(start_byte, start_byte + 1, start_line, start_col);
                tokens.push(Token::new(TokenKind::Newline, span));
                continue;
            }

            // Comment: % to end of line
            if c == '%' {
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // Delimiters
            let kind = match c {
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                ':' => TokenKind::Colon,
                '+' => TokenKind::Plus,
                '-' => TokenKind::Minus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '^' => TokenKind::Caret,
                '\'' => {
                    // MATLAB rule: an apostrophe starts a char literal when an operand
                    // can begin there; otherwise it is the (conjugate) transpose.
                    let operand_can_start = !matches!(
                        tokens.last().map(|t| &t.kind),
                        Some(TokenKind::Number(_))
                            | Some(TokenKind::StringLit(_))
                            | Some(TokenKind::Ident(_))
                            | Some(TokenKind::RParen)
                            | Some(TokenKind::RBracket)
                            | Some(TokenKind::Quote)
                            | Some(TokenKind::DotQuote)
                    );
                    if !operand_can_start {
                        TokenKind::Quote
                    } else {
                        let mut s = String::new();
                        let mut closed = false;
                        while let Some(ch) = self.advance() {
                            if ch == '\'' {
                                if self.peek() == Some('\'') {
                                    // '' escapes to a single quote inside a char literal
                                    s.push('\'');
                                    self.advance();
                                    continue;
                                }
                                closed = true;
                                break;
                            }
                            s.push(ch);
                        }
                        if !closed {
                            return Err(Diagnostic::error(
                                "Unterminated character literal (missing closing quote)",
                                Some(Span::new(
                                    start_byte,
                                    self.current_byte_pos(),
                                    start_line,
                                    start_col,
                                )),
                            ));
                        }
                        TokenKind::StringLit(s)
                    }
                }

                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    }
                }
                '~' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::NotEqual
                    } else {
                        return Err(Diagnostic::error(
                            "Unexpected character '~'. Did you mean '~='?",
                            Some(Span::new(start_byte, start_byte + 1, start_line, start_col)),
                        ));
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    }
                }
                '.' => {
                    match self.peek() {
                        Some('*') => {
                            self.advance();
                            TokenKind::DotStar
                        }
                        Some('/') => {
                            self.advance();
                            TokenKind::DotSlash
                        }
                        Some('^') => {
                            self.advance();
                            TokenKind::DotCaret
                        }
                        Some('\'') => {
                            self.advance();
                            TokenKind::DotQuote
                        }
                        Some(d) if d.is_ascii_digit() => {
                            // Floating point number starting with .
                            self.read_number(start_byte, start_line, start_col, true)?
                        }
                        _ => {
                            return Err(Diagnostic::error(
                                "Unexpected lonely '.' operator",
                                Some(Span::new(start_byte, start_byte + 1, start_line, start_col)),
                            ));
                        }
                    }
                }

                // Identifiers: letters or underscore
                ch if ch.is_ascii_alphabetic() || ch == '_' => {
                    let mut ident = String::new();
                    ident.push(ch);
                    while let Some(next_ch) = self.peek() {
                        if next_ch.is_ascii_alphanumeric() || next_ch == '_' {
                            ident.push(next_ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    TokenKind::Ident(ident)
                }

                // Numbers: digits
                ch if ch.is_ascii_digit() => {
                    self.read_number(start_byte, start_line, start_col, false)?
                }

                // Strings: "..." or '...'
                '"' => {
                    let mut s = String::new();
                    let mut closed = false;
                    while let Some(ch) = self.advance() {
                        if ch == '"' {
                            closed = true;
                            break;
                        }
                        s.push(ch);
                    }
                    if !closed {
                        return Err(Diagnostic::error(
                            "Unterminated string literal",
                            Some(Span::new(
                                start_byte,
                                self.current_byte_pos(),
                                start_line,
                                start_col,
                            )),
                        ));
                    }
                    TokenKind::StringLit(s)
                }

                other => {
                    return Err(Diagnostic::error(
                        format!("Unexpected character '{}'", other),
                        Some(Span::new(start_byte, start_byte + 1, start_line, start_col)),
                    ));
                }
            };

            let end_byte = self.current_byte_pos();
            let span = Span::new(start_byte, end_byte, start_line, start_col);
            tokens.push(Token::new(kind, span));
        }

        let eof_pos = self.source.len();
        tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(eof_pos, eof_pos, self.line, self.column),
        ));
        Ok(tokens)
    }

    fn read_number(
        &mut self,
        start_byte: usize,
        start_line: usize,
        start_col: usize,
        had_dot: bool,
    ) -> Result<TokenKind, Diagnostic> {
        let mut dot_seen = had_dot;
        let mut exp_seen = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !dot_seen && !exp_seen {
                // Peek ahead to ensure it's not followed by an operator like .*
                if let Some(next) = self.peek_next() {
                    if next == '*' || next == '/' || next == '^' || next == '\'' {
                        break;
                    }
                }
                dot_seen = true;
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !exp_seen {
                exp_seen = true;
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        let end_byte = self.current_byte_pos();
        let num_str = &self.source[start_byte..end_byte];
        match num_str.parse::<f64>() {
            Ok(v) => Ok(TokenKind::Number(v)),
            Err(e) => Err(Diagnostic::error(
                format!("Invalid numeric literal '{}': {}", num_str, e),
                Some(Span::new(start_byte, end_byte, start_line, start_col)),
            )),
        }
    }
}
