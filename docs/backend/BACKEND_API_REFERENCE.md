# MatRust Studio Backend API Reference

## 1. Overview

This document provides a complete reference for the internal Rust APIs powering MatRust Studio's computing backend.

## 2. Runtime & Worker APIs

### `matrust_studio::execution::WorkerClient`
The primary asynchronous interface used by the GUI to dispatch execution requests to the background worker thread.

```rust
impl WorkerClient {
    pub fn new() -> Self;
    pub fn execute(&self, code: String);
    pub fn execute_debug(&self, code: String);
    pub fn debug_continue(&self);
    pub fn debug_stop(&self);
    pub fn stop(&self);
    pub fn clear(&self);
    pub fn refresh_workspace(&self);
    pub fn import_csv(&self, var_name: String, path: String, delimiter: u8, has_header: bool);
    pub fn set_breakpoints(&self, lines: HashSet<usize>);
}
```

### `matrust_studio::execution::Session`
Encapsulates an active MATLAB interpreter session with graphics and workspace state:

```rust
impl Session {
    pub fn new() -> Self;
    pub fn exec(&mut self, code: &str, cancel_flag: Option<&Arc<AtomicBool>>) -> RunResult;
    pub fn exec_debug(
        &mut self,
        code: &str,
        cancel_flag: Option<&Arc<AtomicBool>>,
        debug_sync: &Arc<DebugSync>,
        on_pause: &PauseCallback,
    ) -> RunResult;
    pub fn variable_rows(&self) -> Vec<VariableRow>;
    pub fn import_csv(&mut self, var_name: &str, path: &str, delimiter: u8, has_header: bool) -> Result<VariableRow, String>;
    pub fn clear(&mut self);
}
```

## 3. Evaluator & Environment APIs (`crates/rustlab-eval`)

### `rustlab_eval::Interpreter`
```rust
impl Interpreter {
    pub fn new() -> Self;
    pub fn eval_script(&mut self, source: &str) -> Result<Option<Value>, EvalError>;
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, EvalError>;
    pub fn register_builtin<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[Value], usize) -> Result<Vec<Value>, EvalError> + 'static;
    pub fn builtin_names(&self) -> Vec<String>;
    pub fn call_function_handle(
        &mut self,
        handle: &FunctionHandle,
        args: Vec<Value>,
        nargout: usize,
        span: Span,
    ) -> Result<Vec<Value>, EvalError>;
}
```

### `rustlab_eval::Environment`
```rust
impl Environment {
    pub fn new() -> Self;
    pub fn get(&self, name: &str) -> Option<Value>;
    pub fn set(&mut self, name: &str, val: Value);
    pub fn exists(&self, name: &str) -> bool;
    pub fn clear_locals(&mut self);
    pub fn locals(&self) -> HashMap<String, Value>;
}
```

## 4. Value & Math Operations (`crates/rustlab-math`)

### `rustlab_math::Value`
```rust
pub enum Value {
    Double(DenseArray<f64>),
    ComplexDouble(DenseArray<Complex64>),
    Single(DenseArray<f32>),
    Logical(DenseArray<bool>),
    Int8(DenseArray<i8>),
    Int16(DenseArray<i16>),
    Int32(DenseArray<i32>),
    Int64(DenseArray<i64>),
    UInt8(DenseArray<u8>),
    UInt16(DenseArray<u16>),
    UInt32(DenseArray<u32>),
    UInt64(DenseArray<u64>),
    Char(DenseArray<char>),
    String(DenseArray<String>),
    Cell(DenseArray<Value>),
    Struct(StructArray),
    FunctionHandle(Arc<FunctionHandle>),
    Sparse(CscMatrix<f64>),
}
```

Key value methods:
- `shape(&self) -> &[usize]`
- `numel(&self) -> usize`
- `type_name(&self) -> &'static str`
- `is_scalar(&self) -> bool`
- `as_scalar_f64(&self) -> Result<f64, MathError>`
- `to_double_array(&self) -> Result<DenseArray<f64>, MathError>`
