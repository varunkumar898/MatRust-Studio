use crate::ast::*;
use crate::token::{Token, TokenKind};
use matlab_diagnostics::{Diagnostic, Span};

/// Binding power of the colon operator (`a:step:b`). Colon binds below every
/// arithmetic/comparison operator, matching MATLAB's precedence table.
const COLON_BP: u8 = 30;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// True while an element is being parsed inside `[ ]` matrix brackets.
    /// MATLAB's concatenation rules apply there: a `+`/`-` separated from the
    /// previous element by whitespace but attached to its operand (e.g. the
    /// `-1` in `[0 -1]`) starts a NEW element instead of continuing as
    /// subtraction. Outside brackets (`y = 1 -2`) the sign stays binary.
    in_bracket_literal: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            in_bracket_literal: false,
        }
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

    /// `[X, Y] = expr`: parse the name list after an opening bracket. Returns
    /// the multi-assignment target when the bracket contained a comma- or
    /// space-separated identifier list, or None when this bracket is an
    /// ordinary matrix literal (caller rewinds and retries as an expression).
    fn try_parse_multi_assign_names(&mut self) -> Result<Option<AssignTarget>, Diagnostic> {
        let bracket_span = self.peek_span();
        self.advance(); // consume '['
        let mut names: Vec<(String, Span)> = Vec::new();
        loop {
            match self.peek().clone() {
                TokenKind::Ident(name) => {
                    self.advance();
                    names.push((name, self.tokens[self.pos - 1].span));
                }
                TokenKind::Comma | TokenKind::Newline => {
                    self.advance();
                }
                TokenKind::RBracket => {
                    self.advance();
                    if names.is_empty() {
                        return Ok(None); // `[] = ...` never valid; treat as literal
                    }
                    // Assignment requires '=' next; caller checks and rewinds.
                    return Ok(Some(AssignTarget::Multi(names, bracket_span)));
                }
                TokenKind::Number(_) | TokenKind::StringLit(_) => return Ok(None),
                TokenKind::Eof => {
                    return Err(Diagnostic::error(
                        "Unterminated multiple-assignment: expected ']'",
                        Some(self.peek_span()),
                    ))
                }
                _ => return Ok(None),
            }
        }
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

            // MATLAB bracket-concatenation rule: inside `[ ]`, a `+`/`-` with
            // a whitespace gap to its left and no gap to its right (e.g. the
            // `-1` in `[0 -1; 1 0]`) ends this element and starts the next
            // one with a unary sign. Parentheses and call arguments reset the
            // bracket mode, so `y = 1 -2` and `foo(1 -2)` stay subtraction.
            if self.in_bracket_literal
                && matches!(self.peek(), TokenKind::Plus | TokenKind::Minus)
                && self.pos > 0
            {
                let op_tok = &self.tokens[self.pos];
                let detached_left = self.tokens[self.pos - 1].span.end < op_tok.span.start;
                let attached_right = self
                    .tokens
                    .get(self.pos + 1)
                    .map(|next| next.span.start == op_tok.span.end)
                    .unwrap_or(false);
                if detached_left && attached_right {
                    break;
                }
            }

            // Infix operators
            let (op, l_bp, r_bp) = match self.peek() {
                TokenKind::Caret => (BinaryOp::Pow, 70, 71),
                TokenKind::DotCaret => (BinaryOp::DotPow, 70, 71),
                TokenKind::Star => (BinaryOp::Mul, 60, 61),
                TokenKind::Slash => (BinaryOp::Div, 60, 61),
                TokenKind::Backslash => (BinaryOp::LDiv, 60, 61),
                TokenKind::DotStar => (BinaryOp::DotMul, 60, 61),
                TokenKind::DotSlash => (BinaryOp::DotDiv, 60, 61),
                TokenKind::DotBackslash => (BinaryOp::DotLDiv, 60, 61),
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
            TokenKind::ImaginaryNumber(val) => Ok(Expr::Imaginary(val, tok.span)),
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
                    // Call arguments are NOT bracket-concatenation context:
                    // `foo(1 -2)` is subtraction in MATLAB.
                    let saved_mode = self.in_bracket_literal;
                    self.in_bracket_literal = false;
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        args.push(self.parse_expression(0)?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.in_bracket_literal = saved_mode;
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
                // Grouped parentheses are NOT bracket-concatenation context:
                // `(1 -2)` is subtraction in MATLAB.
                let saved_mode = self.in_bracket_literal;
                self.in_bracket_literal = false;
                let inner = self.parse_expression(0)?;
                self.in_bracket_literal = saved_mode;
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
    // ---------------------------------------------------- control flow
    fn parse_block(&mut self, stops: &[&str]) -> Result<Vec<Stmt>, Diagnostic> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &TokenKind::Eof {
                return Ok(stmts);
            }
            if let TokenKind::Ident(name) = self.peek().clone() {
                if stops.contains(&name.as_str()) {
                    return Ok(stmts);
                }
            }
            stmts.push(self.parse_statement()?);
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        self.skip_newlines();
        let start_span = self.peek_span();
        if let TokenKind::Ident(name) = self.peek().clone() {
            match name.as_str() {
                "if" => return self.parse_if(start_span),
                "while" => return self.parse_while(start_span),
                "for" => return self.parse_for(start_span),
                "switch" => return self.parse_switch(start_span),
                "function" => return self.parse_function(start_span),
                "break" => {
                    let tok = self.advance();
                    self.consume_stmt_terminator();
                    return Ok(Stmt::Break { span: tok.span });
                }
                "continue" => {
                    let tok = self.advance();
                    self.consume_stmt_terminator();
                    return Ok(Stmt::Continue { span: tok.span });
                }
                "return" => {
                    let tok = self.advance();
                    self.consume_stmt_terminator();
                    return Ok(Stmt::Return { span: tok.span });
                }
                "end" | "else" | "elseif" | "case" | "otherwise" => {
                    return Err(Diagnostic::error(
                        format!("Unexpected '{}' outside of a block.", name),
                        Some(start_span),
                    ));
                }
                _ => {}
            }
        }
        if self.peek() == &TokenKind::LBracket {
            let save = self.pos;
            if let Some(target) = self.try_parse_multi_assign_names()? {
                if !self.match_token(&TokenKind::Equal) {
                    self.pos = save;
                } else {
                    let val_expr = self.parse_expression(0)?;
                    let (suppress, end_span) = self.consume_stmt_terminator();
                    return Ok(Stmt::Assignment {
                        target,
                        value: val_expr,
                        suppress_output: suppress,
                        span: start_span.merge(end_span),
                    });
                }
            }
        }
        if let TokenKind::Ident(name) = self.peek().clone() {
            let next_pos = self.pos + 1;
            let next_kind = self.tokens.get(next_pos).map(|t| &t.kind);
            if next_kind == Some(&TokenKind::Equal) {
                let ident_tok = self.advance();
                self.advance();
                let val_expr = self.parse_expression(0)?;
                let (suppress, end_span) = self.consume_stmt_terminator();
                return Ok(Stmt::Assignment {
                    target: AssignTarget::Ident(name, ident_tok.span),
                    value: val_expr,
                    suppress_output: suppress,
                    span: start_span.merge(end_span),
                });
            } else if next_kind == Some(&TokenKind::LParen) {
                if self.find_assignment_equals_after_paren(next_pos).is_some() {
                    let ident_tok = self.advance();
                    self.advance();
                    let mut indices = Vec::new();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        indices.push(self.parse_expression(0)?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    if !self.match_token(&TokenKind::RParen) {
                        return Err(Diagnostic::error("Expected ')'", Some(self.peek_span())));
                    }
                    if !self.match_token(&TokenKind::Equal) {
                        return Err(Diagnostic::error("Expected '='", Some(self.peek_span())));
                    }
                    let val_expr = self.parse_expression(0)?;
                    let (suppress, end_span) = self.consume_stmt_terminator();
                    return Ok(Stmt::Assignment {
                        target: AssignTarget::Index {
                            name,
                            indices,
                            span: ident_tok.span,
                        },
                        value: val_expr,
                        suppress_output: suppress,
                        span: start_span.merge(end_span),
                    });
                }
            }
            if matches!(next_kind, Some(TokenKind::Ident(_))) {
                let call = self.parse_command_call(start_span)?;
                let (suppress, end_span) = self.consume_stmt_terminator();
                return Ok(Stmt::Expr {
                    expr: call,
                    suppress_output: suppress,
                    span: start_span.merge(end_span),
                });
            }
        }
        let expr = self.parse_expression(0)?;
        let (suppress, end_span) = self.consume_stmt_terminator();
        Ok(Stmt::Expr {
            expr,
            suppress_output: suppress,
            span: start_span.merge(end_span),
        })
    }

    fn parse_if(&mut self, start_span: Span) -> Result<Stmt, Diagnostic> {
        self.advance();
        let cond = self.parse_expression(0)?;
        self.consume_stmt_terminator();
        let mut branches: Vec<(Expr, Vec<Stmt>)> = Vec::new();
        let mut else_body: Option<Vec<Stmt>> = None;
        branches.push((cond, self.parse_block(&["end", "else", "elseif"])?));
        loop {
            if let TokenKind::Ident(n) = self.peek().clone() {
                if n == "elseif" {
                    self.advance();
                    let c = self.parse_expression(0)?;
                    self.consume_stmt_terminator();
                    branches.push((c, self.parse_block(&["end", "else", "elseif"])?));
                    continue;
                }
                if n == "else" {
                    self.advance();
                    self.consume_stmt_terminator();
                    else_body = Some(self.parse_block(&["end"])?);
                    break;
                }
            }
            break;
        }
        if let TokenKind::Ident(n) = self.peek().clone() {
            if n == "end" {
                self.advance();
            } else {
                return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
            }
        } else {
            return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
        }
        let (_, end_span) = self.consume_stmt_terminator();
        Ok(Stmt::If {
            branches,
            else_body,
            span: start_span.merge(end_span),
        })
    }

    fn parse_while(&mut self, start_span: Span) -> Result<Stmt, Diagnostic> {
        self.advance();
        let cond = self.parse_expression(0)?;
        self.consume_stmt_terminator();
        let body = self.parse_block(&["end"])?;
        if let TokenKind::Ident(n) = self.peek().clone() {
            if n == "end" {
                self.advance();
            } else {
                return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
            }
        } else {
            return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
        }
        let (_, end_span) = self.consume_stmt_terminator();
        Ok(Stmt::While {
            cond,
            body,
            span: start_span.merge(end_span),
        })
    }

    fn parse_for(&mut self, start_span: Span) -> Result<Stmt, Diagnostic> {
        self.advance();
        let paren_open = self.match_token(&TokenKind::LParen);
        if let TokenKind::Ident(name) = self.peek().clone() {
            let var_tok = self.advance();
            let var_span = var_tok.span;
            if self.peek() == &TokenKind::Equal {
                self.advance();
                let expr = self.parse_expression(0)?;
                if paren_open && !self.match_token(&TokenKind::RParen) {
                    return Err(Diagnostic::error("Expected ')'", Some(self.peek_span())));
                }
                self.consume_stmt_terminator();
                let body = self.parse_block(&["end"])?;
                if let TokenKind::Ident(n) = self.peek().clone() {
                    if n == "end" {
                        self.advance();
                    } else {
                        return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
                    }
                } else {
                    return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
                }
                let (_, end_span) = self.consume_stmt_terminator();
                return Ok(Stmt::For {
                    var: name,
                    var_span,
                    values: expr,
                    body,
                    span: start_span.merge(end_span),
                });
            }
        }
        Err(Diagnostic::error(
            "Expected 'for var = expr'",
            Some(start_span),
        ))
    }

    fn parse_switch(&mut self, start_span: Span) -> Result<Stmt, Diagnostic> {
        self.advance();
        let expr = self.parse_expression(0)?;
        self.consume_stmt_terminator();
        let mut cases: Vec<(Vec<Expr>, Vec<Stmt>)> = Vec::new();
        let mut otherwise: Option<Vec<Stmt>> = None;
        loop {
            self.skip_newlines();
            if let TokenKind::Ident(n) = self.peek().clone() {
                if n == "case" {
                    self.advance();
                    let mut labels = Vec::new();
                    labels.push(self.parse_expression(0)?);
                    // In MATLAB a comma after the case expression separates
                    // the label from the first body statement (`case 1, x=2`).
                    // It does not introduce another label.
                    if self.peek() == &TokenKind::Comma {
                        self.advance();
                    }
                    self.consume_stmt_terminator();
                    cases.push((labels, self.parse_block(&["case", "otherwise", "end"])?));
                    continue;
                }
                if n == "otherwise" {
                    self.advance();
                    self.consume_stmt_terminator();
                    otherwise = Some(self.parse_block(&["end"])?);
                    break;
                }
                if n == "end" {
                    self.advance();
                    let (_, end_span) = self.consume_stmt_terminator();
                    return Ok(Stmt::Switch {
                        expr,
                        cases,
                        otherwise,
                        span: start_span.merge(end_span),
                    });
                }
            }
            if self.peek() == &TokenKind::Eof {
                return Err(Diagnostic::error("Expected 'end'", Some(start_span)));
            }
            return Err(Diagnostic::error(
                "Unexpected statement inside 'switch'",
                Some(self.peek_span()),
            ));
        }
        if let TokenKind::Ident(n) = self.peek().clone() {
            if n == "end" {
                self.advance();
            } else {
                return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
            }
        } else {
            return Err(Diagnostic::error("Expected 'end'", Some(self.peek_span())));
        }
        let (_, end_span) = self.consume_stmt_terminator();
        Ok(Stmt::Switch {
            expr,
            cases,
            otherwise,
            span: start_span.merge(end_span),
        })
    }
    /// Parse `function [out1, out2] = name(in1, in2) ... end`.
    ///
    /// Both the outputless form (`function name(x)`) and the single-output
    /// form (`function y = name(x)`) are accepted. Function bodies may contain
    /// any block statement; nested `if`/`for`/`while` blocks consume their own
    /// `end`, so the body terminates at the first `end` belonging to the
    /// function itself.
    fn parse_function(&mut self, start_span: Span) -> Result<Stmt, Diagnostic> {
        self.advance(); // consume `function`
        self.skip_newlines();

        let mut outputs: Vec<(String, Span)> = Vec::new();
        let name: String;
        let name_span: Span;

        match self.peek().clone() {
            TokenKind::LBracket => {
                self.advance(); // consume '['
                loop {
                    self.skip_newlines();
                    match self.peek().clone() {
                        TokenKind::Ident(out_name) => {
                            let tok = self.advance();
                            outputs.push((out_name, tok.span));
                        }
                        TokenKind::Comma => {
                            self.advance();
                        }
                        TokenKind::RBracket => {
                            self.advance();
                            break;
                        }
                        _ => {
                            return Err(Diagnostic::error(
                                "Expected an output name or ']' in function declaration.",
                                Some(self.peek_span()),
                            ));
                        }
                    }
                }
                if !self.match_token(&TokenKind::Equal) {
                    return Err(Diagnostic::error(
                        "Expected '=' after function outputs.",
                        Some(self.peek_span()),
                    ));
                }
                self.skip_newlines();
                let name_tok = self.advance();
                match name_tok.kind {
                    TokenKind::Ident(n) => {
                        name = n;
                        name_span = name_tok.span;
                    }
                    _ => {
                        return Err(Diagnostic::error(
                            "Expected a function name after '='.",
                            Some(name_tok.span),
                        ));
                    }
                }
            }
            TokenKind::Ident(first) => {
                let first_tok = self.advance();
                if self.match_token(&TokenKind::Equal) {
                    // `function y = name(...)`
                    outputs.push((first, first_tok.span));
                    self.skip_newlines();
                    let name_tok = self.advance();
                    match name_tok.kind {
                        TokenKind::Ident(n) => {
                            name = n;
                            name_span = name_tok.span;
                        }
                        _ => {
                            return Err(Diagnostic::error(
                                "Expected a function name after '='.",
                                Some(name_tok.span),
                            ));
                        }
                    }
                } else {
                    // `function name(...)` (no declared outputs)
                    name = first;
                    name_span = first_tok.span;
                }
            }
            _ => {
                return Err(Diagnostic::error(
                    "Expected a function name or output list after 'function'.",
                    Some(self.peek_span()),
                ));
            }
        }

        if !self.match_token(&TokenKind::LParen) {
            return Err(Diagnostic::error(
                "Expected '(' after function name.",
                Some(self.peek_span()),
            ));
        }
        let mut params: Vec<(String, Span)> = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek().clone() {
                TokenKind::RParen => {
                    self.advance();
                    break;
                }
                TokenKind::Comma => {
                    self.advance();
                }
                TokenKind::Ident(param) => {
                    let tok = self.advance();
                    params.push((param, tok.span));
                }
                _ => {
                    return Err(Diagnostic::error(
                        "Expected a parameter name or ')' in function declaration.",
                        Some(self.peek_span()),
                    ));
                }
            }
        }

        self.consume_stmt_terminator();
        let body = self.parse_block(&["end"])?;
        if let TokenKind::Ident(end) = self.peek().clone() {
            if end != "end" {
                return Err(Diagnostic::error(
                    "Expected 'end' to close function definition.",
                    Some(self.peek_span()),
                ));
            }
            self.advance();
        } else {
            return Err(Diagnostic::error(
                "Expected 'end' to close function definition.",
                Some(self.peek_span()),
            ));
        }
        let (_, end_span) = self.consume_stmt_terminator();
        Ok(Stmt::FunctionDef {
            outputs,
            name,
            name_span,
            params,
            body,
            span: start_span.merge(end_span),
        })
    }

    fn parse_matrix_literal(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        // Enable MATLAB's whitespace-sensitive element separation only inside
        // the brackets themselves. Call arguments and grouped parentheses
        // inside the literal revert to ordinary expression parsing.
        let saved_mode = self.in_bracket_literal;
        self.in_bracket_literal = true;
        let result = self.parse_matrix_literal_body(start_span);
        self.in_bracket_literal = saved_mode;
        result
    }

    fn parse_matrix_literal_body(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
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

            // Parse one row element expression. Colon and comparisons bind
            // inside brackets, matching MATLAB (`[1:3]` is [1 2 3]).
            let element = self.parse_expression(COLON_BP)?;
            current_row.push(element);

            // Optional comma or continuation
            if self.peek() == &TokenKind::Comma {
                self.advance();
            }
        }
    }
}
