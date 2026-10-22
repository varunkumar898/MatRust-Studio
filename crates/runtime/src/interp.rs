use std::collections::HashMap;

use matlab_diagnostics::{Diagnostic, Span};
use matlab_linalg as la;
use matlab_syntax::ast::*;
use matlab_values::{DenseMatrix, Value};

/// Variable environment. MATLAB variable names are case-sensitive.
#[derive(Debug, Default, Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.vars.insert(name.into(), value);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.vars.remove(name).is_some()
    }

    pub fn clear(&mut self) {
        self.vars.clear();
    }

    pub fn variable_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.vars.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }
}

/// A result value produced by a statement that did not suppress output
/// (`;` suppressed statements produce no record).
#[derive(Debug, Clone)]
pub struct StatementOutput {
    /// `None` for expression statements (printed as `ans`),
    /// `Some(name)` for assignments (printed as `name =`).
    pub label: Option<String>,
    pub value: Value,
}

/// Outputs of evaluating a single source chunk in an existing environment.
#[derive(Debug, Default)]
pub struct RunResult {
    /// Snapshot of the environment after execution (the caller's `Env` for
    /// `exec_source` stays authoritative; this is a convenience copy).
    pub env: Env,
    pub outputs: Vec<StatementOutput>,
    /// Lines printed by side-effectful builtins (e.g. `disp`).
    pub stdout: Vec<String>,
}

/// Lex + parse + execute `source` in a fresh environment.
pub fn run_source(source: &str) -> Result<RunResult, Diagnostic> {
    let tokens = matlab_syntax::Lexer::new(source).tokenize()?;
    let program = matlab_syntax::Parser::new(tokens).parse_program()?;
    let mut env = Env::new();
    exec_program_env(&mut env, program)
}

/// Lex + parse + execute `source` in the provided (persistent) environment.
pub fn exec_source(env: &mut Env, source: &str) -> Result<RunResult, Diagnostic> {
    let tokens = matlab_syntax::Lexer::new(source).tokenize()?;
    let program = matlab_syntax::Parser::new(tokens).parse_program()?;
    exec_program_env(env, program)
}

pub fn exec_program_env(env: &mut Env, program: Program) -> Result<RunResult, Diagnostic> {
    let mut outputs = Vec::new();
    let mut stdout = Vec::new();
    for stmt in program.statements {
        if let Some(out) = exec_statement(env, stmt, &mut stdout)? {
            outputs.push(out);
        }
    }
    Ok(RunResult {
        env: env.clone(),
        outputs,
        stdout,
    })
}

fn exec_statement(
    env: &mut Env,
    stmt: Stmt,
    stdout: &mut Vec<String>,
) -> Result<Option<StatementOutput>, Diagnostic> {
    match stmt {
        Stmt::Assignment {
            target,
            value,
            suppress_output,
            span,
        } => {
            let value = eval_expr(env, &value, stdout)?;
            match &target {
                AssignTarget::Ident(name, _) => {
                    env.set(name.clone(), value.clone());
                    if suppress_output {
                        Ok(None)
                    } else {
                        Ok(Some(StatementOutput {
                            label: Some(name.clone()),
                            value,
                        }))
                    }
                }
                AssignTarget::Index {
                    name,
                    indices,
                    span: _,
                } => {
                    assign_index(env, name, indices.clone(), value, span, stdout)?;
                    Ok(None)
                }
            }
        }
        Stmt::Expr {
            expr,
            suppress_output,
            ..
        } => {
            let value = eval_expr(env, &expr, stdout)?;
            if suppress_output {
                Ok(None)
            } else {
                Ok(Some(StatementOutput { label: None, value }))
            }
        }
    }
}

