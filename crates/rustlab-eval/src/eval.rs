//! Core AST tree-walking interpreter.

use crate::error::EvalError;
use crate::scope::Environment;
use rustlab_math::{colon_range, DenseArray, FunctionHandle, MathError, StructArray, Value};
use rustlab_parse::ast::*;
use rustlab_parse::Span;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Trait for MATLAB built-in functions.
pub trait BuiltinFn: Send + Sync {
    /// Executes the built-in function with the given arguments and expected number of output values (`nargout`).
    fn call(&self, args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError>;
}

impl<F> BuiltinFn for F
where
    F: Fn(&[Value], usize) -> Result<Vec<Value>, EvalError> + Send + Sync,
{
    fn call(&self, args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
        self(args, nargout)
    }
}

/// Control flow signals produced during statement evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    /// Proceed to the next statement.
    Next,
    /// Loop break.
    Break,
    /// Loop continue.
    Continue,
    /// Function return with zero or more return values.
    Return(Vec<Value>),
}

/// Context passed to resolve the dynamic `end` keyword inside indexing expressions.
#[derive(Debug, Clone, Copy)]
pub struct IndexingContext {
    pub current_dim_len: usize,
    pub total_len: usize,
}

/// Tree-walking MATLAB evaluator.
pub struct Interpreter {
    pub env: Environment,
    pub functions: HashMap<String, FunctionDef>,
    pub builtins: HashMap<String, Arc<dyn BuiltinFn>>,
    pub last_ans: Option<Value>,
    pub output_log: Vec<String>,
    pub indexing_stack: Vec<IndexingContext>,
    pub search_paths: Vec<PathBuf>,
    pub current_nargin: usize,
    pub current_nargout: usize,
}

impl Interpreter {
    /// Creates a new interpreter with a root environment and standard constants.
    pub fn new() -> Self {
        let mut interp = Self {
            env: Environment::new(),
            functions: HashMap::new(),
            builtins: HashMap::new(),
            last_ans: None,
            output_log: Vec::new(),
            indexing_stack: Vec::new(),
            search_paths: vec![std::env::current_dir().unwrap_or_default()],
            current_nargin: 0,
            current_nargout: 0,
        };
        interp.init_constants();
        interp
    }

    /// Registers a standard built-in function into the interpreter.
    pub fn register_builtin<F>(&mut self, name: &str, func: F)
    where
        F: BuiltinFn + 'static,
    {
        self.builtins.insert(name.to_string(), Arc::new(func));
    }

