//! Hand-written MATLAB lexer with contextual quote and matrix whitespace handling.

use crate::error::ParseError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Lexer that transforms MATLAB source text into a stream of tokens.
pub struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    cursor: usize,
    prev_token_kind: Option<TokenKind>,
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer` from source text.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            cursor: 0,
            prev_token_kind: None,
        }
    }

    /// Tokenizes the entire source string into a vector of tokens ending with `Eof`.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// Fetches the next token from the input stream.
    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        let mut preceded_by_whitespace = false;

        // Loop to consume whitespace and comments
        loop {
            if self.is_eof() {
                let span = Span::point(self.cursor);
                return Ok(Token::new(TokenKind::Eof, span, preceded_by_whitespace));
            }

            let b = self.bytes[self.cursor];

            // Space or Tab or CR
            if b == b' ' || b == b'\t' || b == b'\r' {
                preceded_by_whitespace = true;
                self.cursor += 1;
                continue;
            }

            // Line continuation with three dots: '...'
            if b == b'.' && self.peek_bytes(3) == Some(b"...") {
                self.cursor += 3;
                preceded_by_whitespace = true;
                // Skip everything to the end of the line
                while !self.is_eof() && self.bytes[self.cursor] != b'\n' {
                    self.cursor += 1;
                }
                if !self.is_eof() && self.bytes[self.cursor] == b'\n' {
                    self.cursor += 1; // Consume newline without emitting it
                }
                continue;
            }

            // Comments
            if b == b'%' {
                // Check for block comment: '%{' on its own line
                if self.is_block_comment_start() {
                    self.skip_block_comment()?;
                    preceded_by_whitespace = true;
                    continue;
                }
                // Single line comment
                while !self.is_eof() && self.bytes[self.cursor] != b'\n' {
                    self.cursor += 1;
                }
                // Do not consume '\n' here; it will be lexed as a Newline token
                continue;
            }

            break;
        }

        let start = self.cursor;
        let b = self.bytes[self.cursor];

        // Newline
        if b == b'\n' {
            self.cursor += 1;
            let span = Span::new(start, self.cursor);
            let tok = Token::new(TokenKind::Newline, span, preceded_by_whitespace);
            self.prev_token_kind = Some(TokenKind::Newline);
            return Ok(tok);
        }

        // Contextual Transpose vs Character Vector
        // If '\'' occurs when preceding token can precede transpose AND there is no whitespace,
        // it's a transpose operator! (In MATLAB, whitespace before quote starts a char literal).
        if b == b'\'' {
            if !preceded_by_whitespace {
                if let Some(ref prev) = self.prev_token_kind {
                    if prev.can_precede_transpose() {
                        self.cursor += 1;
                        let span = Span::new(start, self.cursor);
                        let tok =
                            Token::new(TokenKind::ConjugateTranspose, span, preceded_by_whitespace);
                        self.prev_token_kind = Some(TokenKind::ConjugateTranspose);
                        return Ok(tok);
                    }
                }
            }
            // Otherwise, it is a character vector literal: 'hello'
            return self.lex_char_vector(start, preceded_by_whitespace);
        }

        // String literal: "hello"
        if b == b'"' {
            return self.lex_string_literal(start, preceded_by_whitespace);
        }

        // Numbers: digits or leading dot before digit (.5)
        if b.is_ascii_digit()
            || (b == b'.' && self.peek_byte(1).is_some_and(|next| next.is_ascii_digit()))
        {
            return self.lex_number(start, preceded_by_whitespace);
        }

        // Identifiers & Keywords
        if b.is_ascii_alphabetic() || b == b'_' {
            return self.lex_ident_or_keyword(start, preceded_by_whitespace);
        }

        // Multi-character & single-character operators
        let kind = match b {
            b'+' => {
                self.cursor += 1;
                TokenKind::Plus
            }
            b'-' => {
                self.cursor += 1;
                TokenKind::Minus
            }
            b'*' => {
                self.cursor += 1;
                TokenKind::Star
            }
            b'/' => {
                self.cursor += 1;
                TokenKind::Slash
            }
            b'\\' => {
                self.cursor += 1;
                TokenKind::Backslash
            }
            b'^' => {
                self.cursor += 1;
                TokenKind::Caret
            }
            b':' => {
                self.cursor += 1;
                TokenKind::Colon
            }
            b'=' => {
                self.cursor += 1;
                if !self.is_eof() && self.bytes[self.cursor] == b'=' {
                    self.cursor += 1;
                    TokenKind::EqEq
                } else {
                    TokenKind::Equal
                }
            }
            b'~' => {
                self.cursor += 1;
                if !self.is_eof() && self.bytes[self.cursor] == b'=' {
                    self.cursor += 1;
                    TokenKind::NotEq
                } else {
                    TokenKind::Not
                }
            }
            b'<' => {
                self.cursor += 1;
                if !self.is_eof() && self.bytes[self.cursor] == b'=' {
                    self.cursor += 1;
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            b'>' => {
                self.cursor += 1;
                if !self.is_eof() && self.bytes[self.cursor] == b'=' {
                    self.cursor += 1;
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            b'&' => {
                self.cursor += 1;
                if !self.is_eof() && self.bytes[self.cursor] == b'&' {
                    self.cursor += 1;
                    TokenKind::AndAnd
                } else {
                    TokenKind::And
                }
            }
            b'|' => {
                self.cursor += 1;
                if !self.is_eof() && self.bytes[self.cursor] == b'|' {
                    self.cursor += 1;
                    TokenKind::OrOr
                } else {
                    TokenKind::Or
                }
            }
            b'.' => {
                self.cursor += 1;
                if !self.is_eof() {
                    match self.bytes[self.cursor] {
                        b'*' => {
                            self.cursor += 1;
                            TokenKind::DotStar
                        }
                        b'/' => {
                            self.cursor += 1;
                            TokenKind::DotSlash
                        }
                        b'\\' => {
                            self.cursor += 1;
                            TokenKind::DotBackslash
                        }
                        b'^' => {
                            self.cursor += 1;
                            TokenKind::DotCaret
                        }
                        b'\'' => {
                            self.cursor += 1;
                            TokenKind::Transpose
                        }
                        _ => TokenKind::Dot,
                    }
                } else {
                    TokenKind::Dot
                }
            }
            b'(' => {
                self.cursor += 1;
                TokenKind::LParen
            }
            b')' => {
                self.cursor += 1;
                TokenKind::RParen
            }
            b'[' => {
                self.cursor += 1;
                TokenKind::LBracket
            }
            b']' => {
                self.cursor += 1;
                TokenKind::RBracket
            }
            b'{' => {
                self.cursor += 1;
                TokenKind::LBrace
            }
            b'}' => {
                self.cursor += 1;
                TokenKind::RBrace
            }
            b';' => {
                self.cursor += 1;
                TokenKind::Semicolon
            }
            b',' => {
                self.cursor += 1;
                TokenKind::Comma
            }
            b'@' => {
                self.cursor += 1;
                TokenKind::At
            }
            _ => {
                let ch = self.source[self.cursor..].chars().next().unwrap_or('\0');
                self.cursor += ch.len_utf8();
                return Err(ParseError::UnrecognizedCharacter {
                    ch,
                    span: Span::new(start, self.cursor),
                });
            }
        };

        let span = Span::new(start, self.cursor);
        let token = Token::new(kind.clone(), span, preceded_by_whitespace);
        self.prev_token_kind = Some(kind);
        Ok(token)
    }

    // --- Private Helper Lexers ---

    fn lex_ident_or_keyword(&mut self, start: usize, ws: bool) -> Result<Token, ParseError> {
        while !self.is_eof() {
            let b = self.bytes[self.cursor];
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.cursor += 1;
            } else {
                break;
            }
        }

        let text = &self.source[start..self.cursor];
        let kind = match text {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elseif" => TokenKind::Elseif,
            "end" => TokenKind::End,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "otherwise" => TokenKind::Otherwise,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "function" => TokenKind::Function,
            "global" => TokenKind::Global,
            "persistent" => TokenKind::Persistent,
            _ => TokenKind::Ident(text.to_string()),
        };

        let span = Span::new(start, self.cursor);
        let token = Token::new(kind.clone(), span, ws);
        self.prev_token_kind = Some(kind);
        Ok(token)
    }

    fn lex_number(&mut self, start: usize, ws: bool) -> Result<Token, ParseError> {
        // Hexadecimal 0x... or Binary 0b...
        if self.bytes[start] == b'0' && self.cursor + 1 < self.bytes.len() {
            let next_b = self.bytes[start + 1];
            if next_b == b'x' || next_b == b'X' {
                self.cursor += 2;
                while !self.is_eof()
                    && (self.bytes[self.cursor].is_ascii_hexdigit()
                        || self.bytes[self.cursor] == b'_')
                {
                    self.cursor += 1;
                }
                let raw = self.source[start + 2..self.cursor].replace('_', "");
                let val =
                    u64::from_str_radix(&raw, 16).map_err(|_| ParseError::InvalidNumberLiteral {
                        literal: self.source[start..self.cursor].to_string(),
                        span: Span::new(start, self.cursor),
                    })? as f64;
                let span = Span::new(start, self.cursor);
                let tok = Token::new(TokenKind::Number(val), span, ws);
                self.prev_token_kind = Some(TokenKind::Number(val));
                return Ok(tok);
            } else if next_b == b'b' || next_b == b'B' {
                self.cursor += 2;
                while !self.is_eof()
                    && (self.bytes[self.cursor] == b'0'
                        || self.bytes[self.cursor] == b'1'
                        || self.bytes[self.cursor] == b'_')
                {
                    self.cursor += 1;
                }
                let raw = self.source[start + 2..self.cursor].replace('_', "");
                let val =
                    u64::from_str_radix(&raw, 2).map_err(|_| ParseError::InvalidNumberLiteral {
                        literal: self.source[start..self.cursor].to_string(),
                        span: Span::new(start, self.cursor),
                    })? as f64;
                let span = Span::new(start, self.cursor);
                let tok = Token::new(TokenKind::Number(val), span, ws);
                self.prev_token_kind = Some(TokenKind::Number(val));
                return Ok(tok);
            }
        }

        // Standard decimal float
        while !self.is_eof()
            && (self.bytes[self.cursor].is_ascii_digit() || self.bytes[self.cursor] == b'_')
        {
            self.cursor += 1;
        }

        // Fractional part
        if !self.is_eof() && self.bytes[self.cursor] == b'.' {
            // Ensure next char is not an operator like '.*' or '.^' or './' or '.''
            let is_dot_op = self
                .peek_byte(1)
                .is_some_and(|b| b == b'*' || b == b'/' || b == b'\\' || b == b'^' || b == b'\'');
            if !is_dot_op {
                self.cursor += 1;
                while !self.is_eof()
                    && (self.bytes[self.cursor].is_ascii_digit() || self.bytes[self.cursor] == b'_')
                {
                    self.cursor += 1;
                }
            }
        }

        // Exponent part
        if !self.is_eof() && (self.bytes[self.cursor] == b'e' || self.bytes[self.cursor] == b'E') {
            let exp_start = self.cursor;
            self.cursor += 1;
            if !self.is_eof()
                && (self.bytes[self.cursor] == b'+' || self.bytes[self.cursor] == b'-')
            {
                self.cursor += 1;
            }
            let digit_start = self.cursor;
            while !self.is_eof()
                && (self.bytes[self.cursor].is_ascii_digit() || self.bytes[self.cursor] == b'_')
            {
                self.cursor += 1;
            }
            if digit_start == self.cursor {
                // No digits after exponent
                self.cursor = exp_start;
            }
        }

        // Check for imaginary suffix 'i' or 'j'
        let mut is_imaginary = false;
        if !self.is_eof() && (self.bytes[self.cursor] == b'i' || self.bytes[self.cursor] == b'j') {
            // Ensure it is not followed by an identifier character like 'item'
            let is_ident_continue = self
                .peek_byte(1)
                .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
            if !is_ident_continue {
                is_imaginary = true;
                self.cursor += 1;
            }
        }

        let num_str = if is_imaginary {
            &self.source[start..self.cursor - 1]
        } else {
            &self.source[start..self.cursor]
        };

        let sanitized = num_str.replace('_', "");
        let val: f64 = sanitized
            .parse()
            .map_err(|_| ParseError::InvalidNumberLiteral {
                literal: self.source[start..self.cursor].to_string(),
                span: Span::new(start, self.cursor),
            })?;

        let kind = if is_imaginary {
            TokenKind::Imaginary(val)
        } else {
            TokenKind::Number(val)
        };

        let span = Span::new(start, self.cursor);
        let token = Token::new(kind.clone(), span, ws);
        self.prev_token_kind = Some(kind);
        Ok(token)
    }

    fn lex_char_vector(&mut self, start: usize, ws: bool) -> Result<Token, ParseError> {
        self.cursor += 1; // Consume opening single quote '\''
        let mut content = String::new();

        loop {
            if self.is_eof() || self.bytes[self.cursor] == b'\n' {
                return Err(ParseError::UnclosedDelimiter {
                    delimiter: '\'',
                    span: Span::new(start, self.cursor),
                });
            }

            let b = self.bytes[self.cursor];
            if b == b'\'' {
                self.cursor += 1;
                // Escaped single quote: ''
                if !self.is_eof() && self.bytes[self.cursor] == b'\'' {
                    content.push('\'');
                    self.cursor += 1;
                } else {
                    // End of character vector
                    break;
                }
            } else {
                let ch = self.source[self.cursor..].chars().next().unwrap_or('\0');
                content.push(ch);
                self.cursor += ch.len_utf8();
            }
        }

        let span = Span::new(start, self.cursor);
        let tok = Token::new(TokenKind::CharVector(content), span, ws);
        self.prev_token_kind = Some(TokenKind::CharVector(String::new()));
        Ok(tok)
    }

    fn lex_string_literal(&mut self, start: usize, ws: bool) -> Result<Token, ParseError> {
        self.cursor += 1; // Consume opening double quote '"'
        let mut content = String::new();

        loop {
            if self.is_eof() || self.bytes[self.cursor] == b'\n' {
                return Err(ParseError::UnclosedDelimiter {
                    delimiter: '"',
                    span: Span::new(start, self.cursor),
                });
            }

            let b = self.bytes[self.cursor];
            if b == b'"' {
                self.cursor += 1;
                // Escaped double quote: ""
                if !self.is_eof() && self.bytes[self.cursor] == b'"' {
                    content.push('"');
                    self.cursor += 1;
                } else {
                    // End of string literal
                    break;
                }
            } else {
                let ch = self.source[self.cursor..].chars().next().unwrap_or('\0');
                content.push(ch);
                self.cursor += ch.len_utf8();
            }
        }

        let span = Span::new(start, self.cursor);
        let tok = Token::new(TokenKind::StringLiteral(content), span, ws);
        self.prev_token_kind = Some(TokenKind::StringLiteral(String::new()));
        Ok(tok)
    }

    fn is_block_comment_start(&self) -> bool {
        // Starts with %{ followed by optional whitespace and newline
        if self.cursor + 1 < self.bytes.len() && self.bytes[self.cursor + 1] == b'{' {
            let mut i = self.cursor + 2;
            while i < self.bytes.len()
                && (self.bytes[i] == b' ' || self.bytes[i] == b'\t' || self.bytes[i] == b'\r')
            {
                i += 1;
            }
            return i == self.bytes.len() || self.bytes[i] == b'\n';
        }
        false
    }

    fn skip_block_comment(&mut self) -> Result<(), ParseError> {
        let start = self.cursor;
        self.cursor += 2; // skip '%{'
        while !self.is_eof() {
            // Check for '%}'
            if self.bytes[self.cursor] == b'%'
                && self.cursor + 1 < self.bytes.len()
                && self.bytes[self.cursor + 1] == b'}'
            {
                self.cursor += 2;
                return Ok(());
            }
            self.cursor += 1;
        }
        Err(ParseError::UnclosedDelimiter {
            delimiter: '%',
            span: Span::new(start, self.cursor),
        })
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.bytes.len()
    }

    fn peek_byte(&self, offset: usize) -> Option<u8> {
        let idx = self.cursor + offset;
        if idx < self.bytes.len() {
            Some(self.bytes[idx])
        } else {
            None
        }
    }

    fn peek_bytes(&self, length: usize) -> Option<&'a [u8]> {
        let end = self.cursor + length;
        if end <= self.bytes.len() {
            Some(&self.bytes[self.cursor..end])
        } else {
            None
        }
    }
}
