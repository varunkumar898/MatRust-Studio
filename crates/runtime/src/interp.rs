use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use matlab_diagnostics::{Diagnostic, Span};
use matlab_linalg as la;
use matlab_syntax::ast::*;
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix, Value};

use crate::graphics::FigureState;
use crate::rng::Rng;

/// Persistent execution session: one environment plus graphics state.
/// The desktop worker owns exactly one `Session`; the CLI may create
/// sessions per invocation. Never create a new environment per command.
#[derive(Debug, Clone)]
pub struct Session {
    pub env: Env,
    pub graphics: crate::graphics::GraphicsState,
    pub rng: Rng,
    /// User-defined functions registered during session execution.
    pub function_table: HashMap<String, Stmt>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            env: Env::default(),
            graphics: crate::graphics::GraphicsState::default(),
            rng: Rng::default(),
            function_table: HashMap::new(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lex + parse + execute one source chunk in this session.
    /// `cancel`, when present, is checked between statements.
    pub fn exec(
        &mut self,
        source: &str,
        cancel: Option<&AtomicBool>,
    ) -> Result<RunResult, Diagnostic> {
        let tokens = matlab_syntax::Lexer::new(source).tokenize()?;
        let program = matlab_syntax::Parser::new(tokens).parse_program()?;
        let mut stdout = Vec::new();
        let mut io = ExecIo {
            stdout: &mut stdout,
            graphics: &mut self.graphics,
            cancel,
            rng: &mut self.rng,
        };
        let mut outputs = Vec::new();
        let mut interrupted = false;
        for stmt in program.statements {
            if let Some(cancel_flag) = io.cancel {
                if cancel_flag.load(Ordering::Relaxed) {
                    interrupted = true;
                    break;
                }
            }
            let res = exec_statement(&mut self.env, stmt, &mut io, 0)?;
            outputs.extend(res.outputs);
        }
        Ok(RunResult {
            env: self.env.clone(),
            outputs,
            stdout,
            figures: self.graphics.figures.clone(),
            interrupted,
        })
    }
}

/// I/O sinks threaded through evaluation: console output produced by
/// builtins, graphics commands, and the optional cancellation flag.
pub struct ExecIo<'a> {
    pub stdout: &'a mut Vec<String>,
    pub graphics: &'a mut crate::graphics::GraphicsState,
    pub cancel: Option<&'a AtomicBool>,
    pub rng: &'a mut Rng,
}

/// Control-flow signal propagated from a statement or block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Flow {
    #[default]
    Normal,
    Break,
    Continue,
    Return,
}

#[derive(Debug, Default)]
pub struct StmtResult {
    pub outputs: Vec<StatementOutput>,
    pub flow: Flow,
}

/// Variable environment. MATLAB variable names are case-sensitive.
#[derive(Debug, Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
    /// User-defined functions registered in this environment/session.
    pub function_table: HashMap<String, Stmt>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            function_table: HashMap::new(),
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

/// One row for a workspace summary table.
#[derive(Debug, Clone, PartialEq)]
pub struct VarRow {
    pub name: String,
    pub kind: String,
    pub rows: usize,
    pub cols: usize,
    pub elements: usize,
    pub preview: String,
}

/// Human-oriented snapshot of every variable in `env`.
pub fn variable_rows(env: &Env) -> Vec<VarRow> {
    let mut rows = Vec::new();
    for name in env.variable_names() {
        let value = &env.vars[name];
        let row = match value {
            Value::Matrix(m) => VarRow {
                name: name.to_string(),
                kind: if m.is_scalar() {
                    "double".into()
                } else {
                    "matrix".into()
                },
                rows: m.rows,
                cols: m.cols,
                elements: m.rows * m.cols,
                preview: preview_matrix(m),
            },
            Value::ComplexMatrix(c) => VarRow {
                name: name.to_string(),
                kind: if c.is_scalar() {
                    "complex double".into()
                } else {
                    "complex matrix".into()
                },
                rows: c.rows,
                cols: c.cols,
                elements: c.rows * c.cols,
                preview: preview_complex_matrix(c),
            },
            Value::String(s) => VarRow {
                name: name.to_string(),
                kind: "char".into(),
                rows: 1,
                cols: s.chars().count(),
                elements: s.chars().count(),
                preview: s.chars().take(48).collect(),
            },
            Value::Boolean(b) => VarRow {
                name: name.to_string(),
                kind: "logical".into(),
                rows: 1,
                cols: 1,
                elements: 1,
                preview: if *b { "1".into() } else { "0".into() },
            },
        };
        rows.push(row);
    }
    rows
}

fn preview_complex_matrix(c: &ComplexDenseMatrix) -> String {
    if c.is_scalar() {
        return c.format_matlab();
    }
    let r_max = 2.min(c.rows);
    let c_max = 4.min(c.cols);
    let truncated = c.rows > r_max || c.cols > c_max;
    let sub_rows: Vec<Vec<Complex64>> = (0..r_max)
        .map(|r| (0..c_max).map(|col| c.data[r * c.cols + col]).collect())
        .collect();
    let sub = ComplexDenseMatrix::from_rows(sub_rows, None).expect("2x4 slice is valid");
    let mut body = sub.format_matlab().replace('\n', "; ");
    if truncated {
        body.push_str(" …");
    }
    if c.cols == 1 && c.rows > 1 {
        format!("[{}; …]", body.strip_suffix(" …").unwrap_or(&body))
    } else {
        format!("[{}]", body)
    }
}