/// `A(1,2) = v` — with MATLAB-style automatic array growth.
fn assign_index(
    env: &mut Env,
    name: &str,
    indices: Vec<Expr>,
    value: Value,
    span: Span,
    stdout: &mut Vec<String>,
) -> Result<(), Diagnostic> {
    let span = Some(span);
    let existing = env
        .get(name)
        .map(|v| coerce_matrix(v.clone(), span))
        .unwrap_or_else(|| Ok(DenseMatrix::scalar(0.0)))?;

    let value_num = coerce_matrix(value, span)?;

    match indices.len() {
        1 => {
            // Linear indexing is column-major.
            let k = index_value(&indices[0], env, stdout)?;
            let n = existing.rows * existing.cols;
            let grown = if k <= n {
                let mut m = existing;
                let k0 = k - 1;
                let r0 = k0 % m.rows;
                let c0 = k0 / m.rows;
                m.data[r0 * m.cols + c0] = value_num.data[0];
                m
            } else {
                // Out-of-bounds linear assignment grows the array to a 1-by-k vector.
                let mut data = vec![0.0; k];
                for old_c in 0..existing.cols {
                    for old_r in 0..existing.rows {
                        data[old_c * existing.rows + old_r] =
                            existing.data[old_r * existing.cols + old_c];
                    }
                }
                data[k - 1] = value_num.data[0];
                DenseMatrix {
                    rows: 1,
                    cols: k,
                    data,
                }
            };
            env.set(name, Value::Matrix(grown));
            Ok(())
        }
        2 => {
            let i = index_value(&indices[0], env, stdout)?;
            let j = index_value(&indices[1], env, stdout)?;
            let v = value_num.data[0];
            let mut grown = existing;
            if i > grown.rows || j > grown.cols {
                let mut data = vec![0.0; i.max(grown.rows) * j.max(grown.cols)];
                for r_src in 0..grown.rows {
                    for c_src in 0..grown.cols {
                        data[r_src * j.max(grown.cols) + c_src] =
                            grown.data[r_src * grown.cols + c_src];
                    }
                }
                grown = DenseMatrix {
                    rows: i.max(grown.rows),
                    cols: j.max(grown.cols),
                    data,
                };
            }
            grown.set_2d(i, j, v, span)?;
            env.set(name, Value::Matrix(grown));
            Ok(())
        }
        n => Err(Diagnostic::error(
            format!(
                "Too many index expressions for '{}' ({} > 2). N-D indexing is not yet supported.",
                name, n
            ),
            span,
        )),
    }
}

/// Read access: `A(1,2)` / `A(3)` inside expressions
/// (parsed as calls to a variable name; variables shadow builtins).
fn call_or_index(
    env: &Env,
    name: &str,
    args: &[Expr],
    span: Span,
    stdout: &mut Vec<String>,
) -> Result<Value, Diagnostic> {
    if let Some(var) = env.get(name).cloned() {
        if args.is_empty() {
            return Ok(var);
        }
        let m = coerce_matrix(var, Some(span))?;
        match args.len() {
            1 => {
                let k = index_value(&args[0], env, stdout)?;
                Ok(Value::Matrix(DenseMatrix::scalar(
                    m.get_linear(k, Some(span))?,
                )))
            }
            2 => {
                let i = index_value(&args[0], env, stdout)?;
                let j = index_value(&args[1], env, stdout)?;
                Ok(Value::Matrix(DenseMatrix::scalar(
                    m.get_2d(i, j, Some(span))?,
                )))
            }
            n => Err(Diagnostic::error(
                format!("Too many index expressions for '{}' ({} > 2). N-D indexing is not yet supported.", name, n),
                Some(span),
            )),
        }
    } else {
        crate::builtins::call_builtin(name, args, env, span, stdout)
    }
}

/// Evaluate an index expression; must be a positive integer.
fn index_value(expr: &Expr, env: &Env, stdout: &mut Vec<String>) -> Result<usize, Diagnostic> {
    let span = Some(expr.span());
    let v = eval_expr(env, expr, stdout)?;
    let m = coerce_matrix(v, span)?;
    if !m.is_scalar() {
        return Err(Diagnostic::error(
            "Array indices must be positive integers or logical values.",
            span,
        ));
    }
    let f = m.data[0];
    if f.fract() != 0.0 || f < 1.0 {
        return Err(Diagnostic::error(
            "Array indices must be positive integers or logical values.",
            span,
        ));
    }
    Ok(f as usize)
}

fn coerce_matrix(v: Value, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    match v {
        Value::Matrix(m) => Ok(m),
        Value::Boolean(b) => Ok(DenseMatrix::scalar(if b { 1.0 } else { 0.0 })),
        Value::String(_) => Err(Diagnostic::error(
            "Conversion to double from char is not possible.",
            span,
        )),
    }
}

