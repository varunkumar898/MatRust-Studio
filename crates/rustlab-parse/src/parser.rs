//! Hand-written recursive descent and Pratt parser for MATLAB.

use crate::ast::*;
use crate::error::ParseError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Parser that converts a stream of `Token`s into a MATLAB AST.
pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    /// Counter tracking nesting depth inside matrix or cell brackets `[...]` or `{...}`.
    matrix_depth: usize,
    /// Recursion depth for expression parsing to prevent stack overflow.
    recursion_depth: usize,
    eof_token: Token,
}

const MAX_RECURSION_DEPTH: usize = 128;

impl Parser {
    /// Creates a new `Parser` from a slice or vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        let eof_span = Span::point(tokens.last().map_or(0, |t| t.span.end));
        let eof_token = Token::new(TokenKind::Eof, eof_span, false);
        Self {
            tokens,
            cursor: 0,
            matrix_depth: 0,
            recursion_depth: 0,
            eof_token,
        }
    }

    /// Parses the entire token stream into a sequence of top-level `Stmt`s.
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !self.is_eof() {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    // --- Statement Parsing ---

    /// Parses a single statement.
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.skip_newlines();
        let tok = self.peek();

        match tok.kind {
            TokenKind::If => self.parse_if(),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Switch => self.parse_switch(),
            TokenKind::Try => self.parse_try_catch(),
            TokenKind::Break => {
                let span = self.advance().span;
                self.consume_statement_terminator();
                Ok(Stmt::Break(span))
            }
            TokenKind::Continue => {
                let span = self.advance().span;
                self.consume_statement_terminator();
                Ok(Stmt::Continue(span))
            }
            TokenKind::Return => {
                let span = self.advance().span;
                self.consume_statement_terminator();
                Ok(Stmt::Return(span))
            }
            TokenKind::Global => self.parse_global(),
            TokenKind::Persistent => self.parse_persistent(),
            TokenKind::Function => self.parse_function_def(),
            _ => self.parse_expr_or_assignment(),
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::If)?.span;
        let cond = self.parse_expr()?;
        self.consume_optional_separator();

        let mut branches = Vec::new();
        let mut body = Vec::new();

        while !self.is_eof() {
            let k = &self.peek().kind;
            if matches!(k, TokenKind::Elseif | TokenKind::Else | TokenKind::End) {
                break;
            }
            body.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        branches.push((cond, body));

        while self.match_token(TokenKind::Elseif) {
            let else_cond = self.parse_expr()?;
            self.consume_optional_separator();
            let mut else_body = Vec::new();
            while !self.is_eof() {
                let k = &self.peek().kind;
                if matches!(k, TokenKind::Elseif | TokenKind::Else | TokenKind::End) {
                    break;
                }
                else_body.push(self.parse_stmt()?);
                self.skip_newlines();
            }
            branches.push((else_cond, else_body));
        }

        let mut else_branch = None;
        if self.match_token(TokenKind::Else) {
            self.consume_optional_separator();
            let mut else_body = Vec::new();
            while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
                else_body.push(self.parse_stmt()?);
                self.skip_newlines();
            }
            else_branch = Some(else_body);
        }

        let end_span = self.expect(TokenKind::End)?.span;
        self.consume_statement_terminator();

        Ok(Stmt::If {
            branches,
            else_branch,
            span: start_span.merge(end_span),
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::For)?.span;
        let var_tok = self.advance();
        let var = match var_tok.kind {
            TokenKind::Ident(name) => name,
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: var_tok.kind.display_name().to_string(),
                    span: var_tok.span,
                })
            }
        };

        self.expect(TokenKind::Equal)?;
        let expr = self.parse_expr()?;
        self.consume_optional_separator();

        let mut body = Vec::new();
        while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
            body.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let end_span = self.expect(TokenKind::End)?.span;
        self.consume_statement_terminator();

        Ok(Stmt::For {
            var,
            expr,
            body,
            span: start_span.merge(end_span),
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::While)?.span;
        let cond = self.parse_expr()?;
        self.consume_optional_separator();

        let mut body = Vec::new();
        while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
            body.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let end_span = self.expect(TokenKind::End)?.span;
        self.consume_statement_terminator();

        Ok(Stmt::While {
            cond,
            body,
            span: start_span.merge(end_span),
        })
    }

    fn parse_switch(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::Switch)?.span;
        let switch_expr = self.parse_expr()?;
        self.consume_optional_separator();

        let mut cases = Vec::new();
        let mut otherwise = None;

        while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
            self.skip_newlines();
            if self.match_token(TokenKind::Case) {
                let case_val = self.parse_expr()?;
                self.consume_optional_separator();
                let mut case_body = Vec::new();
                while !self.is_eof() {
                    let k = &self.peek().kind;
                    if matches!(k, TokenKind::Case | TokenKind::Otherwise | TokenKind::End) {
                        break;
                    }
                    case_body.push(self.parse_stmt()?);
                    self.skip_newlines();
                }
                cases.push((vec![case_val], case_body));
            } else if self.match_token(TokenKind::Otherwise) {
                self.consume_optional_separator();
                let mut otherwise_body = Vec::new();
                while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
                    otherwise_body.push(self.parse_stmt()?);
                    self.skip_newlines();
                }
                otherwise = Some(otherwise_body);
                break;
            } else {
                break;
            }
        }

        let end_span = self.expect(TokenKind::End)?.span;
        self.consume_statement_terminator();

        Ok(Stmt::Switch {
            expr: switch_expr,
            cases,
            otherwise,
            span: start_span.merge(end_span),
        })
    }

    fn parse_try_catch(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::Try)?.span;
        self.consume_optional_separator();

        let mut try_body = Vec::new();
        while !self.is_eof() {
            let k = &self.peek().kind;
            if matches!(k, TokenKind::Catch | TokenKind::End) {
                break;
            }
            try_body.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let mut catch_var = None;
        let mut catch_body = Vec::new();

        if self.match_token(TokenKind::Catch) {
            if let TokenKind::Ident(name) = &self.peek().kind {
                catch_var = Some(name.clone());
                self.advance();
            }
            self.consume_optional_separator();

            while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
                catch_body.push(self.parse_stmt()?);
                self.skip_newlines();
            }
        }

        let end_span = self.expect(TokenKind::End)?.span;
        self.consume_statement_terminator();

        Ok(Stmt::TryCatch {
            try_body,
            catch_var,
            catch_body,
            span: start_span.merge(end_span),
        })
    }

    fn parse_global(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::Global)?.span;
        let mut vars = Vec::new();

        while let TokenKind::Ident(name) = &self.peek().kind {
            vars.push(name.clone());
            self.advance();
        }

        let end_span = self.consume_statement_terminator();
        Ok(Stmt::Global(vars, start_span.merge(end_span)))
    }

    fn parse_persistent(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::Persistent)?.span;
        let mut vars = Vec::new();

        while let TokenKind::Ident(name) = &self.peek().kind {
            vars.push(name.clone());
            self.advance();
        }

        let end_span = self.consume_statement_terminator();
        Ok(Stmt::Persistent(vars, start_span.merge(end_span)))
    }

    fn parse_function_def(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.expect(TokenKind::Function)?.span;

        // Form 1: function [out1, out2] = name(in1, in2)
        // Form 2: function out = name(in1, in2)
        // Form 3: function name(in1, in2)
        let mut outputs = Vec::new();
        let name: String;

        if self.peek().kind == TokenKind::LBracket {
            // [out1, out2] = name(...)
            self.advance();
            while !self.is_eof() && self.peek().kind != TokenKind::RBracket {
                if let TokenKind::Ident(out_name) = &self.peek().kind {
                    outputs.push(out_name.clone());
                    self.advance();
                    if self.peek().kind == TokenKind::Comma {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RBracket)?;
            self.expect(TokenKind::Equal)?;
            let name_tok = self.advance();
            name = match name_tok.kind {
                TokenKind::Ident(n) => n,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "function name".to_string(),
                        found: name_tok.kind.display_name().to_string(),
                        span: name_tok.span,
                    })
                }
            };
        } else {
            let first_id = match self.advance().kind {
                TokenKind::Ident(id) => id,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "identifier".to_string(),
                        found: other.display_name().to_string(),
                        span: self.peek().span,
                    })
                }
            };

            if self.peek().kind == TokenKind::Equal {
                self.advance();
                outputs.push(first_id);
                let name_tok = self.advance();
                name = match name_tok.kind {
                    TokenKind::Ident(n) => n,
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "function name".to_string(),
                            found: name_tok.kind.display_name().to_string(),
                            span: name_tok.span,
                        })
                    }
                };
            } else {
                name = first_id;
            }
        }

        // Inputs: (in1, in2, ...)
        let mut inputs = Vec::new();
        if self.match_token(TokenKind::LParen) {
            while !self.is_eof() && self.peek().kind != TokenKind::RParen {
                if let TokenKind::Ident(in_name) = &self.peek().kind {
                    inputs.push(in_name.clone());
                    self.advance();
                    if self.peek().kind == TokenKind::Comma {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
        }

        self.consume_optional_separator();

        // Body
        let mut body = Vec::new();
        while !self.is_eof() && !matches!(self.peek().kind, TokenKind::End) {
            body.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let end_span = if self.match_token(TokenKind::End) {
            self.prev_span()
        } else {
            self.peek().span
        };
        self.consume_statement_terminator();

        Ok(Stmt::Function(FunctionDef {
            name,
            inputs,
            outputs,
            body,
            span: start_span.merge(end_span),
        }))
    }

    fn parse_expr_or_assignment(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek().span;

        // Check for multiple assignment target: [a, b, c] = ...
        if self.peek().kind == TokenKind::LBracket && self.is_multi_assign_ahead() {
            let targets = self.parse_multi_assign_targets()?;
            self.expect(TokenKind::Equal)?;
            let value = self.parse_expr()?;
            let (print_result, end_span) = self.consume_print_flag();
            return Ok(Stmt::Assignment {
                target: AssignTarget::MultiAssign(targets, start_span.merge(end_span)),
                value,
                print_result,
                span: start_span.merge(end_span),
            });
        }

        let expr = self.parse_expr()?;

        // If followed by '=', this is an assignment
        if self.match_token(TokenKind::Equal) {
            let target = self.expr_to_assign_target(expr)?;
            let value = self.parse_expr()?;
            let (print_result, end_span) = self.consume_print_flag();
            return Ok(Stmt::Assignment {
                target,
                value,
                print_result,
                span: start_span.merge(end_span),
            });
        }

        let (print_result, end_span) = self.consume_print_flag();
        Ok(Stmt::Expr {
            span: start_span.merge(end_span),
            expr,
            print_result,
        })
    }

    fn is_multi_assign_ahead(&self) -> bool {
        // Look ahead inside [ ... ] to see if followed by '='
        let mut idx = self.cursor + 1;
        let mut depth = 1;
        while idx < self.tokens.len() {
            match self.tokens[idx].kind {
                TokenKind::LBracket => depth += 1,
                TokenKind::RBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return idx + 1 < self.tokens.len()
                            && self.tokens[idx + 1].kind == TokenKind::Equal;
                    }
                }
                TokenKind::Eof | TokenKind::Newline | TokenKind::Semicolon => return false,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    fn parse_multi_assign_targets(&mut self) -> Result<Vec<AssignTarget>, ParseError> {
        self.expect(TokenKind::LBracket)?;
        let mut targets = Vec::new();

        while !self.is_eof() && self.peek().kind != TokenKind::RBracket {
            let expr = self.parse_postfix()?;
            targets.push(self.expr_to_assign_target(expr)?);
            if self.peek().kind == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBracket)?;
        Ok(targets)
    }

    fn expr_to_assign_target(&self, expr: Expr) -> Result<AssignTarget, ParseError> {
        match expr {
            Expr::Ident(name, span) => Ok(AssignTarget::Ident(name, span)),
            Expr::Index { callee, args, span } => Ok(AssignTarget::Index { callee, args, span }),
            Expr::CellIndex { callee, args, span } => {
                Ok(AssignTarget::CellIndex { callee, args, span })
            }
            Expr::DotAccess { expr, field, span } => Ok(AssignTarget::Dot { expr, field, span }),
            other => Err(ParseError::InvalidAssignmentTarget { span: other.span() }),
        }
    }

    // --- Expression Pratt Parser ---

    /// Parses an expression with minimum binding power 0.
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            return Err(ParseError::RecursionLimitExceeded {
                span: self.peek().span,
            });
        }
        self.recursion_depth += 1;
        let res = self.parse_expr_bp_inner(min_bp);
        self.recursion_depth -= 1;
        res
    }

    fn parse_expr_bp_inner(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;
        let mut op_count: usize = 0;

        loop {
            // Check for range colon operator: start:end or start:step:end
            // In MATLAB, the colon operator has lower precedence than additive + and - operators!
            if self.peek().kind == TokenKind::Colon && min_bp <= 11 {
                let _colon_span = self.advance().span;
                let next_expr = self.parse_expr_bp(12)?;
                if self.peek().kind == TokenKind::Colon {
                    self.advance();
                    let end_expr = self.parse_expr_bp(12)?;
                    let span = left.span().merge(end_expr.span());
                    left = Expr::Range {
                        start: Box::new(left),
                        step: Some(Box::new(next_expr)),
                        end: Box::new(end_expr),
                        span,
                    };
                } else {
                    let span = left.span().merge(next_expr.span());
                    left = Expr::Range {
                        start: Box::new(left),
                        step: None,
                        end: Box::new(next_expr),
                        span,
                    };
                }
                continue;
            }

            // Binary operators
            let (op, (l_bp, r_bp)) = match self.peek().kind {
                TokenKind::OrOr => (BinaryOp::OrOr, (1, 2)),
                TokenKind::AndAnd => (BinaryOp::AndAnd, (3, 4)),
                TokenKind::Or => (BinaryOp::Or, (5, 6)),
                TokenKind::And => (BinaryOp::And, (7, 8)),
                TokenKind::EqEq => (BinaryOp::Eq, (9, 10)),
                TokenKind::NotEq => (BinaryOp::NotEq, (9, 10)),
                TokenKind::Lt => (BinaryOp::Lt, (9, 10)),
                TokenKind::LtEq => (BinaryOp::LtEq, (9, 10)),
                TokenKind::Gt => (BinaryOp::Gt, (9, 10)),
                TokenKind::GtEq => (BinaryOp::GtEq, (9, 10)),
                TokenKind::Plus => (BinaryOp::Add, (13, 14)),
                TokenKind::Minus => (BinaryOp::Sub, (13, 14)),
                TokenKind::Star => (BinaryOp::Mul, (15, 16)),
                TokenKind::DotStar => (BinaryOp::DotMul, (15, 16)),
                TokenKind::Slash => (BinaryOp::Div, (15, 16)),
                TokenKind::DotSlash => (BinaryOp::DotDiv, (15, 16)),
                TokenKind::Backslash => (BinaryOp::LeftDiv, (15, 16)),
                TokenKind::DotBackslash => (BinaryOp::DotLeftDiv, (15, 16)),
                TokenKind::Caret => (BinaryOp::Pow, (18, 17)), // Right-associative
                TokenKind::DotCaret => (BinaryOp::DotPow, (18, 17)), // Right-associative
                _ => break,
            };

            // In matrix context, if an operator like '+' is preceded by whitespace and the next token is not,
            // it starts a new element in the matrix row!
            if self.matrix_depth > 0
                && self.peek().preceded_by_whitespace
                && (op == BinaryOp::Add || op == BinaryOp::Sub)
            {
                if let Some(next_tok) = self.tokens.get(self.cursor + 1) {
                    if !next_tok.preceded_by_whitespace {
                        break;
                    }
                }
            }

            if l_bp < min_bp {
                break;
            }

            op_count += 1;
            if op_count > MAX_RECURSION_DEPTH {
                return Err(ParseError::RecursionLimitExceeded {
                    span: self.peek().span,
                });
            }

            self.advance();
            let right = self.parse_expr_bp(r_bp)?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek();
        match tok.kind {
            TokenKind::Plus => {
                let op_span = self.advance().span;
                let expr = self.parse_expr_bp(19)?;
                let span = op_span.merge(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Minus => {
                let op_span = self.advance().span;
                let expr = self.parse_expr_bp(19)?;
                let span = op_span.merge(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Not => {
                let op_span = self.advance().span;
                let expr = self.parse_expr_bp(19)?;
                let span = op_span.merge(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        let mut chain_depth: usize = 0;

        loop {
            let tok = self.peek();
            if matches!(
                tok.kind,
                TokenKind::Transpose
                    | TokenKind::ConjugateTranspose
                    | TokenKind::LParen
                    | TokenKind::LBrace
                    | TokenKind::Dot
            ) {
                chain_depth += 1;
                if chain_depth > MAX_RECURSION_DEPTH {
                    return Err(ParseError::RecursionLimitExceeded { span: tok.span });
                }
            }

            match tok.kind {
                TokenKind::Transpose => {
                    let end_span = self.advance().span;
                    let span = expr.span().merge(end_span);
                    expr = Expr::Unary {
                        op: UnaryOp::Transpose,
                        expr: Box::new(expr),
                        span,
                    };
                }
                TokenKind::ConjugateTranspose => {
                    let end_span = self.advance().span;
                    let span = expr.span().merge(end_span);
                    expr = Expr::Unary {
                        op: UnaryOp::ConjugateTranspose,
                        expr: Box::new(expr),
                        span,
                    };
                }
                TokenKind::LParen => {
                    // Function call or indexing: A(x, y)
                    let start_span = self.advance().span;
                    let mut args = Vec::new();
                    while !self.is_eof() && self.peek().kind != TokenKind::RParen {
                        if self.peek().kind == TokenKind::Colon {
                            args.push(Expr::Colon(self.advance().span));
                        } else {
                            args.push(self.parse_expr()?);
                        }
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    let end_span = self.expect(TokenKind::RParen)?.span;
                    expr = Expr::Index {
                        callee: Box::new(expr),
                        args,
                        span: start_span.merge(end_span),
                    };
                }
                TokenKind::LBrace => {
                    // Cell array indexing: C{x, y}
                    let start_span = self.advance().span;
                    let mut args = Vec::new();
                    while !self.is_eof() && self.peek().kind != TokenKind::RBrace {
                        if self.peek().kind == TokenKind::Colon {
                            args.push(Expr::Colon(self.advance().span));
                        } else {
                            args.push(self.parse_expr()?);
                        }
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    let end_span = self.expect(TokenKind::RBrace)?.span;
                    expr = Expr::CellIndex {
                        callee: Box::new(expr),
                        args,
                        span: start_span.merge(end_span),
                    };
                }
                TokenKind::Dot => {
                    // Struct field: s.field or s.(expr)
                    self.advance();
                    if self.peek().kind == TokenKind::LParen {
                        self.advance();
                        let field_expr = self.parse_expr()?;
                        let end_span = self.expect(TokenKind::RParen)?.span;
                        let span = expr.span().merge(end_span);
                        expr = Expr::DynamicDotAccess {
                            expr: Box::new(expr),
                            field_expr: Box::new(field_expr),
                            span,
                        };
                    } else if let TokenKind::Ident(field) = self.advance().kind {
                        let span = expr.span().merge(self.prev_span());
                        expr = Expr::DotAccess {
                            expr: Box::new(expr),
                            field,
                            span,
                        };
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            expected: "field name or '('".to_string(),
                            found: self.peek().kind.display_name().to_string(),
                            span: self.peek().span,
                        });
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Number(n) => Ok(Expr::Number(n, tok.span)),
            TokenKind::Imaginary(n) => Ok(Expr::Imaginary(n, tok.span)),
            TokenKind::CharVector(s) => Ok(Expr::CharVector(s, tok.span)),
            TokenKind::StringLiteral(s) => Ok(Expr::StringLiteral(s, tok.span)),
            TokenKind::Ident(name) => Ok(Expr::Ident(name, tok.span)),
            TokenKind::End => Ok(Expr::End(tok.span)),
            TokenKind::Colon => Ok(Expr::Colon(tok.span)),
            TokenKind::LParen => {
                let inner = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(inner)
            }
            TokenKind::LBracket => self.parse_matrix_literal(tok.span),
            TokenKind::LBrace => self.parse_cell_literal(tok.span),
            TokenKind::At => self.parse_function_handle(tok.span),
            TokenKind::Eof => Err(ParseError::UnexpectedEof {
                expected: "expression".to_string(),
                span: tok.span,
            }),
            other => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: other.display_name().to_string(),
                span: tok.span,
            }),
        }
    }

    fn parse_matrix_literal(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        self.matrix_depth += 1;
        let mut rows = Vec::new();
        let mut current_row = Vec::new();

        self.skip_newlines_in_matrix();

        while !self.is_eof() && self.peek().kind != TokenKind::RBracket {
            let elem = self.parse_expr()?;
            current_row.push(elem);

            // Delimiters between elements on the same row or row separators
            let k = &self.peek().kind;
            if *k == TokenKind::Comma {
                self.advance();
                self.skip_newlines_in_matrix();
            } else if *k == TokenKind::Semicolon || *k == TokenKind::Newline {
                self.advance();
                rows.push(std::mem::take(&mut current_row));
                self.skip_newlines_in_matrix();
            } else if *k == TokenKind::RBracket {
                break;
            }
        }

        if !current_row.is_empty() {
            rows.push(current_row);
        }

        let end_span = self.expect(TokenKind::RBracket)?.span;
        self.matrix_depth -= 1;

        Ok(Expr::Matrix {
            rows,
            span: start_span.merge(end_span),
        })
    }

    fn parse_cell_literal(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        self.matrix_depth += 1;
        let mut rows = Vec::new();
        let mut current_row = Vec::new();

        self.skip_newlines_in_matrix();

        while !self.is_eof() && self.peek().kind != TokenKind::RBrace {
            let elem = self.parse_expr()?;
            current_row.push(elem);

            let k = &self.peek().kind;
            if *k == TokenKind::Comma {
                self.advance();
                self.skip_newlines_in_matrix();
            } else if *k == TokenKind::Semicolon || *k == TokenKind::Newline {
                self.advance();
                rows.push(std::mem::take(&mut current_row));
                self.skip_newlines_in_matrix();
            } else if *k == TokenKind::RBrace {
                break;
            }
        }

        if !current_row.is_empty() {
            rows.push(current_row);
        }

        let end_span = self.expect(TokenKind::RBrace)?.span;
        self.matrix_depth -= 1;

        Ok(Expr::CellArray {
            rows,
            span: start_span.merge(end_span),
        })
    }

    fn parse_function_handle(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        // Form 1: @sin (named handle)
        // Form 2: @(x, y) x + y (anonymous function)
        if self.match_token(TokenKind::LParen) {
            let mut params = Vec::new();
            while !self.is_eof() && self.peek().kind != TokenKind::RParen {
                if let TokenKind::Ident(param) = &self.peek().kind {
                    params.push(param.clone());
                    self.advance();
                    if self.peek().kind == TokenKind::Comma {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            let body = self.parse_expr()?;
            let span = start_span.merge(body.span());
            Ok(Expr::AnonymousFunction {
                params,
                body: Box::new(body),
                span,
            })
        } else if let TokenKind::Ident(name) = self.advance().kind {
            let span = start_span.merge(self.prev_span());
            Ok(Expr::FunctionHandle(name, span))
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "identifier or '('".to_string(),
                found: self.peek().kind.display_name().to_string(),
                span: self.peek().span,
            })
        }
    }

    // --- Helper Utilities ---

    fn peek(&self) -> &Token {
        self.tokens.get(self.cursor).unwrap_or(&self.eof_token)
    }

    fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            self.peek().clone()
        }
    }

    fn prev_span(&self) -> Span {
        if self.cursor > 0 && self.cursor <= self.tokens.len() {
            self.tokens[self.cursor - 1].span
        } else {
            Span::default()
        }
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        let tok = self.advance();
        if tok.kind == expected {
            Ok(tok)
        } else if tok.kind == TokenKind::Eof {
            Err(ParseError::UnexpectedEof {
                expected: expected.display_name().to_string(),
                span: tok.span,
            })
        } else {
            Err(ParseError::UnexpectedToken {
                expected: expected.display_name().to_string(),
                found: tok.kind.display_name().to_string(),
                span: tok.span,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek().kind == TokenKind::Newline {
            self.advance();
        }
    }

    fn skip_newlines_in_matrix(&mut self) {
        while self.peek().kind == TokenKind::Newline {
            self.advance();
        }
    }

    fn consume_optional_separator(&mut self) {
        while matches!(
            self.peek().kind,
            TokenKind::Comma | TokenKind::Semicolon | TokenKind::Newline
        ) {
            self.advance();
        }
    }

    fn consume_statement_terminator(&mut self) -> Span {
        let mut span = self.peek().span;
        while matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::Comma | TokenKind::Newline
        ) {
            span = self.advance().span;
        }
        span
    }

    fn consume_print_flag(&mut self) -> (bool, Span) {
        let mut print_result = true;
        let mut last_span = self.prev_span();

        while matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::Comma | TokenKind::Newline
        ) {
            let tok = self.advance();
            last_span = tok.span;
            if tok.kind == TokenKind::Semicolon {
                print_result = false;
            }
        }

        (print_result, last_span)
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len() || self.peek().kind == TokenKind::Eof
    }
}