fn preview_matrix(m: &DenseMatrix) -> String {
    if m.is_scalar() {
        return m.format_matlab();
    }
    let r_max = 2.min(m.rows);
    let c_max = 4.min(m.cols);
    let truncated = m.rows > r_max || m.cols > c_max;
    let sub_rows: Vec<Vec<f64>> = (0..r_max)
        .map(|r| (0..c_max).map(|c| m.data[r * m.cols + c]).collect())
        .collect();
    let sub = DenseMatrix::from_rows(sub_rows, None).expect("2x4 slice is valid");
    let mut body = sub.format_matlab().replace('\n', "; ");
    if truncated {
        body.push_str(" …");
    }
    if m.cols == 1 && m.rows > 1 {
        // column vector: show as [a; b; …] with a transposition-free hint
        format!("[{}; …]", body.strip_suffix(" …").unwrap_or(&body))
    } else {
        format!("[{}]", body)
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
    /// Final graphics state (figures created/modified by this chunk and any
    /// previously existing ones in the same session).
    pub figures: Vec<FigureState>,
    /// True when execution stopped early because the cancel flag was set.
    pub interrupted: bool,
}

/// Lex + parse + execute `source` in a fresh environment.
pub fn run_source(source: &str) -> Result<RunResult, Diagnostic> {
    let mut session = Session::new();
    session.exec(source, None)
}

/// Lex + parse + execute `source` in the provided (persistent) environment.
/// Graphics state is local to this call.
pub fn exec_source(env: &mut Env, source: &str) -> Result<RunResult, Diagnostic> {
    let tokens = matlab_syntax::Lexer::new(source).tokenize()?;
    let program = matlab_syntax::Parser::new(tokens).parse_program()?;
    let mut stdout = Vec::new();
    let mut graphics = crate::graphics::GraphicsState::default();
    let mut rng = Rng::default();
    let mut io = ExecIo {
        stdout: &mut stdout,
        graphics: &mut graphics,
        cancel: None,
        rng: &mut rng,
    };
    let mut outputs = Vec::new();
    for stmt in program.statements {
        let res = exec_statement(env, stmt.clone(), &mut io, 0)?;
        outputs.extend(res.outputs);
    }
    Ok(RunResult {
        env: env.clone(),
        outputs,
        stdout,
        figures: graphics.figures,
        interrupted: false,
    })
}

pub fn exec_program_env(env: &mut Env, program: Program) -> Result<RunResult, Diagnostic> {
    let mut outputs = Vec::new();
    let mut stdout = Vec::new();
    let mut graphics = crate::graphics::GraphicsState::default();
    let mut rng = Rng::default();
    let mut io = ExecIo {
        stdout: &mut stdout,
        graphics: &mut graphics,
        cancel: None,
        rng: &mut rng,
    };
    for stmt in program.statements {
        let res = exec_statement(env, stmt.clone(), &mut io, 0)?;
        outputs.extend(res.outputs);
    }
    Ok(RunResult {
        env: env.clone(),
        outputs,
        stdout,
        figures: graphics.figures,
        interrupted: false,
    })
}

fn exec_statement(
    env: &mut Env,
    stmt: Stmt,
    io: &mut ExecIo<'_>,
    loop_depth: usize,
) -> Result<StmtResult, Diagnostic> {
    match stmt {
        Stmt::Assignment {
            target,
            value,
            suppress_output,
            span,
        } => {
            // Handle multi-output assignment specially.
            if let AssignTarget::Multi(names, _) = &target {
                let (func_name, func_args) = match &value {
                    Expr::Call { callee, args, .. } => (callee.clone(), args.clone()),
                    _ => return Err(Diagnostic::error(
                        "Multi-output assignment requires a function call on the right-hand side.",
                        Some(span),
                    )),
                };
                let nargout = names.len();
                let outputs = crate::builtins::call_builtin_multi(
                    &func_name, &func_args, env, span, io, nargout,
                )?;
                // Assign each output to the corresponding variable.
                for (i, (name, _)) in names.iter().enumerate() {
                    if i < outputs.len() {
                        env.set(name.clone(), outputs[i].clone());
                    } else {
                        env.set(name.clone(), Value::Matrix(DenseMatrix::empty()));
                    }
                }
                let mut stmt_outputs = Vec::new();
                if !suppress_output {
                    for (i, (name, _)) in names.iter().enumerate() {
                        if i < outputs.len() {
                            stmt_outputs.push(StatementOutput {
                                label: Some(name.clone()),
                                value: outputs[i].clone(),
                            });
                        }
                    }
                }
                return Ok(StmtResult {
                    outputs: stmt_outputs,
                    flow: Flow::Normal,
                });
            }
            let value = eval_expr(env, &value, io)?;
            match &target {
                AssignTarget::Ident(name, _) => {
                    env.set(name.clone(), value.clone());
                    if suppress_output {
                        Ok(StmtResult {
                            outputs: vec![],
                            flow: Flow::Normal,
                        })
                    } else {
                        Ok(StmtResult {
                            outputs: vec![StatementOutput {
                                label: Some(name.clone()),
                                value,
                            }],
                            flow: Flow::Normal,
                        })
                    }
                }
                AssignTarget::Index {
                    name,
                    indices,
                    span: _,
                } => {
                    assign_index(env, name, indices.clone(), value, span, io)?;
                    Ok(StmtResult::default())
                }
                AssignTarget::Multi(_, _) => unreachable!(),
            }
        }
        Stmt::Expr {
            expr,
            suppress_output,
            ..
        } => {
            let value = eval_expr(env, &expr, io)?;
            if suppress_output {
                Ok(StmtResult {
                    outputs: vec![],
                    flow: Flow::Normal,
                })
            } else {
                Ok(StmtResult {
                    outputs: vec![StatementOutput { label: None, value }],
                    flow: Flow::Normal,
                })
            }
        }
        // Control flow
        Stmt::If {
            branches,
            else_body,
            ..
        } => {
            for (cond, body) in branches {
                let cond_val = eval_expr(env, &cond, io)?;
                if condition_is_true(&cond_val, None)? {
                    return exec_block(env, &body, io, loop_depth);
                }
            }
            if let Some(body) = else_body {
                return exec_block(env, &body, io, loop_depth);
            }
            Ok(StmtResult::default())
        }
        Stmt::While { cond, body, .. } => {
            let mut last_outputs = Vec::new();
            loop {
                if let Some(c) = io.cancel {
                    if c.load(Ordering::Relaxed) {
                        break;
                    }
                }
                let cond_val = eval_expr(env, &cond, io)?;
                if !condition_is_true(&cond_val, None)? {
                    break;
                }
                let block = exec_block(env, &body, io, loop_depth + 1)?;
                last_outputs = block.outputs;
                match block.flow {
                    Flow::Break => break,
                    Flow::Continue => continue,
                    _ => {}
                }
            }
            Ok(StmtResult {
                outputs: last_outputs,
                flow: Flow::Normal,
            })
        }
        Stmt::For {
            var, values, body, ..
        } => {
            let iter_vals = eval_for_iterable(env, &values, io)?;
            let mut last_outputs = Vec::new();
            for v in iter_vals {
                if let Some(c) = io.cancel {
                    if c.load(Ordering::Relaxed) {
                        break;
                    }
                }
                env.set(var.clone(), v);
                let block = exec_block(env, &body, io, loop_depth + 1)?;
                last_outputs = block.outputs;
                match block.flow {
                    Flow::Break => break,
                    Flow::Continue => continue,
                    _ => {}
                }
            }
            Ok(StmtResult {
                outputs: last_outputs,
                flow: Flow::Normal,
            })
        }
        Stmt::Break { .. } => {
            if loop_depth == 0 {
                return Err(Diagnostic::error(
                    "A 'break' statement may only appear inside a loop.",
                    None,
                ));
            }
            Ok(StmtResult {
                outputs: Vec::new(),
                flow: Flow::Break,
            })
        }
        Stmt::Continue { .. } => {
            if loop_depth == 0 {
                return Err(Diagnostic::error(
                    "A 'continue' statement may only appear inside a loop.",
                    None,
                ));
            }
            Ok(StmtResult {
                outputs: Vec::new(),
                flow: Flow::Continue,
            })
        }
        Stmt::Return { .. } => Ok(StmtResult {
            outputs: Vec::new(),
            flow: Flow::Return,
        }),
        Stmt::Switch {
            expr,
            cases,
            otherwise,
            ..
        } => {
            let val = eval_expr(env, &expr, io)?;
            let target = coerce_matrix(val, Some(expr.span()))?;
            let mut outputs = Vec::new();
            for (labels, body) in cases {
                let mut hit = false;
                for label in labels {
                    let lv = eval_expr(env, &label, io)?;
                    let cand = coerce_matrix(lv, Some(label.span()))?;
                    if target.rows == cand.rows
                        && target.cols == cand.cols
                        && target
                            .data
                            .iter()
                            .zip(cand.data.iter())
                            .all(|(x, y)| (x - y).abs() < 1e-12)
                    {
                        hit = true;
                        break;
                    }
                }
                if hit {
                    let res = exec_block(env, &body, io, loop_depth)?;
                    outputs.extend(res.outputs);
                    return Ok(StmtResult {
                        outputs,
                        flow: res.flow,
                    });
                }
            }
            if let Some(body) = otherwise {
                let res = exec_block(env, &body, io, loop_depth)?;
                outputs.extend(res.outputs);
                return Ok(StmtResult {
                    outputs,
                    flow: res.flow,
                });
            }
            Ok(StmtResult {
                outputs,
                flow: Flow::Normal,
            })
        }
        Stmt::FunctionDef { outputs, name, name_span, params, body, span } => {
            env.function_table.insert(
                name.clone(),
                Stmt::FunctionDef {
                    outputs: outputs.clone(),
                    name: name.clone(),
                    name_span: name_span.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    span: span.clone(),
                },
            );
            Ok(StmtResult { outputs: Vec::new(), flow: Flow::Normal })
        }
    }
}

/// Execute a user-defined function in an isolated local environment.
/// Stage 3 partial: evaluates arguments, binds named parameters, executes
/// the body via exec_statement, and returns the first declared output if set.
/// Multi-output full assignment is not yet integrated; this implementation
/// returns a single value for each declared function (first only when multiple).
fn exec_user_function(
    parent_env: &Env,
    func: Stmt,
    args: &[Expr],
    span: Span,
    io: &mut ExecIo<'_>,
) -> Result<Value, Diagnostic> {
    if let Stmt::FunctionDef { outputs, params, body, .. } = func {
        let mut local_env = Env::new();
        // Bind parameters by evaluating arguments against caller environment.
        for (i, (param_name, _)) in params.iter().enumerate() {
            let value = if i < args.len() {
                eval_expr_pub(parent_env, &args[i], io)?
            } else {
                Value::Matrix(DenseMatrix::scalar(0.0))
            };
            local_env.set(param_name.clone(), value);
        }
        // Inject nargin / nargout for basic test coverage.
        local_env.set("nargin".to_string(), Value::Matrix(DenseMatrix::scalar(args.len() as f64)));
        local_env.set("nargout".to_string(), Value::Matrix(DenseMatrix::scalar(outputs.len().max(1) as f64)));
        // Execute body statements.
        let mut collected = Vec::new();
        for stmt in body {
            let res = exec_statement(&mut local_env, stmt.clone(), io, 0)?;
            if res.flow == Flow::Return {
                collected = res.outputs;
                break;
            }
            collected = res.outputs;
        }
        if outputs.is_empty() {
            return Ok(Value::Matrix(DenseMatrix::scalar(0.0)));
        }
        // Return the first declared output if present.
        let out_name = outputs[0].0.clone();
        if let Some(val) = local_env.get(&out_name) {
            return Ok(val.clone());
        } else {
            return Err(Diagnostic::error(
                format!("Output argument '{}' not assigned.", out_name),
                Some(span),
            ));
        }
    } else {
        Err(Diagnostic::error("User-defined function definition expected.", Some(span)))
    }
}

/// Execute a block of statements, propagating flow signals.
fn exec_block(
    env: &mut Env,
    stmts: &[Stmt],
    io: &mut ExecIo<'_>,
    loop_depth: usize,
) -> Result<StmtResult, Diagnostic> {
    let mut outputs = Vec::new();
    let mut flow = Flow::Normal;
    for stmt in stmts {
        if let Some(c) = io.cancel {
            if c.load(Ordering::Relaxed) {
                break;
            }
        }
        let res = exec_statement(env, stmt.clone(), io, loop_depth)?;
        outputs.extend(res.outputs);
        match res.flow {
            Flow::Normal => {}
            other => {
                flow = other;
                break;
            }
        }
    }
    Ok(StmtResult { outputs, flow })
}

/// MATLAB `for` iteration: iterate over columns of the array.
fn eval_for_iterable(
    env: &Env,
    values: &Expr,
    io: &mut ExecIo<'_>,
) -> Result<Vec<Value>, Diagnostic> {
    let v = eval_expr(env, &values, io)?;
    let m = v.as_matrix().ok_or_else(|| {
        Diagnostic::error("FOR loop values must be a numeric or logical array.", None)
    })?;
    // Iterate over columns (MATLAB convention).
    let mut out = Vec::with_capacity(m.cols);
    for c in 0..m.cols {
        let col: Vec<f64> = (0..m.rows).map(|r| m.data[r * m.cols + c]).collect();
        if col.len() == 1 {
            out.push(Value::Matrix(DenseMatrix::scalar(col[0])));
        } else {
            out.push(Value::Matrix(DenseMatrix {
                rows: col.len(),
                cols: 1,
                data: col,
            }));
        }
    }
    Ok(out)
}

/// Evaluate a condition to a boolean.
fn condition_is_true(val: &Value, _span: Option<Span>) -> Result<bool, Diagnostic> {
    match val {
        Value::Matrix(m) => {
            if m.data.is_empty() {
                return Ok(false);
            }
            if m.data.len() == 1 {
                Ok(m.data[0] != 0.0)
            } else {
                // For arrays, MATLAB requires scalar logical; error for non-scalar.
                Err(Diagnostic::error(
                    "Condition must be a scalar logical value.",
                    None,
                ))
            }
        }
        Value::ComplexMatrix(c) => {
            if c.data.is_empty() {
                return Ok(false);
            }
            if c.data.len() == 1 {
                let z = c.data[0];
                Ok(z.re != 0.0 || z.im != 0.0)
            } else {
                Err(Diagnostic::error(
                    "Condition must be a scalar logical value.",
                    None,
                ))
            }
        }
        Value::Boolean(b) => Ok(*b),
        Value::String(_) => Err(Diagnostic::error(
            "Condition must be a scalar logical value.",
            None,
        )),
    }
}

/// `A(1,2) = v` — with MATLAB-style automatic array growth.
fn assign_index(
    env: &mut Env,
    name: &str,
    indices: Vec<Expr>,
    value: Value,
    span: Span,
    io: &mut ExecIo<'_>,
) -> Result<(), Diagnostic> {
    let span_op = Some(span);

    let is_complex_assignment = match (env.get(name), &value) {
        (Some(Value::ComplexMatrix(_)), _) => true,
        (_, Value::ComplexMatrix(_)) => true,
        _ => false,
    };

    if is_complex_assignment {
        let existing = env
            .get(name)
            .map(|v| {
                v.to_complex_matrix().ok_or_else(|| {
                    Diagnostic::error("Cannot index into non-numeric array.", span_op)
                })
            })
            .unwrap_or_else(|| Ok(ComplexDenseMatrix::scalar(Complex64::zero())))?;

        let val_c = value
            .to_complex_matrix()
            .and_then(|m| m.as_scalar())
            .ok_or_else(|| {
                Diagnostic::error(
                    "Only scalar values can be assigned into array elements.",
                    span_op,
                )
            })?;

        let shape = Some((existing.rows, existing.cols));

        match indices.len() {
            1 => {
                let k = index_value(&indices[0], env, shape, 0, io)?;
                let n = existing.rows * existing.cols;
                let grown = if k <= n {
                    let mut m = existing;
                    let k0 = k - 1;
                    let r0 = k0 % m.rows;
                    let c0 = k0 / m.rows;
                    m.data[r0 * m.cols + c0] = val_c;
                    m
                } else {
                    let mut data = vec![Complex64::zero(); k];
                    for old_c in 0..existing.cols {
                        for old_r in 0..existing.rows {
                            data[old_c * existing.rows + old_r] =
                                existing.data[old_r * existing.cols + old_c];
                        }
                    }
                    data[k - 1] = val_c;
                    ComplexDenseMatrix {
                        rows: 1,
                        cols: k,
                        data,
                    }
                };
                env.set(name, Value::ComplexMatrix(grown));
                Ok(())
            }
            2 => {
                let i = index_value(&indices[0], env, shape, 2, io)?;
                let j = index_value(&indices[1], env, shape, 1, io)?;
                let mut grown = existing;
                if i > grown.rows || j > grown.cols {
                    let mut data = vec![Complex64::zero(); i.max(grown.rows) * j.max(grown.cols)];
                    for r_src in 0..grown.rows {
                        for c_src in 0..grown.cols {
                            data[r_src * j.max(grown.cols) + c_src] =
                                grown.data[r_src * grown.cols + c_src];
                        }
                    }
                    grown = ComplexDenseMatrix {
                        rows: i.max(grown.rows),
                        cols: j.max(grown.cols),
                        data,
                    };
                }
                grown.set_2d(i, j, val_c, span_op)?;
                env.set(name, Value::ComplexMatrix(grown));
                Ok(())
            }
            n => Err(Diagnostic::error(
                format!(
                    "Too many index expressions for '{}' ({} > 2). N-D indexing is not yet supported.",
                    name, n
                ),
                span_op,
            )),
        }
    } else {
        let existing = env
            .get(name)
            .map(|v| coerce_matrix(v.clone(), span_op))
            .unwrap_or_else(|| Ok(DenseMatrix::scalar(0.0)))?;

        let value_num = coerce_matrix(value, span_op)?;
        let shape = Some((existing.rows, existing.cols));

        match indices.len() {
            1 => {
                // Linear indexing is column-major.
                let k = index_value(&indices[0], env, shape, 0, io)?;
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
                let i = index_value(&indices[0], env, shape, 2, io)?;
                let j = index_value(&indices[1], env, shape, 1, io)?;
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
                grown.set_2d(i, j, v, span_op)?;
                env.set(name, Value::Matrix(grown));
                Ok(())
            }
            n => Err(Diagnostic::error(
                format!(
                    "Too many index expressions for '{}' ({} > 2). N-D indexing is not yet supported.",
                    name, n
                ),
                span_op,
            )),
        }
    }
}

/// Read access: `A(1,2)` / `A(3)` inside expressions
/// (parsed as calls to a variable name; variables shadow builtins).
fn call_or_index(
    env: &Env,
    name: &str,
    args: &[Expr],
    span: Span,
    io: &mut ExecIo<'_>,
) -> Result<Value, Diagnostic> {
    if let Some(var) = env.get(name).cloned() {
        if args.is_empty() {
            return Ok(var);
        }
        match var {
            Value::ComplexMatrix(c) => {
                let shape = Some((c.rows, c.cols));
                match args.len() {
                    1 => match &args[0] {
                        Expr::ColonAll(_) => {
                            let data = c.to_col_major();
                            Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                                rows: 1,
                                cols: data.len(),
                                data,
                            }))
                        }
                        Expr::Range { .. } => {
                            let indices = eval_range_indices(&args[0], shape, 0, env, io)?;
                            let mut data = Vec::with_capacity(indices.len());
                            for &k in &indices {
                                data.push(c.get_linear(k, Some(span))?);
                            }
                            Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                                rows: 1,
                                cols: data.len(),
                                data,
                            }))
                        }
                        _ => {
                            let k = index_value(&args[0], env, shape, 0, io)?;
                            Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(
                                c.get_linear(k, Some(span))?,
                            )))
                        }
                    },
                    2 => {
                        let i_expr = &args[0];
                        let j_expr = &args[1];
                        let i_is_range = matches!(i_expr, Expr::ColonAll(_) | Expr::Range { .. });
                        let j_is_range = matches!(j_expr, Expr::ColonAll(_) | Expr::Range { .. });
                        if i_is_range || j_is_range {
                            let row_indices = if i_is_range {
                                if let Expr::ColonAll(_) = i_expr {
                                    (1..=c.rows).collect()
                                } else {
                                    eval_range_indices(i_expr, shape, 2, env, io)?
                                }
                            } else {
                                let k = index_value(i_expr, env, shape, 2, io)?;
                                vec![k]
                            };
                            let col_indices = if j_is_range {
                                if let Expr::ColonAll(_) = j_expr {
                                    (1..=c.cols).collect()
                                } else {
                                    eval_range_indices(j_expr, shape, 1, env, io)?
                                }
                            } else {
                                let k = index_value(j_expr, env, shape, 1, io)?;
                                vec![k]
                            };
                            let mut data = Vec::with_capacity(row_indices.len() * col_indices.len());
                            for &ri in &row_indices {
                                for &cj in &col_indices {
                                    data.push(c.get_2d(ri, cj, Some(span))?);
                                }
                            }
                            Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                                rows: row_indices.len(),
                                cols: col_indices.len(),
                                data,
                            }))
                        } else {
                            let i = index_value(i_expr, env, shape, 2, io)?;
                            let j = index_value(j_expr, env, shape, 1, io)?;
                            Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(
                                c.get_2d(i, j, Some(span))?,
                            )))
                        }
                    }
                    n => Err(Diagnostic::error(
                        format!(
                            "Too many index expressions for '{}' ({} > 2). N-D indexing is not yet supported.",
                            name, n
                        ),
                        Some(span),
                    )),
                }
            }
            _ => {
                let m = coerce_matrix(var, Some(span))?;
                let shape = Some((m.rows, m.cols));
                match args.len() {
                    1 => match &args[0] {
                        Expr::ColonAll(_) => {
                            let data = m.to_col_major();
                            Ok(Value::Matrix(DenseMatrix {
                                rows: 1,
                                cols: data.len(),
                                data,
                            }))
                        }
                        Expr::Range { .. } => {
                            let indices = eval_range_indices(&args[0], shape, 0, env, io)?;
                            let mut data = Vec::with_capacity(indices.len());
                            for &k in &indices {
                                data.push(m.get_linear(k, Some(span))?);
                            }
                            Ok(Value::Matrix(DenseMatrix {
                                rows: 1,
                                cols: data.len(),
                                data,
                            }))
                        }
                        _ => {
                            let k = index_value(&args[0], env, shape, 0, io)?;
                            Ok(Value::Matrix(DenseMatrix::scalar(
                                m.get_linear(k, Some(span))?,
                            )))
                        }
                    },
                    2 => {
                        let i_expr = &args[0];
                        let j_expr = &args[1];
                        let i_is_range = matches!(i_expr, Expr::ColonAll(_) | Expr::Range { .. });
                        let j_is_range = matches!(j_expr, Expr::ColonAll(_) | Expr::Range { .. });
                        if i_is_range || j_is_range {
                            let row_indices = if i_is_range {
                                if let Expr::ColonAll(_) = i_expr {
                                    (1..=m.rows).collect()
                                } else {
                                    eval_range_indices(i_expr, shape, 2, env, io)?
                                }
                            } else {
                                let k = index_value(i_expr, env, shape, 2, io)?;
                                vec![k]
                            };
                            let col_indices = if j_is_range {
                                if let Expr::ColonAll(_) = j_expr {
                                    (1..=m.cols).collect()
                                } else {
                                    eval_range_indices(j_expr, shape, 1, env, io)?
                                }
                            } else {
                                let k = index_value(j_expr, env, shape, 1, io)?;
                                vec![k]
                            };
                            let mut data = Vec::with_capacity(row_indices.len() * col_indices.len());
                            for &ri in &row_indices {
                                for &cj in &col_indices {
                                    data.push(m.get_2d(ri, cj, Some(span))?);
                                }
                            }
                            Ok(Value::Matrix(DenseMatrix {
                                rows: row_indices.len(),
                                cols: col_indices.len(),
                                data,
                            }))
                        } else {
                            let i = index_value(i_expr, env, shape, 2, io)?;
                            let j = index_value(j_expr, env, shape, 1, io)?;
                            Ok(Value::Matrix(DenseMatrix::scalar(
                                m.get_2d(i, j, Some(span))?,
                            )))
                        }
                    }
                    n => Err(Diagnostic::error(
                        format!(
                            "Too many index expressions for '{}' ({} > 2). N-D indexing is not yet supported.",
                            name, n
                        ),
                        Some(span),
                    )),
                }
            }
        }
    } else if let Some(func_stmt) = env.function_table.get(name).cloned() {
        // User-defined function call (Stage 3 partial: single-output scalar basic).
        exec_user_function(env, func_stmt, args, span, io)
    } else {
        crate::builtins::call_builtin(name, args, env, span, io)
    }
}