    /// Returns a sorted list of all registered builtin function names.
    pub fn builtin_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.builtins.keys().cloned().collect();
        let eval_builtins = [
            "cellfun",
            "arrayfun",
            "structfun",
            "which",
            "exist",
            "addpath",
            "rmpath",
        ];
        for eb in eval_builtins {
            if !names.contains(&eb.to_string()) {
                names.push(eb.to_string());
            }
        }
        names.sort();
        names
    }

    fn init_constants(&mut self) {
        self.env.set("pi", Value::scalar(std::f64::consts::PI));
        self.env.set("Inf", Value::scalar(f64::INFINITY));
        self.env.set("inf", Value::scalar(f64::INFINITY));
        self.env.set("NaN", Value::scalar(f64::NAN));
        self.env.set("nan", Value::scalar(f64::NAN));
        self.env.set("true", Value::logical_scalar(true));
        self.env.set("false", Value::logical_scalar(false));
        self.env.set(
            "i",
            Value::ComplexDouble(DenseArray::scalar(rustlab_math::Complex64::new(0.0, 1.0))),
        );
        self.env.set(
            "j",
            Value::ComplexDouble(DenseArray::scalar(rustlab_math::Complex64::new(0.0, 1.0))),
        );
    }

    /// Parses and evaluates an entire script source text.
    pub fn eval_script(&mut self, source: &str) -> Result<Option<Value>, EvalError> {
        let stmts = rustlab_parse::parse(source)?;
        let mut last_val = None;

        for stmt in &stmts {
            match self.eval_stmt(stmt)? {
                Flow::Next => {
                    if let Stmt::Expr { print_result, .. } = stmt {
                        if *print_result {
                            last_val = self.last_ans.clone();
                        }
                    } else if let Stmt::Assignment {
                        target,
                        print_result,
                        ..
                    } = stmt
                    {
                        if *print_result {
                            if let AssignTarget::Ident(name, _) = target {
                                last_val = self.env.get(name);
                            }
                        }
                    }
                }
                Flow::Return(mut vals) => return Ok(vals.pop()),
                Flow::Break | Flow::Continue => break,
            }
        }

        if last_val.is_some() {
            return Ok(last_val);
        }
        if let Some(Stmt::Assignment {
            target: AssignTarget::Ident(name, _),
            ..
        }) = stmts.last()
        {
            return Ok(self.env.get(name));
        }
        Ok(self.last_ans.clone())
    }

    /// Evaluates a block of statements.
    pub fn eval_stmts(&mut self, stmts: &[Stmt]) -> Result<Flow, EvalError> {
        for stmt in stmts {
            let flow = self.eval_stmt(stmt)?;
            if flow != Flow::Next {
                return Ok(flow);
            }
        }
        Ok(Flow::Next)
    }

    /// Evaluates a single statement.
    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Flow, EvalError> {
        match stmt {
            Stmt::Expr {
                expr,
                print_result,
                span,
            } => {
                let val = self.eval_expr(expr)?;
                self.last_ans = Some(val.clone());
                self.env.set("ans", val.clone());
                if *print_result {
                    let out = format!("ans =\n\n{}\n", val);
                    self.output_log.push(out);
                }
                let _ = span;
                Ok(Flow::Next)
            }
            Stmt::Assignment {
                target,
                value,
                print_result,
                span,
            } => self.eval_assignment(target, value, *print_result, *span),
            Stmt::If {
                branches,
                else_branch,
                ..
            } => {
                for (cond_expr, body) in branches {
                    let cond_val = self.eval_expr(cond_expr)?;
                    if self.is_truthy(&cond_val)? {
                        return self.eval_stmts(body);
                    }
                }
                if let Some(else_body) = else_branch {
                    return self.eval_stmts(else_body);
                }
                Ok(Flow::Next)
            }
            Stmt::For {
                var,
                expr,
                body,
                span,
            } => {
                let iter_val = self.eval_expr(expr)?;
                let double_arr = iter_val.to_double_array().map_err(|e| EvalError::Math {
                    error: e,
                    span: *span,
                })?;

                let shape = double_arr.shape();
                let rows = shape[0];
                let cols = shape[1];

                // In MATLAB: 'for col = matrix' iterates column by column!
                for c in 1..=cols {
                    let mut col_data = Vec::with_capacity(rows);
                    for r in 1..=rows {
                        let elem =
                            *double_arr
                                .get_subscript(&[r, c])
                                .map_err(|e| EvalError::Math {
                                    error: e,
                                    span: *span,
                                })?;
                        col_data.push(elem);
                    }

                    // Column vector shape [rows, 1]
                    let col_val = if rows == 1 {
                        Value::scalar(col_data[0])
                    } else {
                        Value::Double(DenseArray::col_vector(col_data))
                    };

                    self.env.set(var, col_val);

                    match self.eval_stmts(body)? {
                        Flow::Next | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(r) => return Ok(Flow::Return(r)),
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::While { cond, body, .. } => {
                loop {
                    let cond_val = self.eval_expr(cond)?;
                    if !self.is_truthy(&cond_val)? {
                        break;
                    }
                    match self.eval_stmts(body)? {
                        Flow::Next | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(r) => return Ok(Flow::Return(r)),
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::Switch {
                expr,
                cases,
                otherwise,
                ..
            } => {
                let switch_val = self.eval_expr(expr)?;
                let mut matched = false;

                for (case_vals, body) in cases {
                    for cv_expr in case_vals {
                        let cv = self.eval_expr(cv_expr)?;
                        let is_eq = self.are_equal(&switch_val, &cv)?;
                        if is_eq {
                            matched = true;
                            let flow = self.eval_stmts(body)?;
                            if flow != Flow::Next {
                                return Ok(flow);
                            }
                            break;
                        }
                    }
                    if matched {
                        break;
                    }
                }

                if !matched {
                    if let Some(other_body) = otherwise {
                        return self.eval_stmts(other_body);
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::TryCatch {
                try_body,
                catch_var,
                catch_body,
                ..
            } => match self.eval_stmts(try_body) {
                Ok(flow) => Ok(flow),
                Err(err) => {
                    if let Some(var) = catch_var {
                        let (id, msg) = match &err {
                            EvalError::UserError {
                                identifier,
                                message,
                                ..
                            } => (identifier.clone(), message.clone()),
                            EvalError::UndefinedSymbol { name, .. } => (
                                "MATLAB:UndefinedFunction".to_string(),
                                format!("Undefined function or variable '{name}'."),
                            ),
                            _ => ("MATLAB:genericError".to_string(), err.to_string()),
                        };
                        let mut map = HashMap::new();
                        map.insert("identifier".to_string(), Value::char_vector(&id));
                        map.insert("message".to_string(), Value::char_vector(&msg));
                        let struct_val = StructArray::scalar(map);
                        self.env.set(var, Value::Struct(struct_val));
                    }
                    self.eval_stmts(catch_body)
                }
            },
            Stmt::Break(_) => Ok(Flow::Break),
            Stmt::Continue(_) => Ok(Flow::Continue),
            Stmt::Return(_) => Ok(Flow::Return(Vec::new())),
            Stmt::Global(vars, _) => {
                for v in vars {
                    self.env.declare_global(v);
                }
                Ok(Flow::Next)
            }
            Stmt::Persistent(vars, _) => {
                for v in vars {
                    self.env.declare_persistent(v);
                }
                Ok(Flow::Next)
            }
            Stmt::Function(def) => {
                self.functions.insert(def.name.clone(), def.clone());
                Ok(Flow::Next)
            }
        }
    }

    // --- Expression Evaluation ---

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Number(n, _) => Ok(Value::scalar(*n)),
            Expr::Imaginary(i, _) => {
                let c = rustlab_math::Complex64::new(0.0, *i);
                Ok(Value::ComplexDouble(DenseArray::scalar(c)))
            }
            Expr::CharVector(s, _) => Ok(Value::char_vector(s)),
            Expr::StringLiteral(s, _) => Ok(Value::string_scalar(s)),
            Expr::Ident(name, span) => {
                if let Some(val) = self.env.get(name) {
                    Ok(val)
                } else if self.functions.contains_key(name) {
                    // Function called with zero arguments
                    self.call_user_function(name, Vec::new(), 1, *span)
                } else if let Some(builtin) = self.builtins.get(name).cloned() {
                    let res = builtin.call(&[], 1)?;
                    Ok(res.into_iter().next().unwrap_or_else(Value::empty))
                } else if self.resolve_and_load_mfile(name)? {
                    if self.functions.contains_key(name) {
                        self.call_user_function(name, Vec::new(), 1, *span)
                    } else {
                        Ok(self.last_ans.clone().unwrap_or_else(Value::empty))
                    }
                } else {
                    Err(EvalError::UndefinedSymbol {
                        name: name.clone(),
                        span: *span,
                    })
                }
            }
            Expr::End(span) => {
                if let Some(ctx) = self.indexing_stack.last() {
                    Ok(Value::scalar(ctx.current_dim_len as f64))
                } else {
                    Err(EvalError::UserError {
                        identifier: "MATLAB:badend".to_string(),
                        message: "The end operator is valid only in indexing expressions."
                            .to_string(),
                        span: *span,
                    })
                }
            }
            Expr::Colon(span) => {
                if let Some(ctx) = self.indexing_stack.last() {
                    // Full colon slice 1:N
                    let r = colon_range(1.0, 1.0, ctx.current_dim_len as f64);
                    Ok(Value::Double(r))
                } else {
                    Err(EvalError::UserError {
                        identifier: "MATLAB:badcolon".to_string(),
                        message: "Colon operator is only valid in subscript expressions."
                            .to_string(),
                        span: *span,
                    })
                }
            }
            Expr::Range {
                start,
                step,
                end,
                span,
            } => {
                let start_v =
                    self.eval_expr(start)?
                        .as_scalar_f64()
                        .map_err(|e| EvalError::Math {
                            error: e,
                            span: *span,
                        })?;
                let end_v = self
                    .eval_expr(end)?
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math {
                        error: e,
                        span: *span,
                    })?;
                let step_v = if let Some(s) = step {
                    self.eval_expr(s)?
                        .as_scalar_f64()
                        .map_err(|e| EvalError::Math {
                            error: e,
                            span: *span,
                        })?
                } else {
                    1.0
                };

                let r = colon_range(start_v, step_v, end_v);
                Ok(Value::Double(r))
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                if *op == BinaryOp::AndAnd {
                    let lv = self.eval_expr(left)?;
                    let truthy = self.is_truthy(&lv)?;
                    if !truthy {
                        return Ok(Value::logical_scalar(false));
                    }
                    let rv = self.eval_expr(right)?;
                    let r_truthy = self.is_truthy(&rv)?;
                    return Ok(Value::logical_scalar(r_truthy));
                }
                if *op == BinaryOp::OrOr {
                    let lv = self.eval_expr(left)?;
                    let truthy = self.is_truthy(&lv)?;
                    if truthy {
                        return Ok(Value::logical_scalar(true));
                    }
                    let rv = self.eval_expr(right)?;
                    let r_truthy = self.is_truthy(&rv)?;
                    return Ok(Value::logical_scalar(r_truthy));
                }
                // Fast-path: Both operands are local identifiers (e.g. A + B).
                // Evaluates binary operation with zero cloning from environment!
                if let (Expr::Ident(l_name, _), Expr::Ident(r_name, _)) = (&**left, &**right) {
                    if let (Some(lv), Some(rv)) =
                        (self.env.get_local(l_name), self.env.get_local(r_name))
                    {
                        return self.eval_binary_op(*op, lv, rv, *span);
                    }
                }

                let lv = self.eval_expr(left)?;
                let rv = self.eval_expr(right)?;
                self.eval_binary_op(*op, &lv, &rv, *span)
            }
            Expr::Unary { op, expr, span } => {
                let v = self.eval_expr(expr)?;
                self.eval_unary_op(*op, &v, *span)
            }
            Expr::Matrix { rows, span } => self.eval_matrix_literal(rows, *span),
            Expr::CellArray { rows, span } => self.eval_cell_literal(rows, *span),
            Expr::Index { callee, args, span } => self.eval_index_or_call(callee, args, *span),
            Expr::CellIndex { callee, args, span } => self.eval_cell_deref(callee, args, *span),
            Expr::DotAccess { expr, field, span } => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Struct(s) => {
                        s.get_field(field)
                            .cloned()
                            .ok_or_else(|| EvalError::UserError {
                                identifier: "MATLAB:nonExistentField".to_string(),
                                message: format!("Reference to non-existent field '{}'.", field),
                                span: *span,
                            })
                    }
                    _ => Err(EvalError::UserError {
                        identifier: "MATLAB:dotOnNonStruct".to_string(),
                        message: "Dot indexing is only supported on structures.".to_string(),
                        span: *span,
                    }),
                }
            }
            Expr::DynamicDotAccess {
                expr,
                field_expr,
                span,
            } => {
                let val = self.eval_expr(expr)?;
                let field_val = self.eval_expr(field_expr)?;
                let field_name = match field_val {
                    Value::Char(c) => c.as_slice().iter().collect::<String>(),
                    Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
                    _ => {
                        return Err(EvalError::UserError {
                            identifier: "MATLAB:badFieldType".to_string(),
                            message: "Field name must be a string or character vector.".to_string(),
                            span: *span,
                        })
                    }
                };
                match val {
                    Value::Struct(s) => {
                        s.get_field(&field_name)
                            .cloned()
                            .ok_or_else(|| EvalError::UserError {
                                identifier: "MATLAB:nonExistentField".to_string(),
                                message: format!(
                                    "Reference to non-existent field '{}'.",
                                    field_name
                                ),
                                span: *span,
                            })
                    }
                    _ => Err(EvalError::UserError {
                        identifier: "MATLAB:dotOnNonStruct".to_string(),
                        message: "Dot indexing is only supported on structures.".to_string(),
                        span: *span,
                    }),
                }
            }
            Expr::FunctionHandle(name, _) => Ok(Value::function_handle_named(name)),
            Expr::AnonymousFunction { params, body, .. } => {
                // Lexical capture of free variables by value at creation time!
                let mut bound = HashSet::new();
                for p in params {
                    bound.insert(p.clone());
                }
                let mut free_vars = HashSet::new();
                collect_free_variables(body, &bound, &mut free_vars);

                let mut closure_env = HashMap::new();
                for var in free_vars {
                    if let Some(val) = self.env.get(&var) {
                        closure_env.insert(var, val);
                    }
                }

                Ok(Value::function_handle_anon(
                    params.clone(),
                    *body.clone(),
                    closure_env,
                ))
            }
        }
    }

    // --- Helper Evaluators ---

    fn eval_assignment(
        &mut self,
        target: &AssignTarget,
        value_expr: &Expr,
        print_result: bool,
        span: Span,
    ) -> Result<Flow, EvalError> {
        let val = self.eval_expr(value_expr)?;

        match target {
            AssignTarget::Ident(name, _) => {
                if print_result {
                    let out = format!("{} =\n\n{}\n", name, val);
                    self.output_log.push(out);
                }
                self.env.set(name, val);
            }
            AssignTarget::Index { callee, args, .. } => {
                if let Expr::Ident(var_name, _) = &**callee {
                    let mut current_val = self.env.get(var_name).unwrap_or_else(Value::empty);

                    // Boolean mask assignment: A(mask) = val
                    if args.len() == 1 {
                        let total_len = current_val.numel();
                        self.indexing_stack.push(IndexingContext {
                            current_dim_len: total_len,
                            total_len,
                        });
                        let idx_res = self.eval_expr(&args[0]);
                        self.indexing_stack.pop();
                        let idx_val = idx_res?;

                        if let Value::Logical(ref mask) = idx_val {
                            if val.is_empty() {
                                // Deletion: A(mask) = []
                                let inv_data: Vec<bool> =
                                    mask.as_slice().iter().map(|&b| !b).collect();
                                let inv_mask =
                                    DenseArray::from_vec_unchecked(mask.shape().to_vec(), inv_data);
                                let res_val = current_val
                                    .get_mask(&inv_mask)
                                    .map_err(|e| EvalError::Math { error: e, span })?;
                                self.env.set(var_name, res_val.clone());
                                self.last_ans = Some(res_val.clone());
                                if print_result {
                                    let out = format!("{} =\n\n{}\n", var_name, res_val);
                                    self.output_log.push(out);
                                }
                                return Ok(Flow::Next);
                            }
                            current_val
                                .set_mask(mask, &val)
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            self.env.set(var_name, current_val.clone());
                            self.last_ans = Some(current_val.clone());
                            if print_result {
                                let out = format!("{} =\n\n{}\n", var_name, current_val);
                                self.output_log.push(out);
                            }
                            return Ok(Flow::Next);
                        }

                        if val.is_empty() {
                            let da = idx_val
                                .to_double_array()
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            let mut del_set = std::collections::HashSet::new();
                            for &d in da.as_slice() {
                                let i = d as usize;
                                if i >= 1 {
                                    del_set.insert(i);
                                }
                            }
                            let total = current_val.numel();
                            let mut keep_mask = Vec::with_capacity(total);
                            for i in 1..=total {
                                keep_mask.push(!del_set.contains(&i));
                            }
                            let mask_arr =
                                DenseArray::from_vec_unchecked(vec![total, 1], keep_mask);
                            let res_val = current_val
                                .get_mask(&mask_arr)
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            self.env.set(var_name, res_val.clone());
                            self.last_ans = Some(res_val.clone());
                            if print_result {
                                let out = format!("{} =\n\n{}\n", var_name, res_val);
                                self.output_log.push(out);
                            }
                            return Ok(Flow::Next);
                        }

                        // Linear numeric indexing with growth
                        let mut current_arr = current_val
                            .to_double_array()
                            .unwrap_or_else(|_| DenseArray::empty());
                        let new_val = val
                            .as_scalar_f64()
                            .map_err(|e| EvalError::Math { error: e, span })?;
                        let idx = idx_val
                            .as_scalar_f64()
                            .map_err(|e| EvalError::Math { error: e, span })?
                            as usize;
                        current_arr
                            .set_linear_grow(idx, new_val)
                            .map_err(|e| EvalError::Math { error: e, span })?;

                        let res_val = Value::Double(current_arr);
                        self.env.set(var_name, res_val.clone());
                        self.last_ans = Some(res_val.clone());
                        if print_result {
                            let out = format!("{} =\n\n{}\n", var_name, res_val);
                            self.output_log.push(out);
                        }
                    } else if let Value::Sparse(mut sp) = current_val {
                        if args.len() == 2 && val.is_scalar() {
                            let r_res = self.eval_expr(&args[0])?;
                            let c_res = self.eval_expr(&args[1])?;
                            let r = r_res
                                .as_scalar_f64()
                                .map_err(|e| EvalError::Math { error: e, span })?
                                as usize;
                            let c = c_res
                                .as_scalar_f64()
                                .map_err(|e| EvalError::Math { error: e, span })?
                                as usize;
                            let v = val
                                .as_scalar_f64()
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            sp.set_1based(r, c, v)
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            let res_val = Value::Sparse(sp);
                            self.env.set(var_name, res_val.clone());
                            self.last_ans = Some(res_val.clone());
                            if print_result {
                                let out = format!("{} =\n\n{}\n", var_name, res_val);
                                self.output_log.push(out);
                            }
                            return Ok(Flow::Next);
                        } else {
                            return Err(EvalError::UserError {
                                identifier: "MATLAB:subsasgn".to_string(),
                                message: "Sparse subscript assignment requires scalar value and 2D subscripts.".to_string(),
                                span,
                            });
                        }
                    } else {
                        // Subscript indexing with growth or sliced assignment
                        if current_val.is_integer()
                            && val.is_integer()
                            && current_val.type_name() != val.type_name()
                        {
                            return Err(EvalError::Math {
                                error: MathError::integer_class_mismatch(),
                                span,
                            });
                        }

                        let mut current_arr = current_val
                            .to_double_array()
                            .unwrap_or_else(|_| DenseArray::empty());
                        let mut sub_indices: Vec<Vec<usize>> = Vec::with_capacity(args.len());
                        let shp = current_arr.shape().to_vec();

                        for (d, a) in args.iter().enumerate() {
                            let dim_len = if d == args.len() - 1 && shp.len() > args.len() {
                                shp[d..].iter().product()
                            } else {
                                shp.get(d).copied().unwrap_or(1)
                            };
                            self.indexing_stack.push(IndexingContext {
                                current_dim_len: dim_len,
                                total_len: current_arr.numel(),
                            });
                            let s_res = self.eval_expr(a);
                            self.indexing_stack.pop();
                            let s_val = s_res?;
                            let s_da = s_val
                                .to_double_array()
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            let idxs: Vec<usize> =
                                s_da.as_slice().iter().map(|&x| x as usize).collect();
                            sub_indices.push(idxs);
                        }

                        if val.is_empty() {
                            let mut exp_shp = shp.clone();
                            while exp_shp.len() < sub_indices.len() {
                                exp_shp.push(1);
                            }
                            let non_colon_dims: Vec<usize> = (0..sub_indices.len())
                                .filter(|&d| sub_indices[d].len() != exp_shp[d])
                                .collect();
                            if non_colon_dims.len() == 1 {
                                let del_dim = non_colon_dims[0];
                                let del_set: std::collections::HashSet<usize> =
                                    sub_indices[del_dim].iter().copied().collect();
                                let mut new_shp = exp_shp.clone();
                                let remaining = (1..=exp_shp[del_dim])
                                    .filter(|idx| !del_set.contains(idx))
                                    .count();
                                new_shp[del_dim] = remaining;
                                new_shp = rustlab_math::normalize_shape(&new_shp);

                                let total = current_arr.numel();
                                let mut new_data = Vec::new();
                                for offset in 0..total {
                                    let coords = rustlab_math::offset_to_coords(offset, &exp_shp);
                                    if !del_set.contains(&(coords[del_dim] + 1)) {
                                        new_data.push(current_arr.as_slice()[offset]);
                                    }
                                }
                                let res_arr = DenseArray::from_vec(new_shp, new_data)
                                    .map_err(|e| EvalError::Math { error: e, span })?;
                                let res_val = Value::Double(res_arr);
                                self.env.set(var_name, res_val.clone());
                                self.last_ans = Some(res_val.clone());
                                if print_result {
                                    let out = format!("{} =\n\n{}\n", var_name, res_val);
                                    self.output_log.push(out);
                                }
                                return Ok(Flow::Next);
                            }
                            return Err(EvalError::UserError {
                                identifier: "MATLAB:nullAssignment".to_string(),
                                message: "A null assignment can have only one non-colon index."
                                    .to_string(),
                                span,
                            });
                        }

                        let slice_shape: Vec<usize> = sub_indices.iter().map(|s| s.len()).collect();
                        let slice_numel: usize = slice_shape.iter().product();

                        if val.is_scalar() {
                            let scalar_val = val
                                .as_scalar_f64()
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            for offset in 0..slice_numel {
                                let coords = rustlab_math::offset_to_coords(offset, &slice_shape);
                                let mut target_coords = Vec::with_capacity(sub_indices.len());
                                for (d, &c) in coords.iter().enumerate() {
                                    target_coords.push(sub_indices[d][c]);
                                }
                                current_arr
                                    .set_subscript_grow(&target_coords, scalar_val)
                                    .map_err(|e| EvalError::Math { error: e, span })?;
                            }
                        } else {
                            if val.numel() != slice_numel {
                                return Err(EvalError::UserError {
                                    identifier: "MATLAB:subsasgn:dimensionMismatch".to_string(),
                                    message: "Subscripted assignment dimension mismatch."
                                        .to_string(),
                                    span,
                                });
                            }
                            let val_da = val
                                .to_double_array()
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            for offset in 0..slice_numel {
                                let coords = rustlab_math::offset_to_coords(offset, &slice_shape);
                                let mut target_coords = Vec::with_capacity(sub_indices.len());
                                for (d, &c) in coords.iter().enumerate() {
                                    target_coords.push(sub_indices[d][c]);
                                }
                                let elem = val_da.as_slice()[offset];
                                current_arr
                                    .set_subscript_grow(&target_coords, elem)
                                    .map_err(|e| EvalError::Math { error: e, span })?;
                            }
                        }

                        let res_val = if current_val.is_integer() {
                            Value::Double(current_arr)
                                .cast_to(current_val.type_name())
                                .map_err(|e| EvalError::Math { error: e, span })?
                        } else {
                            Value::Double(current_arr)
                        };

                        self.env.set(var_name, res_val.clone());
                        self.last_ans = Some(res_val.clone());
                        if print_result {
                            let out = format!("{} =\n\n{}\n", var_name, res_val);
                            self.output_log.push(out);
                        }
                    }
                }
            }
            AssignTarget::MultiAssign(targets, _) => {
                // Multi-assignment: [a, b] = func()
                if let Expr::Index { callee, args, .. } = value_expr {
                    if let Expr::Ident(func_name, f_span) = &**callee {
                        let mut evaluated_args = Vec::new();
                        for a in args {
                            evaluated_args.push(self.eval_expr(a)?);
                        }

                        let outputs = if self.functions.contains_key(func_name) {
                            self.call_user_function_multi(
                                func_name,
                                evaluated_args,
                                targets.len(),
                                *f_span,
                            )?
                        } else if let Some(builtin) = self.builtins.get(func_name).cloned() {
                            builtin.call(&evaluated_args, targets.len())?
                        } else if self.resolve_and_load_mfile(func_name)? {
                            self.call_user_function_multi(
                                func_name,
                                evaluated_args,
                                targets.len(),
                                *f_span,
                            )?
                        } else {
                            return Err(EvalError::UndefinedSymbol {
                                name: func_name.clone(),
                                span: *f_span,
                            });
                        };

                        for (i, t) in targets.iter().enumerate() {
                            if let AssignTarget::Ident(n, _) = t {
                                if let Some(v) = outputs.get(i) {
                                    self.env.set(n, v.clone());
                                }
                            }
                        }
                    }
                }
            }
            AssignTarget::CellIndex {
                callee,
                args,
                span: cell_span,
            } => {
                if let Expr::Ident(var_name, _) = &**callee {
                    let mut current_cell = match self.env.get(var_name) {
                        Some(Value::Cell(arr)) => arr,
                        _ => DenseArray::empty(),
                    };

                    if args.len() == 1 {
                        let total_len = current_cell.numel();
                        self.indexing_stack.push(IndexingContext {
                            current_dim_len: total_len,
                            total_len,
                        });
                        let idx_res = self.eval_expr(&args[0]);
                        self.indexing_stack.pop();
                        let idx = idx_res?.as_scalar_f64().map_err(|e| EvalError::Math {
                            error: e,
                            span: *cell_span,
                        })? as usize;
                        current_cell
                            .set_linear_grow(idx, val.clone())
                            .map_err(|e| EvalError::Math {
                                error: e,
                                span: *cell_span,
                            })?;
                    } else {
                        let mut subs = Vec::with_capacity(args.len());
                        let shp = current_cell.shape().to_vec();
                        for (d, a) in args.iter().enumerate() {
                            let dim_len = if d == args.len() - 1 && shp.len() > args.len() {
                                shp[d..].iter().product()
                            } else {
                                shp.get(d).copied().unwrap_or(1)
                            };
                            self.indexing_stack.push(IndexingContext {
                                current_dim_len: dim_len,
                                total_len: current_cell.numel(),
                            });
                            let s_res = self.eval_expr(a);
                            self.indexing_stack.pop();
                            let s = s_res?.as_scalar_f64().map_err(|e| EvalError::Math {
                                error: e,
                                span: *cell_span,
                            })? as usize;
                            subs.push(s);
                        }
                        current_cell
                            .set_subscript_grow(&subs, val.clone())
                            .map_err(|e| EvalError::Math {
                                error: e,
                                span: *cell_span,
                            })?;
                    }

                    let res_val = Value::Cell(current_cell);
                    self.env.set(var_name, res_val.clone());
                    self.last_ans = Some(res_val.clone());
                    if print_result {
                        let out = format!("{} =\n\n{}\n", var_name, res_val);
                        self.output_log.push(out);
                    }
                }
            }
            AssignTarget::Dot { expr, field, .. } => {
                if let Expr::Ident(var_name, _) = &**expr {
                    let mut current_struct = match self.env.get(var_name) {
                        Some(Value::Struct(s)) => s,
                        _ => StructArray::scalar(HashMap::new()),
                    };
                    current_struct.set_field(field, val.clone());
                    let res_val = Value::Struct(current_struct);
                    self.env.set(var_name, res_val.clone());
                    self.last_ans = Some(res_val.clone());
                    if print_result {
                        let out = format!("{} =\n\n{}\n", var_name, res_val);
                        self.output_log.push(out);
                    }
                }
            }
        }

        Ok(Flow::Next)
    }

    fn eval_index_or_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: Span,
    ) -> Result<Value, EvalError> {
        // Special higher-order builtins with evaluator access
        if let Expr::Ident(name, _) = callee {
            if name == "cellfun" {
                return self.eval_cellfun(args, span);
            }
            if name == "arrayfun" {
                return self.eval_arrayfun(args, span);
            }
            if name == "structfun" {
                return self.eval_structfun(args, span);
            }
            if name == "nargin" {
                return Ok(Value::scalar(self.current_nargin as f64));
            }
            if name == "nargout" {
                return Ok(Value::scalar(self.current_nargout as f64));
            }
            if name == "which" {
                return self.eval_which(args, span);
            }
            if name == "exist" {
                return self.eval_exist(args, span);
            }
            if name == "addpath" {
                return self.eval_addpath(args, span);
            }
            if name == "rmpath" {
                return self.eval_rmpath(args, span);
            }

            // Check variable in local environment
            if let Some(val) = self.env.get(name) {
                return self.index_or_call_value(val, args, span);
            }

            // Check user-defined function
            if self.functions.contains_key(name) {
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args {
                    evaluated_args.push(self.eval_expr(a)?);
                }
                return self.call_user_function(name, evaluated_args, 1, span);
            }

            // Check registered builtin function
            if let Some(builtin) = self.builtins.get(name).cloned() {
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args {
                    evaluated_args.push(self.eval_expr(a)?);
                }
                let mut results = builtin.call(&evaluated_args, 1)?;
                return Ok(results.pop().unwrap_or_else(Value::empty));
            }

            // Check M-file on search paths
            if self.resolve_and_load_mfile(name)? {
                if self.functions.contains_key(name) {
                    let mut evaluated_args = Vec::with_capacity(args.len());
                    for a in args {
                        evaluated_args.push(self.eval_expr(a)?);
                    }
                    return self.call_user_function(name, evaluated_args, 1, span);
                } else {
                    return Ok(self.last_ans.clone().unwrap_or_else(Value::empty));
                }
            }

            return Err(EvalError::UndefinedSymbol {
                name: name.clone(),
                span,
            });
        }

        // Callee is a general expression (e.g. s.field(end), C{1}(2), (@(x) x+1)(5))
        let callee_val = self.eval_expr(callee)?;
        self.index_or_call_value(callee_val, args, span)
    }

    fn index_or_call_value(
        &mut self,
        val: Value,
        args: &[Expr],
        span: Span,
    ) -> Result<Value, EvalError> {
        // 1. Calling a function handle
        if let Value::FunctionHandle(handle) = val {
            let mut evaluated_args = Vec::with_capacity(args.len());
            for a in args {
                evaluated_args.push(self.eval_expr(a)?);
            }
            let mut results = self.call_function_handle(&handle, evaluated_args, 1, span)?;
            return Ok(results.pop().unwrap_or_else(Value::empty));
        }

        // 1b. Sparse matrix indexing: S(i, j)
        if let Value::Sparse(ref sp) = val {
            if args.len() == 2 {
                let r_res = self.eval_expr(&args[0])?;
                let c_res = self.eval_expr(&args[1])?;
                let r = r_res
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math { error: e, span })?
                    as usize;
                let c = c_res
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math { error: e, span })?
                    as usize;
                let elem = sp
                    .get_1based(r, c)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::scalar(elem));
            } else if args.len() == 1 {
                let idx_res = self.eval_expr(&args[0])?;
                let idx = idx_res
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math { error: e, span })?
                    as usize;
                let dense = sp.to_dense();
                let elem = *dense
                    .get_linear(idx)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::scalar(elem));
            }
        }

        // 2. Cell array indexing (extracts sub-cell): C(idx)
        if let Value::Cell(cell_arr) = val {
            if args.len() == 1 {
                let total_len = cell_arr.numel();
                self.indexing_stack.push(IndexingContext {
                    current_dim_len: total_len,
                    total_len,
                });
                let idx_res = self.eval_expr(&args[0]);
                self.indexing_stack.pop();
                let idx_val = idx_res?;

                if let Value::Logical(ref mask) = idx_val {
                    let sub_cell = cell_arr
                        .get_mask(mask)
                        .map_err(|e| EvalError::Math { error: e, span })?;
                    return Ok(Value::Cell(sub_cell));
                }

                let idx = idx_val
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math { error: e, span })?
                    as usize;
                let elem = cell_arr
                    .get_linear(idx)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Cell(DenseArray::scalar(elem.clone())));
            } else {
                let mut subs = Vec::with_capacity(args.len());
                let shp = cell_arr.shape().to_vec();
                for (d, a) in args.iter().enumerate() {
                    let dim_len = if d == args.len() - 1 && shp.len() > args.len() {
                        shp[d..].iter().product()
                    } else {
                        shp.get(d).copied().unwrap_or(1)
                    };
                    self.indexing_stack.push(IndexingContext {
                        current_dim_len: dim_len,
                        total_len: cell_arr.numel(),
                    });
                    let s_res = self.eval_expr(a);
                    self.indexing_stack.pop();
                    let s = s_res?
                        .as_scalar_f64()
                        .map_err(|e| EvalError::Math { error: e, span })?
                        as usize;
                    subs.push(s);
                }
                let elem = cell_arr
                    .get_subscript(&subs)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Cell(DenseArray::scalar(elem.clone())));
            }
        }

        // 3. String / Char linear indexing
        match val {
            Value::Char(ref ca) if args.len() == 1 => {
                let total_len = ca.numel();
                self.indexing_stack.push(IndexingContext {
                    current_dim_len: total_len,
                    total_len,
                });
                let idx_res = self.eval_expr(&args[0]);
                self.indexing_stack.pop();
                let idx = idx_res?
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math { error: e, span })?
                    as usize;
                let ch = *ca
                    .get_linear(idx)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Char(DenseArray::scalar(ch)));
            }
            Value::String(ref sa) if args.len() == 1 => {
                let total_len = sa.numel();
                self.indexing_stack.push(IndexingContext {
                    current_dim_len: total_len,
                    total_len,
                });
                let idx_res = self.eval_expr(&args[0]);
                self.indexing_stack.pop();
                let idx = idx_res?
                    .as_scalar_f64()
                    .map_err(|e| EvalError::Math { error: e, span })?
                    as usize;
                let st = sa
                    .get_linear(idx)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::string_scalar(st));
            }
            _ => {}
        }

        // 4. General Tensor Indexing (Double, ComplexDouble, Logical, etc.)
        if args.len() == 1 {
            // Linear indexing or boolean masking: A(idx) or A(mask)
            let total_len = val.numel();
            self.indexing_stack.push(IndexingContext {
                current_dim_len: total_len,
                total_len,
            });
            let idx_res = self.eval_expr(&args[0]);
            self.indexing_stack.pop();
            let idx_val = idx_res?;

            // Boolean mask indexing: A(mask) -> column vector
            if let Value::Logical(ref mask) = idx_val {
                let masked = val
                    .get_mask(mask)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(masked);
            }

            let da = val
                .to_double_array()
                .map_err(|e| EvalError::Math { error: e, span })?;
            let idx_arr = idx_val
                .to_double_array()
                .map_err(|e| EvalError::Math { error: e, span })?;

            if idx_arr.is_scalar() {
                let i = idx_arr.as_slice()[0] as usize;
                let elem = *da
                    .get_linear(i)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                Ok(Value::scalar(elem))
            } else {
                // Slicing by vector of indices: A(1:4) or A(:)
                let mut sliced_data = Vec::with_capacity(idx_arr.numel());
                for &i_flt in idx_arr.as_slice() {
                    let elem = *da
                        .get_linear(i_flt as usize)
                        .map_err(|e| EvalError::Math { error: e, span })?;
                    sliced_data.push(elem);
                }
                let out_shape = if matches!(args[0], Expr::Colon(_)) {
                    vec![idx_arr.numel(), 1]
                } else {
                    idx_arr.shape().to_vec()
                };
                let out_arr = DenseArray::from_vec(out_shape, sliced_data)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                Ok(Value::Double(out_arr))
            }
        } else {
            // Multi-subscript indexing: A(r, c, ...)
            let da = val
                .to_double_array()
                .map_err(|e| EvalError::Math { error: e, span })?;
            let mut subs = Vec::with_capacity(args.len());
            let shp = da.shape().to_vec();

            for (d, a) in args.iter().enumerate() {
                let dim_len = if d == args.len() - 1 && shp.len() > args.len() {
                    shp[d..].iter().product()
                } else {
                    shp.get(d).copied().unwrap_or(1)
                };
                self.indexing_stack.push(IndexingContext {
                    current_dim_len: dim_len,
                    total_len: da.numel(),
                });
                let sub_res = self.eval_expr(a);
                self.indexing_stack.pop();
                let sub_val = sub_res?;
                let sub_arr = sub_val
                    .to_double_array()
                    .map_err(|e| EvalError::Math { error: e, span })?;
                subs.push(sub_arr);
            }

            // Scalar subscripts: A(r, c)
            if subs.iter().all(|s| s.is_scalar()) {
                let coords: Vec<usize> = subs.iter().map(|s| s.as_slice()[0] as usize).collect();
                let elem = *da
                    .get_subscript(&coords)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                if val.is_integer() {
                    let res = Value::scalar(elem)
                        .cast_to(val.type_name())
                        .map_err(|e| EvalError::Math { error: e, span })?;
                    Ok(res)
                } else {
                    Ok(Value::scalar(elem))
                }
            } else {
                // General N-D tensor slice indexing: A(r, c, k, ...)
                let dim_indices: Vec<Vec<usize>> = subs
                    .iter()
                    .map(|s| s.as_slice().iter().map(|&x| x as usize).collect())
                    .collect();

                let out_shape: Vec<usize> = dim_indices.iter().map(|s| s.len()).collect();
                let norm_out_shape = rustlab_math::normalize_shape(&out_shape);
                let total_elements: usize = norm_out_shape.iter().product();
                let mut out_data = Vec::with_capacity(total_elements);

                for out_offset in 0..total_elements {
                    let out_coords = rustlab_math::offset_to_coords(out_offset, &norm_out_shape);
                    let mut target_coords = Vec::with_capacity(dim_indices.len());
                    for (d, &c) in out_coords.iter().enumerate() {
                        if d < dim_indices.len() {
                            target_coords.push(dim_indices[d][c]);
                        }
                    }
                    let elem = *da
                        .get_subscript(&target_coords)
                        .map_err(|e| EvalError::Math { error: e, span })?;
                    out_data.push(elem);
                }
                let out_arr = DenseArray::from_vec(norm_out_shape, out_data)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                let res = if val.is_integer() {
                    Value::Double(out_arr)
                        .cast_to(val.type_name())
                        .map_err(|e| EvalError::Math { error: e, span })?
                } else {
                    Value::Double(out_arr)
                };
                Ok(res)
            }
        }
    }

    /// Evaluates `cellfun(@fn, C, ...)`
    pub fn eval_cellfun(&mut self, args: &[Expr], span: Span) -> Result<Value, EvalError> {
        if args.len() < 2 {
            return Err(EvalError::UserError {
                identifier: "MATLAB:cellfun:minrhs".to_string(),
                message: "cellfun requires at least 2 inputs.".to_string(),
                span,
            });
        }
        let fn_val = self.eval_expr(&args[0])?;
        let cell_val = self.eval_expr(&args[1])?;

        let cell_da = match cell_val {
            Value::Cell(da) => da,
            _ => {
                return Err(EvalError::UserError {
                    identifier: "MATLAB:cellfun:notCell".to_string(),
                    message: "Input #2 must be a cell array.".to_string(),
                    span,
                })
            }
        };

        let mut uniform_output = true;
        let mut error_handler: Option<FunctionHandle> = None;

        let mut idx = 2;
        while idx < args.len() {
            let opt_name_val = self.eval_expr(&args[idx])?;
            let opt_name = match opt_name_val {
                Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
                Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
                _ => break,
            };
            if opt_name.eq_ignore_ascii_case("UniformOutput") && idx + 1 < args.len() {
                let u_val = self.eval_expr(&args[idx + 1])?;
                uniform_output = u_val.as_scalar_f64().unwrap_or(1.0) != 0.0;
                idx += 2;
            } else if opt_name.eq_ignore_ascii_case("ErrorHandler") && idx + 1 < args.len() {
                let eh_val = self.eval_expr(&args[idx + 1])?;
                if let Value::FunctionHandle(h) = eh_val {
                    error_handler = Some((*h).clone());
                }
                idx += 2;
            } else {
                idx += 1;
            }
        }

        let fn_handle = match fn_val {
            Value::FunctionHandle(h) => (*h).clone(),
            Value::Char(ca) => FunctionHandle::Named(ca.as_slice().iter().collect()),
            Value::String(sa) => {
                FunctionHandle::Named(sa.as_slice().first().cloned().unwrap_or_default())
            }
            _ => {
                return Err(EvalError::UserError {
                    identifier: "MATLAB:cellfun:badHandle".to_string(),
                    message: "First input must be a function handle or function name.".to_string(),
                    span,
                })
            }
        };

        let mut results = Vec::with_capacity(cell_da.numel());
        for item in cell_da.as_slice() {
            let call_res = self.call_function_handle(&fn_handle, vec![item.clone()], 1, span);
            match call_res {
                Ok(mut outs) => {
                    results.push(outs.pop().unwrap_or_else(Value::empty));
                }
                Err(err) => {
                    if let Some(ref eh) = error_handler {
                        let err_struct = Value::Struct(StructArray::scalar({
                            let mut m = HashMap::new();
                            m.insert(
                                "message".to_string(),
                                Value::string_scalar(&err.to_string()),
                            );
                            m
                        }));
                        let mut eh_res =
                            self.call_function_handle(eh, vec![err_struct, item.clone()], 1, span)?;
                        results.push(eh_res.pop().unwrap_or_else(Value::empty));
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        if uniform_output {
            if results.is_empty() {
                return Ok(Value::empty());
            }
            if results
                .iter()
                .all(|r| matches!(r, Value::Double(d) if d.is_scalar()))
            {
                let floats: Vec<f64> = results
                    .iter()
                    .filter_map(|r| r.as_scalar_f64().ok())
                    .collect();
                let arr = DenseArray::from_vec(cell_da.shape().to_vec(), floats)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Double(arr));
            } else if results
                .iter()
                .all(|r| matches!(r, Value::Logical(l) if l.is_scalar()))
            {
                let bools: Vec<bool> = results
                    .iter()
                    .map(|r| {
                        if let Value::Logical(l) = r {
                            l.as_slice()[0]
                        } else {
                            false
                        }
                    })
                    .collect();
                let arr = DenseArray::from_vec(cell_da.shape().to_vec(), bools)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Logical(arr));
            } else {
                return Err(EvalError::UserError {
                    identifier: "MATLAB:cellfun:NotAScalarOutput".to_string(),
                    message:
                        "Non-scalar or non-matching output in cellfun with UniformOutput=true."
                            .to_string(),
                    span,
                });
            }
        }

        let out_arr = DenseArray::from_vec(cell_da.shape().to_vec(), results)
            .map_err(|e| EvalError::Math { error: e, span })?;
        Ok(Value::Cell(out_arr))
    }

    /// Evaluates `arrayfun(@fn, A, ...)`
    pub fn eval_arrayfun(&mut self, args: &[Expr], span: Span) -> Result<Value, EvalError> {
        if args.len() < 2 {
            return Err(EvalError::UserError {
                identifier: "MATLAB:arrayfun:minrhs".to_string(),
                message: "arrayfun requires at least 2 inputs.".to_string(),
                span,
            });
        }
        let fn_val = self.eval_expr(&args[0])?;
        let arr_val = self.eval_expr(&args[1])?;

        let fn_handle = match fn_val {
            Value::FunctionHandle(h) => (*h).clone(),
            Value::Char(ca) => FunctionHandle::Named(ca.as_slice().iter().collect()),
            Value::String(sa) => {
                FunctionHandle::Named(sa.as_slice().first().cloned().unwrap_or_default())
            }
            _ => {
                return Err(EvalError::UserError {
                    identifier: "MATLAB:arrayfun:badHandle".to_string(),
                    message: "First input must be a function handle or function name.".to_string(),
                    span,
                })
            }
        };

        let mut uniform_output = true;
        let mut idx = 2;
        while idx < args.len() {
            let opt_name_val = self.eval_expr(&args[idx])?;
            let opt_name = match opt_name_val {
                Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
                Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
                _ => break,
            };
            if opt_name.eq_ignore_ascii_case("UniformOutput") && idx + 1 < args.len() {
                let u_val = self.eval_expr(&args[idx + 1])?;
                uniform_output = u_val.as_scalar_f64().unwrap_or(1.0) != 0.0;
                idx += 2;
            } else {
                idx += 1;
            }
        }

        let da = arr_val
            .to_double_array()
            .map_err(|e| EvalError::Math { error: e, span })?;
        let mut results = Vec::with_capacity(da.numel());

        for &elem in da.as_slice() {
            let mut call_res =
                self.call_function_handle(&fn_handle, vec![Value::scalar(elem)], 1, span)?;
            results.push(call_res.pop().unwrap_or_else(Value::empty));
        }

        if uniform_output {
            if results
                .iter()
                .all(|r| matches!(r, Value::Double(d) if d.is_scalar()))
            {
                let floats: Vec<f64> = results
                    .iter()
                    .filter_map(|r| r.as_scalar_f64().ok())
                    .collect();
                let arr = DenseArray::from_vec(da.shape().to_vec(), floats)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Double(arr));
            } else if results
                .iter()
                .all(|r| matches!(r, Value::Logical(l) if l.is_scalar()))
            {
                let bools: Vec<bool> = results
                    .iter()
                    .map(|r| {
                        if let Value::Logical(l) = r {
                            l.as_slice()[0]
                        } else {
                            false
                        }
                    })
                    .collect();
                let arr = DenseArray::from_vec(da.shape().to_vec(), bools)
                    .map_err(|e| EvalError::Math { error: e, span })?;
                return Ok(Value::Logical(arr));
            }
        }

        let out_arr = DenseArray::from_vec(da.shape().to_vec(), results)
            .map_err(|e| EvalError::Math { error: e, span })?;
        Ok(Value::Cell(out_arr))
    }

    /// Evaluates `structfun(@fn, S, ...)`
    pub fn eval_structfun(&mut self, args: &[Expr], span: Span) -> Result<Value, EvalError> {
        if args.len() < 2 {
            return Err(EvalError::UserError {
                identifier: "MATLAB:structfun:minrhs".to_string(),
                message: "structfun requires at least 2 inputs.".to_string(),
                span,
            });
        }
        let fn_val = self.eval_expr(&args[0])?;
        let struct_val = self.eval_expr(&args[1])?;

        let s = match struct_val {
            Value::Struct(s) => s,
            _ => {
                return Err(EvalError::UserError {
                    identifier: "MATLAB:structfun:notStruct".to_string(),
                    message: "Input #2 must be a structure.".to_string(),
                    span,
                })
            }
        };

        let fn_handle = match fn_val {
            Value::FunctionHandle(h) => (*h).clone(),
            Value::Char(ca) => FunctionHandle::Named(ca.as_slice().iter().collect()),
            Value::String(sa) => {
                FunctionHandle::Named(sa.as_slice().first().cloned().unwrap_or_default())
            }
            _ => {
                return Err(EvalError::UserError {
                    identifier: "MATLAB:structfun:badHandle".to_string(),
                    message: "First input must be a function handle or function name.".to_string(),
                    span,
                })
            }
        };

        let mut res_map = HashMap::new();
        let default_val = Value::empty();
        for field in &s.fields {
            let val = s.get_field(field).unwrap_or(&default_val);
            let mut outs = self.call_function_handle(&fn_handle, vec![val.clone()], 1, span)?;
            res_map.insert(field.clone(), outs.pop().unwrap_or_else(Value::empty));
        }

        Ok(Value::Struct(StructArray::scalar(res_map)))
    }

    /// Calls a first-class function handle (named or anonymous with closure).
    pub fn call_function_handle(
        &mut self,
        handle: &FunctionHandle,
        args: Vec<Value>,
        nargout: usize,
        span: Span,
    ) -> Result<Vec<Value>, EvalError> {
        match handle {
            FunctionHandle::Named(name) => {
                if self.functions.contains_key(name) {
                    self.call_user_function_multi(name, args, nargout, span)
                } else if let Some(builtin) = self.builtins.get(name).cloned() {
                    builtin.call(&args, nargout)
                } else if self.resolve_and_load_mfile(name)? {
                    self.call_user_function_multi(name, args, nargout, span)
                } else {
                    Err(EvalError::UndefinedSymbol {
                        name: name.clone(),
                        span,
                    })
                }
            }
            FunctionHandle::Anonymous {
                params,
                body,
                closure_env,
            } => {
                let mut child_env = self.env.child_for_function("@anonymous");
                // Inject captured closure variables
                for (k, v) in closure_env {
                    child_env.set(k, v.clone());
                }
                // Bind parameters
                for (i, param) in params.iter().enumerate() {
                    let arg_val = args.get(i).cloned().unwrap_or_else(Value::empty);
                    child_env.set(param, arg_val);
                }
                child_env.set("nargin", Value::scalar(args.len() as f64));
                child_env.set("nargout", Value::scalar(nargout as f64));

                let prev_nargin = self.current_nargin;
                let prev_nargout = self.current_nargout;
                self.current_nargin = args.len();
                self.current_nargout = nargout;

                std::mem::swap(&mut self.env, &mut child_env);
                let res = self.eval_expr(body);
                std::mem::swap(&mut self.env, &mut child_env);

                self.current_nargin = prev_nargin;
                self.current_nargout = prev_nargout;

                let val = res?;
                Ok(vec![val])
            }
        }
    }

    fn call_user_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
        nargout: usize,
        span: Span,
    ) -> Result<Value, EvalError> {
        let outputs = self.call_user_function_multi(name, args, nargout, span)?;
        Ok(outputs.into_iter().next().unwrap_or_else(Value::empty))
    }

    fn call_user_function_multi(
        &mut self,
        name: &str,
        args: Vec<Value>,
        nargout: usize,
        span: Span,
    ) -> Result<Vec<Value>, EvalError> {
        let def = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::UndefinedSymbol {
                name: name.to_string(),
                span,
            })?;

        let has_varargin = def.inputs.last().map(|s| s.as_str()) == Some("varargin");
        let has_varargout = def.outputs.last().map(|s| s.as_str()) == Some("varargout");

        let fixed_inputs_len = if has_varargin {
            def.inputs.len().saturating_sub(1)
        } else {
            def.inputs.len()
        };

        if !has_varargin && args.len() > def.inputs.len() {
            return Err(EvalError::TooManyInputs {
                expected: def.inputs.len(),
                found: args.len(),
                span,
            });
        }

        // Create isolated child environment
        let mut func_env = self.env.child_for_function(name);

        // Bind fixed input arguments
        for i in 0..fixed_inputs_len {
            let val = args.get(i).cloned().unwrap_or_else(Value::empty);
            func_env.set(&def.inputs[i], val);
        }

        // Bind varargin if present
        if has_varargin {
            let surplus = if args.len() > fixed_inputs_len {
                args[fixed_inputs_len..].to_vec()
            } else {
                Vec::new()
            };
            let cell_arr = DenseArray::from_vec(vec![1, surplus.len()], surplus)
                .map_err(|e| EvalError::Math { error: e, span })?;
            func_env.set("varargin", Value::Cell(cell_arr));
        }

        // Set nargin and nargout
        func_env.set("nargin", Value::scalar(args.len() as f64));
        func_env.set("nargout", Value::scalar(nargout as f64));

        let prev_nargin = self.current_nargin;
        let prev_nargout = self.current_nargout;
        self.current_nargin = args.len();
        self.current_nargout = nargout;

        // Swap environments and execute function body
        std::mem::swap(&mut self.env, &mut func_env);
        let res = self.eval_stmts(&def.body);
        std::mem::swap(&mut self.env, &mut func_env);

        self.current_nargin = prev_nargin;
        self.current_nargout = prev_nargout;

        res?;

        let fixed_outputs_len = if has_varargout {
            def.outputs.len().saturating_sub(1)
        } else {
            def.outputs.len()
        };

        let mut result_outputs = Vec::new();
        for i in 0..fixed_outputs_len {
            let out_name = &def.outputs[i];
            let val = func_env.get(out_name).unwrap_or_else(Value::empty);
            result_outputs.push(val);
        }

        if has_varargout {
            if let Some(Value::Cell(ca)) = func_env.get("varargout") {
                for item in ca.as_slice() {
                    result_outputs.push(item.clone());
                }
            }
        }

        Ok(result_outputs)
    }

    /// Resolves and loads an M-file dynamically from the configured search paths.
    pub fn resolve_and_load_mfile(&mut self, name: &str) -> Result<bool, EvalError> {
        let filename = format!("{name}.m");
        for search_dir in &self.search_paths.clone() {
            let candidate = search_dir.join(&filename);
            if candidate.is_file() {
                if let Ok(source) = fs::read_to_string(&candidate) {
                    let stmts = rustlab_parse::parse(&source)?;
                    let mut has_func = false;
                    for stmt in &stmts {
                        if let Stmt::Function(ref def) = stmt {
                            self.functions.insert(def.name.clone(), def.clone());
                            has_func = true;
                        }
                    }
                    if !has_func {
                        // It is a script file: execute statements sequentially in caller's environment
                        for stmt in &stmts {
                            let flow = self.eval_stmt(stmt)?;
                            if matches!(flow, Flow::Return(_)) {
                                break;
                            }
                        }
                    }
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn eval_which(&mut self, args: &[Expr], span: Span) -> Result<Value, EvalError> {
        if args.is_empty() {
            return Err(EvalError::UserError {
                identifier: "MATLAB:minrhs".to_string(),
                message: "which requires at least 1 input.".to_string(),
                span,
            });
        }
        let target_val = self.eval_expr(&args[0])?;
        let name = match target_val {
            Value::Char(c) => c.as_slice().iter().collect::<String>(),
            Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
            _ => format!("{target_val}"),
        };

        if self.env.get(&name).is_some() {
            return Ok(Value::char_vector(&format!("'{name}' is a variable.")));
        }
        if self.builtins.contains_key(&name) {
            return Ok(Value::char_vector(&format!(
                "'{name}' is a built-in function."
            )));
        }
        if self.functions.contains_key(&name) {
            return Ok(Value::char_vector(&format!(
                "'{name}' is a user-defined function."
            )));
        }
        let filename = format!("{name}.m");
        for search_dir in &self.search_paths {
            let candidate = search_dir.join(&filename);
            if candidate.is_file() {
                return Ok(Value::char_vector(&candidate.to_string_lossy()));
            }
        }
        Ok(Value::char_vector(&format!("'{name}' not found.")))
    }

    fn eval_exist(&mut self, args: &[Expr], span: Span) -> Result<Value, EvalError> {
        if args.is_empty() {
            return Err(EvalError::UserError {
                identifier: "MATLAB:minrhs".to_string(),
                message: "exist requires at least 1 input.".to_string(),
                span,
            });
        }
        let target_val = self.eval_expr(&args[0])?;
        let name = match target_val {
            Value::Char(c) => c.as_slice().iter().collect::<String>(),
            Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
            _ => format!("{target_val}"),
        };

        let filter = if args.len() >= 2 {
            let fval = self.eval_expr(&args[1])?;
            match fval {
                Value::Char(c) => Some(c.as_slice().iter().collect::<String>().to_lowercase()),
                Value::String(s) => s.as_slice().first().map(|st| st.to_lowercase()),
                _ => None,
            }
        } else {
            None
        };

        let is_var = self.env.get(&name).is_some();
        let is_builtin = self.builtins.contains_key(&name);
        let is_func = self.functions.contains_key(&name);
        let mut is_file = false;
        let mut is_dir = std::path::Path::new(&name).is_dir();

        let filename = format!("{name}.m");
        for search_dir in &self.search_paths {
            if search_dir.join(&filename).is_file() {
                is_file = true;
                break;
            }
            if search_dir.join(&name).is_file() {
                is_file = true;
                break;
            }
            if search_dir.join(&name).is_dir() {
                is_dir = true;
            }
        }

        let code = match filter.as_deref() {
            Some("var") => {
                if is_var {
                    1.0
                } else {
                    0.0
                }
            }
            Some("builtin") => {
                if is_builtin {
                    5.0
                } else {
                    0.0
                }
            }
            Some("file") => {
                if is_file || is_func {
                    2.0
                } else if is_dir {
                    7.0
                } else {
                    0.0
                }
            }
            Some("dir") => {
                if is_dir {
                    7.0
                } else {
                    0.0
                }
            }
            _ => {
                if is_var {
                    1.0
                } else if is_builtin {
                    5.0
                } else if is_func || is_file {
                    2.0
                } else if is_dir {
                    7.0
                } else {
                    0.0
                }
            }
        };

        Ok(Value::scalar(code))
    }

    fn eval_addpath(&mut self, args: &[Expr], _span: Span) -> Result<Value, EvalError> {
        for a in args {
            let val = self.eval_expr(a)?;
            let p_str = match val {
                Value::Char(c) => c.as_slice().iter().collect::<String>(),
                Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
                _ => continue,
            };
            let pb = PathBuf::from(p_str);
            if !self.search_paths.contains(&pb) {
                self.search_paths.insert(0, pb);
            }
        }
        Ok(Value::empty())
    }

    fn eval_rmpath(&mut self, args: &[Expr], _span: Span) -> Result<Value, EvalError> {
        for a in args {
            let val = self.eval_expr(a)?;
            let p_str = match val {
                Value::Char(c) => c.as_slice().iter().collect::<String>(),
                Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
                _ => continue,
            };
            let pb = PathBuf::from(p_str);
            self.search_paths.retain(|p| p != &pb);
        }
        Ok(Value::empty())
    }

    fn eval_cell_deref(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: Span,
    ) -> Result<Value, EvalError> {
        let val = self.eval_expr(callee)?;
        match val {
            Value::Cell(da) => {
                if args.len() == 1 {
                    let total_len = da.numel();
                    self.indexing_stack.push(IndexingContext {
                        current_dim_len: total_len,
                        total_len,
                    });
                    let idx_res = self.eval_expr(&args[0]);
                    self.indexing_stack.pop();
                    let idx_val = idx_res?;
                    let idx = idx_val
                        .as_scalar_f64()
                        .map_err(|e| EvalError::Math { error: e, span })?
                        as usize;
                    let elem = da
                        .get_linear(idx)
                        .map_err(|e| EvalError::Math { error: e, span })?;
                    Ok(elem.clone())
                } else {
                    let mut subs = Vec::with_capacity(args.len());
                    let shp = da.shape().to_vec();
                    for (d, a) in args.iter().enumerate() {
                        let dim_len = if d == args.len() - 1 && shp.len() > args.len() {
                            shp[d..].iter().product()
                        } else {
                            shp.get(d).copied().unwrap_or(1)
                        };
                        self.indexing_stack.push(IndexingContext {
                            current_dim_len: dim_len,
                            total_len: da.numel(),
                        });
                        let sub_res = self.eval_expr(a);
                        self.indexing_stack.pop();
                        let sub_val = sub_res?;
                        let s = sub_val
                            .as_scalar_f64()
                            .map_err(|e| EvalError::Math { error: e, span })?
                            as usize;
                        subs.push(s);
                    }
                    let elem = da
                        .get_subscript(&subs)
                        .map_err(|e| EvalError::Math { error: e, span })?;
                    Ok(elem.clone())
                }
            }
            _ => Err(EvalError::UserError {
                identifier: "MATLAB:cellRefOnNonCell".to_string(),
                message: "Cell contents reference from a non-cell array object.".to_string(),
                span,
            }),
        }
    }

    fn eval_matrix_literal(&mut self, rows: &[Vec<Expr>], span: Span) -> Result<Value, EvalError> {
        if rows.is_empty() {
            return Ok(Value::empty());
        }

        let mut evaled_rows = Vec::with_capacity(rows.len());
        let mut all_chars = true;

        for r in rows {
            let mut row_elems = Vec::with_capacity(r.len());
            for e in r {
                let v = self.eval_expr(e)?;
                if !matches!(v, Value::Char(_)) {
                    all_chars = false;
                }
                row_elems.push(v);
            }
            evaled_rows.push(row_elems);
        }

        if all_chars {
            let mut char_rows: Vec<Vec<char>> = Vec::new();
            for r in evaled_rows {
                let mut row_chars = Vec::new();
                for v in r {
                    if let Value::Char(ca) = v {
                        row_chars.extend_from_slice(ca.as_slice());
                    }
                }
                char_rows.push(row_chars);
            }
            let num_rows = char_rows.len();
            let num_cols = char_rows[0].len();
            for r in &char_rows {
                if r.len() != num_cols {
                    return Err(EvalError::Math {
                        error: rustlab_math::MathError::dim_agree(
                            &[num_rows, num_cols],
                            &[num_rows, r.len()],
                        ),
                        span,
                    });
                }
            }
            let mut col_major_chars = Vec::with_capacity(num_rows * num_cols);
            for c_idx in 0..num_cols {
                for r in &char_rows {
                    col_major_chars.push(r[c_idx]);
                }
            }
            let arr = DenseArray::from_vec(vec![num_rows, num_cols], col_major_chars)
                .map_err(|e| EvalError::Math { error: e, span })?;
            return Ok(Value::Char(arr));
        }

        let all_logical = evaled_rows
            .iter()
            .all(|r| r.iter().all(|v| matches!(v, Value::Logical(_))));
        if all_logical {
            let mut bool_rows: Vec<Vec<bool>> = Vec::new();
            for r in &evaled_rows {
                let mut row_bools = Vec::new();
                for v in r {
                    if let Value::Logical(la) = v {
                        row_bools.extend_from_slice(la.as_slice());
                    }
                }
                bool_rows.push(row_bools);
            }
            let num_rows = bool_rows.len();
            let num_cols = bool_rows[0].len();
            for r in &bool_rows {
                if r.len() != num_cols {
                    return Err(EvalError::Math {
                        error: rustlab_math::MathError::dim_agree(
                            &[num_rows, num_cols],
                            &[num_rows, r.len()],
                        ),
                        span,
                    });
                }
            }
            let mut col_major_bools = Vec::with_capacity(num_rows * num_cols);
            for c_idx in 0..num_cols {
                for r in &bool_rows {
                    col_major_bools.push(r[c_idx]);
                }
            }
            let arr = DenseArray::from_vec(vec![num_rows, num_cols], col_major_bools)
                .map_err(|e| EvalError::Math { error: e, span })?;
            return Ok(Value::Logical(arr));
        }

        let any_complex = evaled_rows
            .iter()
            .any(|r| r.iter().any(|v| matches!(v, Value::ComplexDouble(_))));

        if any_complex {
            let mut final_rows: Vec<Vec<rustlab_math::Complex64>> = Vec::new();
            for r in evaled_rows {
                let mut row_nums = Vec::new();
                for v in r {
                    match v {
                        Value::ComplexDouble(ca) => {
                            row_nums.extend_from_slice(ca.as_slice());
                        }
                        Value::Double(da) => {
                            for &x in da.as_slice() {
                                row_nums.push(rustlab_math::Complex64::new(x, 0.0));
                            }
                        }
                        Value::Logical(la) => {
                            for &b in la.as_slice() {
                                row_nums.push(rustlab_math::Complex64::new(
                                    if b { 1.0 } else { 0.0 },
                                    0.0,
                                ));
                            }
                        }
                        other => {
                            let da = other
                                .to_double_array()
                                .map_err(|e| EvalError::Math { error: e, span })?;
                            for &x in da.as_slice() {
                                row_nums.push(rustlab_math::Complex64::new(x, 0.0));
                            }
                        }
                    }
                }
                final_rows.push(row_nums);
            }

            let num_rows = final_rows.len();
            let num_cols = final_rows[0].len();
            for r in &final_rows {
                if r.len() != num_cols {
                    return Err(EvalError::Math {
                        error: rustlab_math::MathError::dim_agree(
                            &[num_rows, num_cols],
                            &[num_rows, r.len()],
                        ),
                        span,
                    });
                }
            }

            let mut col_major_data = Vec::with_capacity(num_rows * num_cols);
            for c_idx in 0..num_cols {
                for r in &final_rows {
                    col_major_data.push(r[c_idx]);
                }
            }

            let arr = DenseArray::from_vec(vec![num_rows, num_cols], col_major_data)
                .map_err(|e| EvalError::Math { error: e, span })?;
            return Ok(Value::ComplexDouble(arr));
        }

        let mut final_rows: Vec<Vec<f64>> = Vec::new();
        for r in evaled_rows {
            let mut row_nums = Vec::new();
            for v in r {
                let da = v
                    .to_double_array()
                    .map_err(|e| EvalError::Math { error: e, span })?;
                row_nums.extend_from_slice(da.as_slice());
            }
            final_rows.push(row_nums);
        }

        let num_rows = final_rows.len();
        let num_cols = final_rows[0].len();
        for r in &final_rows {
            if r.len() != num_cols {
                return Err(EvalError::Math {
                    error: rustlab_math::MathError::dim_agree(
                        &[num_rows, num_cols],
                        &[num_rows, r.len()],
                    ),
                    span,
                });
            }
        }

        let mut col_major_data = Vec::with_capacity(num_rows * num_cols);
        for c_idx in 0..num_cols {
            for r in &final_rows {
                col_major_data.push(r[c_idx]);
            }
        }

        let arr = DenseArray::from_vec(vec![num_rows, num_cols], col_major_data)
            .map_err(|e| EvalError::Math { error: e, span })?;
        Ok(Value::Double(arr))
    }

    fn eval_cell_literal(&mut self, rows: &[Vec<Expr>], span: Span) -> Result<Value, EvalError> {
        if rows.is_empty() {
            let arr = DenseArray::from_vec(vec![0, 0], Vec::new())
                .map_err(|e| EvalError::Math { error: e, span })?;
            return Ok(Value::Cell(arr));
        }

        let mut evaled_rows = Vec::with_capacity(rows.len());
        for r in rows {
            let mut row_elems = Vec::with_capacity(r.len());
            for e in r {
                let v = self.eval_expr(e)?;
                row_elems.push(v);
            }
            evaled_rows.push(row_elems);
        }

        let num_rows = evaled_rows.len();
        let num_cols = evaled_rows[0].len();
        for r in &evaled_rows {
            if r.len() != num_cols {
                return Err(EvalError::Math {
                    error: rustlab_math::MathError::dim_agree(
                        &[num_rows, num_cols],
                        &[num_rows, r.len()],
                    ),
                    span,
                });
            }
        }

        let mut col_major_items = Vec::with_capacity(num_rows * num_cols);
        for c_idx in 0..num_cols {
            for r in &evaled_rows {
                col_major_items.push(r[c_idx].clone());
            }
        }

        let arr = DenseArray::from_vec(vec![num_rows, num_cols], col_major_items)
            .map_err(|e| EvalError::Math { error: e, span })?;
        Ok(Value::Cell(arr))
    }

    fn eval_binary_op(
        &self,
        op: BinaryOp,
        left: &Value,
        right: &Value,
        span: Span,
    ) -> Result<Value, EvalError> {
        match op {
            BinaryOp::Add => left
                .add(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Sub => left
                .sub(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Mul => left
                .mul(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::DotMul => left
                .dot_mul(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::DotDiv => left
                .dot_div(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::DotLeftDiv => left
                .dot_left_div(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Div => {
                if left.is_scalar() && right.is_scalar() {
                    left.dot_div(right)
                        .map_err(|e| EvalError::Math { error: e, span })
                } else if let Some(builtin) = self.builtins.get("mrdivide") {
                    let mut outs = builtin.call(&[left.clone(), right.clone()], 1)?;
                    Ok(outs.pop().unwrap_or_else(Value::empty))
                } else {
                    left.dot_div(right)
                        .map_err(|e| EvalError::Math { error: e, span })
                }
            }
            BinaryOp::LeftDiv => {
                if left.is_scalar() && right.is_scalar() {
                    left.dot_left_div(right)
                        .map_err(|e| EvalError::Math { error: e, span })
                } else if let Some(builtin) = self.builtins.get("mldivide") {
                    let mut outs = builtin.call(&[left.clone(), right.clone()], 1)?;
                    Ok(outs.pop().unwrap_or_else(Value::empty))
                } else {
                    left.dot_left_div(right)
                        .map_err(|e| EvalError::Math { error: e, span })
                }
            }
            BinaryOp::Pow | BinaryOp::DotPow => left
                .dot_pow(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Eq => left
                .eq(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::NotEq => left
                .not_eq(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Lt => left
                .lt(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::LtEq => left
                .lt_eq(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Gt => left
                .gt(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::GtEq => left
                .gt_eq(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::And | BinaryOp::AndAnd => left
                .and(right)
                .map_err(|e| EvalError::Math { error: e, span }),
            BinaryOp::Or | BinaryOp::OrOr => left
                .or(right)
                .map_err(|e| EvalError::Math { error: e, span }),
        }
    }

    fn eval_unary_op(&self, op: UnaryOp, val: &Value, span: Span) -> Result<Value, EvalError> {
        let res = match op {
            UnaryOp::Plus => Ok(val.clone()),
            UnaryOp::Minus => val.neg(),
            UnaryOp::Not => val.not(),
            UnaryOp::Transpose => val.transpose(),
            UnaryOp::ConjugateTranspose => val.ctranspose(),
        };
        res.map_err(|e| EvalError::Math { error: e, span })
    }

    fn is_truthy(&self, val: &Value) -> Result<bool, EvalError> {
        if val.is_empty() {
            return Ok(false);
        }
        let da = val.to_double_array().map_err(|e| EvalError::Math {
            error: e,
            span: Span::default(),
        })?;
        // MATLAB truthiness: all elements must be non-zero
        Ok(da.as_slice().iter().all(|&x| x != 0.0))
    }

    fn are_equal(&self, a: &Value, b: &Value) -> Result<bool, EvalError> {
        let eq_val = a.eq(b).map_err(|e| EvalError::Math {
            error: e,
            span: Span::default(),
        })?;
        if let Value::Logical(arr) = eq_val {
            Ok(arr.as_slice().iter().all(|&b| b))
        } else {
            Ok(false)
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_free_variables(expr: &Expr, bound: &HashSet<String>, out: &mut HashSet<String>) {
    match expr {
        Expr::Ident(name, _) => {
            if !bound.contains(name) {
                out.insert(name.clone());
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_free_variables(left, bound, out);
            collect_free_variables(right, bound, out);
        }
        Expr::Unary { expr, .. } => {
            collect_free_variables(expr, bound, out);
        }
        Expr::Range {
            start, step, end, ..
        } => {
            collect_free_variables(start, bound, out);
            if let Some(s) = step {
                collect_free_variables(s, bound, out);
            }
            collect_free_variables(end, bound, out);
        }
        Expr::Matrix { rows, .. } | Expr::CellArray { rows, .. } => {
            for row in rows {
                for e in row {
                    collect_free_variables(e, bound, out);
                }
            }
        }
        Expr::Index { callee, args, .. } | Expr::CellIndex { callee, args, .. } => {
            collect_free_variables(callee, bound, out);
            for a in args {
                collect_free_variables(a, bound, out);
            }
        }
        Expr::DotAccess { expr, .. } => {
            collect_free_variables(expr, bound, out);
        }
        Expr::DynamicDotAccess {
            expr, field_expr, ..
        } => {
            collect_free_variables(expr, bound, out);
            collect_free_variables(field_expr, bound, out);
        }
        Expr::AnonymousFunction { params, body, .. } => {
            let mut inner_bound = bound.clone();
            for p in params {
                inner_bound.insert(p.clone());
            }
            collect_free_variables(body, &inner_bound, out);
        }
        _ => {}
    }
}
