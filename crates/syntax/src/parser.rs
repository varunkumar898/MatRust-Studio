use crate::ast::*;
use crate::token::{Token, TokenKind};
use matlab_diagnostics::{Diagnostic, Span};

/// Binding power of the colon operator (`a:step:b`). Colon binds below every
/// arithmetic/comparison operator, matching MATLAB's precedence table.
const COLON_BP: u8 = 30;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or_default()
    }

    fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            let last_span = self.tokens.last().map(|t| t.span).unwrap_or_default();
            Token::new(TokenKind::Eof, last_span)
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.peek() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &TokenKind::Newline {
            self.advance();
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut statements = Vec::new();

        self.skip_newlines();
        while self.peek() != &TokenKind::Eof {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        self.skip_newlines();
        let start_span = self.peek_span();

        // Check for assignment: Ident = ... OR Ident(...) = ...
        if let TokenKind::Ident(name) = self.peek().clone() {
            // Lookahead
            let next_pos = self.pos + 1;
            let next_kind = self.tokens.get(next_pos).map(|t| &t.kind);

            if next_kind == Some(&TokenKind::Equal) {
                // Simple variable assignment: name = expr
                let ident_tok = self.advance();
                self.advance(); // consume '='
                let val_expr = self.parse_expression(0)?;
                let (suppress, end_span) = self.consume_stmt_terminator();
                let span = start_span.merge(end_span);
                return Ok(Stmt::Assignment {
                    target: AssignTarget::Ident(name, ident_tok.span),
                    value: val_expr,
                    suppress_output: suppress,
                    span,
                });
            } else if next_kind == Some(&TokenKind::LParen) {
                // Could be index assignment: A(1, 2) = ...
                // Check if closing paren is followed by '='
                if self.find_assignment_equals_after_paren(next_pos).is_some() {
                    let ident_tok = self.advance();
                    self.advance(); // consume '('
                    let mut indices = Vec::new();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        indices.push(self.parse_expression(0)?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    if !self.match_token(&TokenKind::RParen) {
                        return Err(Diagnostic::error(
                            "Expected ')' closing array index",
                            Some(self.peek_span()),
                        ));
                    }
                    if !self.match_token(&TokenKind::Equal) {
                        return Err(Diagnostic::error(
                            "Expected '=' in assignment",
                            Some(self.peek_span()),
                        ));
                    }
                    let val_expr = self.parse_expression(0)?;
                    let (suppress, end_span) = self.consume_stmt_terminator();
                    let span = start_span.merge(end_span);
                    return Ok(Stmt::Assignment {
                        target: AssignTarget::Index {
                            name,
                            indices,
                            span: ident_tok.span,
                        },
                        value: val_expr,
                        suppress_output: suppress,
                        span,
                    });
                }
            }

            // Command syntax: `Ident Ident ...` at statement start (e.g.
            // `grid on`, `close all`). Following identifiers are taken as
            // (string) arguments; the runtime resolves zero-argument bare
            // identifiers separately. Inside parentheses or after '=' this
            // rule does not apply, matching MATLAB's scoping of command
            // syntax to statement starts.
            if matches!(next_kind, Some(TokenKind::Ident(_))) {
                let call = self.parse_command_call(start_span)?;
                let (suppress, end_span) = self.consume_stmt_terminator();
                let span = start_span.merge(end_span);
                return Ok(Stmt::Expr {
                    expr: call,
                    suppress_output: suppress,
                    span,
                });
            }
        }

        // Otherwise, it is an expression statement
        let expr = self.parse_expression(0)?;
        let (suppress, end_span) = self.consume_stmt_terminator();
        let span = start_span.merge(end_span);
        Ok(Stmt::Expr {
            expr,
            suppress_output: suppress,
            span,
        })
    }

    /// Parse a command-syntax call: `Ident Ident...` at statement start.
    /// The first identifier is the callee; each subsequent identifier is
    /// passed as a string-literal argument (MATLAB passes command words as
    /// unevaluated text; identifiers/numbers cover `grid on`, `close all`,
    /// `format long`-style usage).
    fn parse_command_call(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let callee_tok = self.advance();
        let callee = match callee_tok.kind {
            TokenKind::Ident(name) => name,
            _ => {
                return Err(Diagnostic::error(
                    "Internal error: command call must start with an identifier.",
                    Some(callee_tok.span),
                ));
            }
        };
        let mut args = Vec::new();
        let mut end_span = callee_tok.span;
        loop {
            let (kind, span) = match self.tokens.get(self.pos) {
                Some(t) => (t.kind.clone(), t.span),
                None => break,
            };
            match kind {
                TokenKind::Ident(word) => {
                    self.advance();
                    end_span = span;
                    args.push(Expr::StringLit(word, span));
                }
                TokenKind::Number(v) => {
                    self.advance();
                    end_span = span;
                    args.push(Expr::StringLit(format!("{}", v), span));
                }
                _ => break,
            }
        }
        let span = start_span.merge(end_span);
        Ok(Expr::Call { callee, args, span })
    }

    fn find_assignment_equals_after_paren(&self, lparen_pos: usize) -> Option<usize> {
        let mut depth = 0;
        let mut i = lparen_pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        // Check if following token is '='
                        if self.tokens.get(i + 1).map(|t| &t.kind) == Some(&TokenKind::Equal) {
                            return Some(i + 1);
                        } else {
                            return None;
                        }
                    }
                }
                TokenKind::Semicolon | TokenKind::Newline | TokenKind::Eof => return None,
                _ => {}
            }
            i += 1;
        }
        None
    }

    fn consume_stmt_terminator(&mut self) -> (bool, Span) {
        let mut suppress = false;
        let mut last_span = self.peek_span();

        while match self.peek() {
            TokenKind::Semicolon => {
                suppress = true;
                last_span = self.peek_span();
                self.advance();
                true
            }
            TokenKind::Comma | TokenKind::Newline => {
                last_span = self.peek_span();
                self.advance();
                true
            }
            _ => false,
        } {}

        (suppress, last_span)
    }

    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Postfix operators: ', .'
            if self.peek() == &TokenKind::Quote {
                let op_tok = self.advance();
                let span = lhs.span().merge(op_tok.span);
                lhs = Expr::Transpose {
                    expr: Box::new(lhs),
                    conjugate: true,
                    span,
                };
                continue;
            } else if self.peek() == &TokenKind::DotQuote {
                let op_tok = self.advance();
                let span = lhs.span().merge(op_tok.span);
                lhs = Expr::Transpose {
                    expr: Box::new(lhs),
                    conjugate: false,
                    span,
                };
                continue;
            }

            // Colon operator: binds loosest (below comparisons), so it is only
            // consumed when the caller allows (min_bp <= COLON_BP). MATLAB
            // accepts exactly 2 or 3 colon operands: `a:b`, `a:step:b`.
            if self.peek() == &TokenKind::Colon {
                if min_bp > COLON_BP {
                    break;
                }
                if matches!(lhs, Expr::Range { .. }) {
                    return Err(Diagnostic::error(
                        "Too many ':' in a colon expression (max a:step:b).",
                        Some(self.peek_span()),
                    ));
                }
                self.advance(); // consume ':'
                let start_span = lhs.span();
                if self.peek() == &TokenKind::Colon {
                    // `a::b`— no step operand: MATLAB treats 1:0:3 as empty;
                    // a missing middle operand is a syntax error.
                    return Err(Diagnostic::error(
                        "Expected an expression between ':' operators.",
                        Some(self.peek_span()),
                    ));
                }
                // Middle operand: a plain expression above colon precedence.
                let middle = if matches!(self.peek(), TokenKind::Colon | TokenKind::Eof) {
                    return Err(Diagnostic::error(
                        "Missing expression for colon operand.",
                        Some(self.peek_span()),
                    ));
                } else {
                    self.parse_expression(COLON_BP + 1)?
                };
                if self.peek() == &TokenKind::Colon {
                    // `a:step:b`
                    self.advance();
                    let stop = self.parse_expression(COLON_BP + 1)?;
                    let span = start_span.merge(stop.span());
                    lhs = Expr::Range {
                        start: Box::new(lhs),
                        step: Some(Box::new(middle)),
                        stop: Box::new(stop),
                        span,
                    };
                } else {
                    // `a:b`
                    let span = start_span.merge(middle.span());
                    lhs = Expr::Range {
                        start: Box::new(lhs),
                        step: None,
                        stop: Box::new(middle),
                        span,
                    };
                    // Any further colon is too many (documented limitation:
                    // MATLAB's chained `a:b:c:d` forms without a step are
                    // rarely used; we reject them explicitly).
                    if self.peek() == &TokenKind::Colon {
                        return Err(Diagnostic::error(
                            "Too many ':' in a colon expression (max a:step:b).",
                            Some(self.peek_span()),
                        ));
                    }
                }
                continue;
            }

            // Infix operators
            let (op, l_bp, r_bp) = match self.peek() {
                TokenKind::Caret => (BinaryOp::Pow, 70, 71),
                TokenKind::DotCaret => (BinaryOp::DotPow, 70, 71),
                TokenKind::Star => (BinaryOp::Mul, 60, 61),
                TokenKind::Slash => (BinaryOp::Div, 60, 61),
                TokenKind::DotStar => (BinaryOp::DotMul, 60, 61),
                TokenKind::DotSlash => (BinaryOp::DotDiv, 60, 61),
                TokenKind::Plus => (BinaryOp::Add, 50, 51),
                TokenKind::Minus => (BinaryOp::Sub, 50, 51),
                TokenKind::EqualEqual => (BinaryOp::Eq, 40, 41),
                TokenKind::NotEqual => (BinaryOp::Ne, 40, 41),
                TokenKind::Less => (BinaryOp::Lt, 40, 41),
                TokenKind::LessEqual => (BinaryOp::Le, 40, 41),
                TokenKind::Greater => (BinaryOp::Gt, 40, 41),
                TokenKind::GreaterEqual => (BinaryOp::Ge, 40, 41),
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.advance(); // consume operator
            let rhs = self.parse_expression(r_bp)?;
            let span = lhs.span().merge(rhs.span());
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
                span,
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, Diagnostic> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Number(val) => Ok(Expr::Number(val, tok.span)),
            TokenKind::StringLit(s) => Ok(Expr::StringLit(s, tok.span)),
            TokenKind::Colon => Ok(Expr::ColonAll(tok.span)),
            TokenKind::Ident(name) => {
                // Reserved indexing keyword: `end` binds to the enclosing
                // indexed array's dimension (evaluated in interp; error when
                // used outside indexing).
                if name == "end" {
                    return Ok(Expr::End(tok.span));
                }
                // If followed by '(', function call / index expression
                if self.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        args.push(self.parse_expression(0)?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    let end_span = self.peek_span();
                    if !self.match_token(&TokenKind::RParen) {
                        return Err(Diagnostic::error(
                            "Expected ')' closing function argument list",
                            Some(end_span),
                        ));
                    }
                    let span = tok.span.merge(end_span);
                    Ok(Expr::Call {
                        callee: name,
                        args,
                        span,
                    })
                } else {
                    Ok(Expr::Variable(name, tok.span))
                }
            }
            TokenKind::Plus => {
                let expr = self.parse_prefix()?;
                let span = tok.span.merge(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Pos,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Minus => {
                let expr = self.parse_prefix()?;
                let span = tok.span.merge(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::LParen => {
                let inner = self.parse_expression(0)?;
                if !self.match_token(&TokenKind::RParen) {
                    return Err(Diagnostic::error(
                        "Expected ')' closing parenthesized expression",
                        Some(self.peek_span()),
                    ));
                }
                Ok(inner)
            }
            TokenKind::LBracket => self.parse_matrix_literal(tok.span),
            other => Err(Diagnostic::error(
                format!("Unexpected token '{:?}' in expression", other),
                Some(tok.span),
            )),
        }
    }

    /// Parse matrix literal: [1 2; 3 4] or [1, 2; 3, 4]
    fn parse_matrix_literal(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut current_row: Vec<Expr> = Vec::new();

        loop {
            // Skip any initial newlines inside brackets
            while self.peek() == &TokenKind::Newline {
                self.advance();
                if !current_row.is_empty() {
                    rows.push(std::mem::take(&mut current_row));
                }
            }

            if self.peek() == &TokenKind::RBracket {
                let close_tok = self.advance();
                if !current_row.is_empty() {
                    rows.push(current_row);
                }
                let span = start_span.merge(close_tok.span);
                return Ok(Expr::MatrixLit(rows, span));
            }

            if self.peek() == &TokenKind::Eof {
                return Err(Diagnostic::error(
                    "Unterminated matrix literal: expected ']'",
                    Some(self.peek_span()),
                ));
            }

            if self.match_token(&TokenKind::Semicolon) {
                rows.push(std::mem::take(&mut current_row));
                continue;
            }

            if self.match_token(&TokenKind::Comma) {
                // Comma explicitly separates elements in current row
                continue;
            }

            // Parse one row element expression
            let element = self.parse_expression(45)?;
            current_row.push(element);

            // Optional comma or continuation
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
    }
}