/// Evaluate an index expression; must be a positive integer.
fn eval_range_indices(
    expr: &Expr,
    shape: Option<(usize, usize)>,
    dim: usize,
    env: &Env,
    io: &mut ExecIo<'_>,
) -> Result<Vec<usize>, Diagnostic> {
    if let Expr::Range {
        start, step, stop, ..
    } = expr
    {
        let dim_size = match shape {
            Some((rows, cols)) => match dim {
                0 => (rows * cols).max(1),
                1 => cols,
                _ => rows,
            },
            None => 1,
        };
        // Resolve End in start/stop/step.
        let resolved_start = resolve_end_expr(start, dim_size);
        let resolved_stop = resolve_end_expr(stop, dim_size);
        let resolved_step = step.as_ref().map(|s| resolve_end_expr(s, dim_size));

        let s_val = eval_simple_scalar(&resolved_start, env, io)?;
        let e_val = eval_simple_scalar(&resolved_stop, env, io)?;
        let step_val = match resolved_step {
            Some(s_expr) => eval_simple_scalar(&s_expr, env, io)?,
            None => 1.0,
        };
        if step_val == 0.0 {
            return Err(Diagnostic::error("Step cannot be zero.", None));
        }
        let mut indices = Vec::new();
        if step_val > 0.0 {
            let mut k = s_val;
            while k <= e_val + 1e-9 {
                if k >= 1.0 {
                    indices.push(k as usize);
                }
                k += step_val;
            }
        } else {
            let mut k = s_val;
            while k >= e_val - 1e-9 {
                if k >= 1.0 {
                    indices.push(k as usize);
                }
                k += step_val;
            }
        }
        Ok(indices)
    } else {
        // Not a range - evaluate as single index.
        let k = index_value(expr, env, shape, dim, io)?;
        Ok(vec![k])
    }
}