pub fn eval_expr(env: &Env, expr: &Expr, stdout: &mut Vec<String>) -> Result<Value, Diagnostic> {
    let span = Some(expr.span());
    match expr {
        Expr::Number(n, _) => Ok(Value::Matrix(DenseMatrix::scalar(*n))),
        Expr::StringLit(s, _) => Ok(Value::String(s.clone())),
        Expr::Variable(name, _) => match env.get(name) {
            Some(v) => Ok(v.clone()),
            None => {
                // Zero-argument constants (e.g. `pi`) work as bare identifiers.
                if crate::builtins::is_known_constant(name) {
                    crate::builtins::call_builtin(name, &[], env, span.unwrap_or_default(), stdout)
                } else {
                    Err(Diagnostic::error(
                        format!("Unrecognized function or variable '{}'.", name),
                        span,
                    ))
                }
            }
        },
        Expr::MatrixLit(rows, _) => {
            let mut out_rows = Vec::with_capacity(rows.len());
            for row in rows {
                let mut out_row = Vec::with_capacity(row.len());
                for element in row {
                    match eval_expr(env, element, stdout)? {
                        Value::Matrix(m) => {
                            if !m.is_scalar() {
                                return Err(Diagnostic::error(
                                    "Only scalar elements are supported inside matrix literals for now; nested concatenation is not yet supported.",
                                    span,
                                ));
                            }
                            out_row.push(m.data[0]);
                        }
                        Value::Boolean(b) => out_row.push(if b { 1.0 } else { 0.0 }),
                        Value::String(_) => {
                            return Err(Diagnostic::error(
                                "Conversion to double from char is not possible.",
                                span,
                            ));
                        }
                    }
                }
                out_rows.push(out_row);
            }
            Ok(Value::Matrix(DenseMatrix::from_rows(out_rows, span)?))
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let l = eval_expr(env, left, stdout)?;
            let r = eval_expr(env, right, stdout)?;
            apply_binary(*op, &l, &r, span)
        }
        Expr::Unary {
            op, expr: inner, ..
        } => {
            let v = eval_expr(env, inner, stdout)?;
            let m = coerce_matrix(v, span)?;
            let result = match op {
                UnaryOp::Pos => m,
                UnaryOp::Neg => la::negate(&m),
            };
            Ok(Value::Matrix(result))
        }
        Expr::Call { callee, args, .. } => call_or_index(env, callee, args, expr.span(), stdout),
        Expr::Transpose { expr: inner, .. } => {
            let v = eval_expr(env, inner, stdout)?;
            let m = coerce_matrix(v, span)?;
            Ok(Value::Matrix(la::transpose(&m)))
        }
    }
}

pub fn coerce_matrix_pub(v: Value, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    coerce_matrix(v, span)
}

pub fn eval_expr_pub(
    env: &Env,
    expr: &Expr,
    stdout: &mut Vec<String>,
) -> Result<Value, Diagnostic> {
    eval_expr(env, expr, stdout)
}

fn apply_binary(
    op: BinaryOp,
    l: &Value,
    r: &Value,
    span: Option<Span>,
) -> Result<Value, Diagnostic> {
    // char operands do not participate in arithmetic/comparison in this milestone
    if matches!(l, Value::String(_)) || matches!(r, Value::String(_)) {
        return Err(Diagnostic::error(
            "Conversion to double from char is not possible.",
            span,
        ));
    }

    let l = coerce_matrix(l.clone(), span)?;
    let r = coerce_matrix(r.clone(), span)?;

    let m = match op {
        BinaryOp::Add => la::add(&l, &r, span)?,
        BinaryOp::Sub => la::sub(&l, &r, span)?,
        BinaryOp::Mul => la::matmul(&l, &r, span)?,
        BinaryOp::Div => la::mrdivide(&l, &r, span)?,
        BinaryOp::DotMul => la::elementwise_mul(&l, &r, span)?,
        BinaryOp::DotDiv => la::elementwise_div(&l, &r, span)?,
        BinaryOp::Pow => {
            let p = r.as_scalar().ok_or_else(|| {
                Diagnostic::error("Exponent must be a scalar for matrix power.", span)
            })?;
            la::matrix_pow(&l, p, span)?
        }
        BinaryOp::DotPow => la::elementwise_pow(&l, &r, span)?,
        BinaryOp::Eq => la::compare(la::CompareOp::Eq, &l, &r, span)?,
        BinaryOp::Ne => la::compare(la::CompareOp::Ne, &l, &r, span)?,
        BinaryOp::Lt => la::compare(la::CompareOp::Lt, &l, &r, span)?,
        BinaryOp::Le => la::compare(la::CompareOp::Le, &l, &r, span)?,
        BinaryOp::Gt => la::compare(la::CompareOp::Gt, &l, &r, span)?,
        BinaryOp::Ge => la::compare(la::CompareOp::Ge, &l, &r, span)?,
    };
    Ok(Value::Matrix(m))
}