fn resolve_end_expr(expr: &Expr, dim_size: usize) -> Expr {
    match expr {
        Expr::End(_) => Expr::Number(dim_size as f64, expr.span()),
        Expr::Binary {
            op,
            left,
            right,
            span,
        } => {
            // Handle end-1, end+1, etc.
            let new_left = Box::new(resolve_end_expr(left, dim_size));
            let new_right = Box::new(resolve_end_expr(right, dim_size));
            Expr::Binary {
                op: *op,
                left: new_left,
                right: new_right,
                span: *span,
            }
        }
        other => other.clone(),
    }
}

fn eval_simple_scalar(expr: &Expr, env: &Env, io: &mut ExecIo<'_>) -> Result<f64, Diagnostic> {
    let v = eval_expr(env, &expr, io)?;
    let m = coerce_matrix(v, None)?;
    m.as_scalar()
        .ok_or_else(|| Diagnostic::error("Expected scalar.", None))
}

fn index_value(
    expr: &Expr,
    env: &Env,
    shape: Option<(usize, usize)>,
    dim: usize,
    io: &mut ExecIo<'_>,
) -> Result<usize, Diagnostic> {
    let span = Some(expr.span());
    match expr {
        Expr::End(_) => {
            let (rows, cols) = match shape {
                Some(s) => s,
                None => return Ok(1),
            };
            return Ok(match dim {
                0 => (rows * cols).max(1),
                1 => cols,
                _ => rows,
            });
        }
        Expr::ColonAll(_) => {
            let (rows, cols) = match shape {
                Some(s) => s,
                None => return Ok(1),
            };
            return Ok(match dim {
                0 => (rows * cols).max(1),
                1 => cols,
                _ => rows,
            });
        }
        _ => {}
    }
    let v = eval_expr(env, &expr, io)?;
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
        Value::ComplexMatrix(c) => {
            if c.is_all_real() {
                Ok(c.to_real_matrix())
            } else {
                Err(Diagnostic::error(
                    "Operation expects a real array or scalar, but a complex value with non-zero imaginary component was provided.",
                    span,
                ))
            }
        }
        Value::Boolean(b) => Ok(DenseMatrix::scalar(if b { 1.0 } else { 0.0 })),
        Value::String(_) => Err(Diagnostic::error(
            "Conversion to double from char is not possible.",
            span,
        )),
    }
}

pub fn coerce_matrix_pub(v: Value, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    coerce_matrix(v, span)
}

pub fn eval_expr_pub(env: &Env, expr: &Expr, io: &mut ExecIo<'_>) -> Result<Value, Diagnostic> {
    eval_expr(env, &expr, io)
}

pub fn eval_expr(env: &Env, expr: &Expr, io: &mut ExecIo<'_>) -> Result<Value, Diagnostic> {
    let span = Some(expr.span());
    match expr {
        Expr::Number(n, _) => Ok(Value::Matrix(DenseMatrix::scalar(*n))),
        Expr::Imaginary(n, _) => Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(
            Complex64::new(0.0, *n),
        ))),
        Expr::StringLit(s, _) => Ok(Value::String(s.clone())),
        Expr::Variable(name, _) => match env.get(name) {
            Some(v) => Ok(v.clone()),
            None => {
                // An unknown bare identifier is treated as a zero-argument
                // builtin call (e.g. `pi`, `figure`, `clf`, `close`, `i`, `j`).
                // Unknown names fall into the builtin dispatcher's own
                // "Unrecognized function or variable" message, so the error
                // text for undefined variables is unchanged.
                crate::builtins::call_builtin(name, &[], env, expr.span(), io)
            }
        },
        Expr::MatrixLit(rows, _) => {
            let mut evaled_rows: Vec<Vec<Value>> = Vec::with_capacity(rows.len());
            let mut has_complex = false;
            for row in rows {
                let mut evaled_row = Vec::with_capacity(row.len());
                for element in row {
                    let val = eval_expr(env, element, io)?;
                    if val.is_complex() {
                        has_complex = true;
                    }
                    evaled_row.push(val);
                }
                evaled_rows.push(evaled_row);
            }

            if has_complex {
                let mut out_rows = Vec::with_capacity(evaled_rows.len());
                for evaled_row in evaled_rows {
                    let mut out_row = Vec::with_capacity(evaled_row.len());
                    for v in evaled_row {
                        match v {
                            Value::Matrix(m) => {
                                if !m.is_scalar() {
                                    return Err(Diagnostic::error(
                                        "Only scalar elements are supported inside matrix literals for now; nested concatenation is not yet supported.",
                                        span,
                                    ));
                                }
                                out_row.push(Complex64::real(m.data[0]));
                            }
                            Value::ComplexMatrix(c) => {
                                if !c.is_scalar() {
                                    return Err(Diagnostic::error(
                                        "Only scalar elements are supported inside matrix literals for now; nested concatenation is not yet supported.",
                                        span,
                                    ));
                                }
                                out_row.push(c.data[0]);
                            }
                            Value::Boolean(b) => {
                                out_row.push(Complex64::real(if b { 1.0 } else { 0.0 }))
                            }
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
                Ok(Value::ComplexMatrix(ComplexDenseMatrix::from_rows(
                    out_rows, span,
                )?))
            } else {
                let mut out_rows = Vec::with_capacity(evaled_rows.len());
                for evaled_row in evaled_rows {
                    let mut out_row = Vec::with_capacity(evaled_row.len());
                    for v in evaled_row {
                        match v {
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
                            Value::ComplexMatrix(_) => unreachable!(),
                        }
                    }
                    out_rows.push(out_row);
                }
                Ok(Value::Matrix(DenseMatrix::from_rows(out_rows, span)?))
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let l = eval_expr(env, left, io)?;
            let r = eval_expr(env, right, io)?;
            apply_binary(*op, &l, &r, span)
        }
        Expr::Unary {
            op, expr: inner, ..
        } => {
            let v = eval_expr(env, inner, io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let result = match op {
                        UnaryOp::Pos => c,
                        UnaryOp::Neg => la::complex::negate(&c),
                    };
                    Ok(Value::ComplexMatrix(result))
                }
                _ => {
                    let m = coerce_matrix(v, span)?;
                    let result = match op {
                        UnaryOp::Pos => m,
                        UnaryOp::Neg => la::negate(&m),
                    };
                    Ok(Value::Matrix(result))
                }
            }
        }
        Expr::Call { callee, args, .. } => call_or_index(env, callee, args, expr.span(), io),
        Expr::Range {
            start, step, stop, ..
        } => {
            let s = eval_expr(env, start, io)?;
            let e = eval_expr(env, stop, io)?;
            let step_val = match step {
                Some(s_expr) => {
                    let sv = eval_expr(env, s_expr, io)?;
                    sv.as_matrix()
                        .and_then(|m| m.as_scalar())
                        .ok_or_else(|| Diagnostic::error("Range step must be a scalar.", span))?
                }
                None => 1.0,
            };
            if step_val == 0.0 {
                return Err(Diagnostic::error("Step cannot be zero.", span));
            }
            match (s.as_matrix(), e.as_matrix()) {
                (Some(sm), Some(em)) if sm.is_scalar() && em.is_scalar() => {
                    let s_val = sm.data[0];
                    let e_val = em.data[0];
                    let mut data = Vec::new();
                    if step_val > 0.0 {
                        let mut k = s_val;
                        while k <= e_val + 1e-12 {
                            data.push(k);
                            k += step_val;
                        }
                    } else {
                        let mut k = s_val;
                        while k >= e_val - 1e-12 {
                            data.push(k);
                            k += step_val;
                        }
                    }
                    Ok(Value::Matrix(DenseMatrix::from_rows(vec![data], span)?))
                }
                _ => Err(Diagnostic::error(
                    "Range requires scalar start/stop values.",
                    span,
                )),
            }
        }
        Expr::End(_) => Err(Diagnostic::error(
            "`end` keyword requires indexing context.",
            span,
        )),
        Expr::ColonAll(_) => Err(Diagnostic::error(
            "`:` keyword requires indexing context.",
            span,
        )),
        Expr::Transpose {
            expr: inner,
            conjugate,
            ..
        } => {
            let v = eval_expr(env, inner, io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    if *conjugate {
                        Ok(Value::ComplexMatrix(la::complex::ctranspose(&c)))
                    } else {
                        Ok(Value::ComplexMatrix(la::complex::transpose(&c)))
                    }
                }
                _ => {
                    let m = coerce_matrix(v, span)?;
                    Ok(Value::Matrix(la::transpose(&m)))
                }
            }
        }
    }
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

    if l.is_complex() || r.is_complex() {
        return match op {
            BinaryOp::Add => la::add_values(l, r, span),
            BinaryOp::Sub => la::sub_values(l, r, span),
            BinaryOp::Mul => la::mul_values(l, r, span),
            BinaryOp::DotMul => la::dotmul_values(l, r, span),
            BinaryOp::DotDiv => la::dotdiv_values(l, r, span),
            BinaryOp::DotLDiv => la::dotdiv_values(r, l, span),
            BinaryOp::Div => la::mrdivide_values(l, r, span),
            BinaryOp::LDiv => la::mldivide_values(l, r, span),
            BinaryOp::Pow => {
                let lc = l.to_complex_matrix().and_then(|m| m.as_scalar());
                let rc = r.to_complex_matrix().and_then(|m| m.as_scalar());
                if let (Some(base), Some(exp)) = (lc, rc) {
                    return Ok(Value::from_complex_or_simplified(
                        ComplexDenseMatrix::scalar(base.powc(exp)),
                    ));
                }
                let lc_mat = l
                    .to_complex_matrix()
                    .ok_or_else(|| Diagnostic::error("Invalid operand for power.", span))?;
                let p = r.as_matrix().and_then(|m| m.as_scalar()).ok_or_else(|| {
                    Diagnostic::error("Exponent must be a scalar for matrix power.", span)
                })?;
                if lc_mat.rows != lc_mat.cols {
                    return Err(Diagnostic::error(
                        "Matrix must be square to calculate power.",
                        span,
                    ));
                }
                if p.fract() != 0.0 || p < 0.0 {
                    return Err(Diagnostic::error(
                        "Matrix power with non-integer or negative exponent is not yet supported for complex matrices.",
                        span,
                    ));
                }
                let p_int = p as usize;
                if p_int == 0 {
                    return Ok(Value::ComplexMatrix(ComplexDenseMatrix::eye(lc_mat.rows)));
                }
                let mut res = lc_mat.clone();
                for _ in 1..p_int {
                    res = la::complex::matmul(&res, &lc_mat, span)?;
                }
                Ok(Value::from_complex_or_simplified(res))
            }
            BinaryOp::DotPow => la::dotpow_values(l, r, span),
            BinaryOp::Eq => Ok(Value::Matrix(la::compare_values(
                la::CompareOp::Eq,
                l,
                r,
                span,
            )?)),
            BinaryOp::Ne => Ok(Value::Matrix(la::compare_values(
                la::CompareOp::Ne,
                l,
                r,
                span,
            )?)),
            BinaryOp::Lt => Ok(Value::Matrix(la::compare_values(
                la::CompareOp::Lt,
                l,
                r,
                span,
            )?)),
            BinaryOp::Le => Ok(Value::Matrix(la::compare_values(
                la::CompareOp::Le,
                l,
                r,
                span,
            )?)),
            BinaryOp::Gt => Ok(Value::Matrix(la::compare_values(
                la::CompareOp::Gt,
                l,
                r,
                span,
            )?)),
            BinaryOp::Ge => Ok(Value::Matrix(la::compare_values(
                la::CompareOp::Ge,
                l,
                r,
                span,
            )?)),
        };
    }

    // Both operands are real matrices or booleans.
    // Check if DotPow or scalar Pow promotes negative bases with fractional powers to complex!
    if matches!(op, BinaryOp::DotPow) {
        return la::dotpow_values(l, r, span);
    }

    let lm = coerce_matrix(l.clone(), span)?;
    let rm = coerce_matrix(r.clone(), span)?;

    if matches!(op, BinaryOp::Pow) {
        if lm.is_scalar() && rm.is_scalar() {
            let base = lm.data[0];
            let exp = rm.data[0];
            if base < 0.0 && exp.fract() != 0.0 {
                let base_c = Complex64::real(base);
                let exp_c = Complex64::real(exp);
                return Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(
                    base_c.powc(exp_c),
                )));
            }
        }
        let p = rm.as_scalar().ok_or_else(|| {
            Diagnostic::error("Exponent must be a scalar for matrix power.", span)
        })?;
        let m = la::matrix_pow(&lm, p, span)?;
        return Ok(Value::Matrix(m));
    }

    let m = match op {
        BinaryOp::Add => la::add(&lm, &rm, span)?,
        BinaryOp::Sub => la::sub(&lm, &rm, span)?,
        BinaryOp::Mul => la::matmul(&lm, &rm, span)?,
        BinaryOp::Div => la::mrdivide(&lm, &rm, span)?,
        BinaryOp::LDiv => la::mldivide(&lm, &rm, span)?,
        BinaryOp::DotMul => la::elementwise_mul(&lm, &rm, span)?,
        BinaryOp::DotDiv => la::elementwise_div(&lm, &rm, span)?,
        BinaryOp::DotLDiv => la::elementwise_ldiv(&lm, &rm, span)?,
        BinaryOp::Pow | BinaryOp::DotPow => unreachable!(),
        BinaryOp::Eq => la::compare(la::CompareOp::Eq, &lm, &rm, span)?,
        BinaryOp::Ne => la::compare(la::CompareOp::Ne, &lm, &rm, span)?,
        BinaryOp::Lt => la::compare(la::CompareOp::Lt, &lm, &rm, span)?,
        BinaryOp::Le => la::compare(la::CompareOp::Le, &lm, &rm, span)?,
        BinaryOp::Gt => la::compare(la::CompareOp::Gt, &lm, &rm, span)?,
        BinaryOp::Ge => la::compare(la::CompareOp::Ge, &lm, &rm, span)?,
    };
    Ok(Value::Matrix(m))
}
