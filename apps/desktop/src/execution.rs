//! Worker thread execution engine, Session runtime wrapper, and graphics state.

use rustlab_cli::format::format_eval_error;
use rustlab_eval::{EvalError, Interpreter};
use rustlab_math::{DenseArray, Value};
use rustlab_stdlib::new_interpreter;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Command transmitted to control execution paused at a breakpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugCommand {
    Continue,
    Step,
    Stop,
}

/// Synchronization primitive for pausing and resuming execution at breakpoints.
#[derive(Default)]
pub struct DebugSync {
    pub signal: Mutex<Option<DebugCommand>>,
    pub cvar: Condvar,
}

impl DebugSync {
    pub fn new() -> Self {
        Self {
            signal: Mutex::new(None),
            cvar: Condvar::new(),
        }
    }

    pub fn send(&self, cmd: DebugCommand) {
        let mut lock = self.signal.lock().unwrap_or_else(|e| e.into_inner());
        *lock = Some(cmd);
        self.cvar.notify_all();
    }
}

/// Callback invoked when a breakpoint pauses execution.
pub type PauseCallback<'a> = &'a (dyn Fn(usize, &[VariableRow]) + Send + Sync);

/// Converts a byte offset within source text to a 1-based line number.
pub fn offset_to_line(source: &str, offset: usize) -> usize {
    let safe_offset = offset.min(source.len());
    source[..safe_offset].chars().filter(|&c| c == '\n').count() + 1
}

/// Single variable entry displayed in the Workspace browser.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableRow {
    pub name: String,
    pub type_name: String,
    pub size_str: String,
    pub bytes: usize,
    pub preview: String,
}

/// A scientific graphical plot element.
#[derive(Clone, Debug)]
pub enum PlotItem {
    Line {
        name: String,
        x: Vec<f64>,
        y: Vec<f64>,
        color: [u8; 4],
        width: f32,
    },
    Scatter {
        name: String,
        x: Vec<f64>,
        y: Vec<f64>,
        color: [u8; 4],
        radius: f32,
    },
    Bar {
        name: String,
        x: Vec<f64>,
        y: Vec<f64>,
        color: [u8; 4],
    },
    Histogram {
        name: String,
        values: Vec<f64>,
        bins: usize,
        color: [u8; 4],
    },
}

/// A graphical figure window holding plot series and axes metadata.
#[derive(Clone, Debug)]
pub struct FigureState {
    pub id: usize,
    pub title: String,
    pub items: Vec<PlotItem>,
    pub x_label: String,
    pub y_label: String,
    pub grid: bool,
}

impl Default for FigureState {
    fn default() -> Self {
        Self {
            id: 1,
            title: "Figure 1".to_string(),
            items: Vec::new(),
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            grid: true,
        }
    }
}

/// Manages multiple figures and currently focused figure.
#[derive(Clone, Debug, Default)]
pub struct GraphicsState {
    pub figures: Vec<FigureState>,
    pub current_figure: Option<usize>,
}

impl GraphicsState {
    pub fn get_or_create_figure(&mut self, id: usize) -> &mut FigureState {
        if let Some(pos) = self.figures.iter().position(|f| f.id == id) {
            self.current_figure = Some(id);
            return &mut self.figures[pos];
        }
        let fig = FigureState {
            id,
            title: format!("Figure {id}"),
            items: Vec::new(),
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            grid: true,
        };
        self.figures.push(fig);
        self.current_figure = Some(id);
        self.figures.last_mut().expect("just pushed")
    }

    pub fn current_figure_mut(&mut self) -> &mut FigureState {
        let id = self.current_figure.unwrap_or(1);
        self.get_or_create_figure(id)
    }

    pub fn close_figure(&mut self, id: usize) {
        self.figures.retain(|f| f.id != id);
        if self.current_figure == Some(id) {
            self.current_figure = self.figures.last().map(|f| f.id);
        }
    }

    pub fn close_all(&mut self) {
        self.figures.clear();
        self.current_figure = None;
    }
}

/// Result returned by an execution pass.
#[derive(Clone, Debug)]
pub struct RunResult {
    pub output: String,
    pub error: Option<String>,
    pub execution_time: Duration,
    pub variables: Vec<VariableRow>,
    pub graphics: GraphicsState,
    pub paused_at_line: Option<usize>,
}

/// Persistent MATLAB execution session wrapping the RustLab Interpreter.
pub struct Session {
    pub interp: Interpreter,
    pub graphics: Arc<Mutex<GraphicsState>>,
    pub breakpoints: Arc<Mutex<HashSet<usize>>>,
    pub current_line: Arc<Mutex<Option<usize>>>,
    pub custom_output: Arc<Mutex<Vec<String>>>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        let mut interp = new_interpreter();
        let graphics = Arc::new(Mutex::new(GraphicsState::default()));
        let breakpoints = Arc::new(Mutex::new(HashSet::new()));
        let current_line = Arc::new(Mutex::new(None));
        let custom_output = Arc::new(Mutex::new(Vec::new()));

        // Register GUI-aware plotting and I/O builtins into interpreter
        Self::register_gui_builtins(
            &mut interp,
            Arc::clone(&graphics),
            Arc::clone(&custom_output),
        );

        Self {
            interp,
            graphics,
            breakpoints,
            current_line,
            custom_output,
        }
    }

    fn register_gui_builtins(
        interp: &mut Interpreter,
        graphics: Arc<Mutex<GraphicsState>>,
        custom_output: Arc<Mutex<Vec<String>>>,
    ) {
        let out_disp = Arc::clone(&custom_output);
        interp.register_builtin(
            "disp",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                if args.is_empty() {
                    return Err(EvalError::UserError {
                        identifier: "MATLAB:minrhs".to_string(),
                        message: "disp requires at least 1 input.".to_string(),
                        span: rustlab_parse::Span::default(),
                    });
                }
                let s = match &args[0] {
                    Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
                    Value::String(sa) if sa.is_scalar() => {
                        sa.as_slice().first().cloned().unwrap_or_default()
                    }
                    other => format!("{other}"),
                };
                if let Ok(mut lock) = out_disp.lock() {
                    lock.push(format!("{s}\n"));
                }
                println!("{s}");
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );
        let g_plot = Arc::clone(&graphics);
        interp.register_builtin(
            "plot",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                let mut g = g_plot.lock().unwrap_or_else(|e| e.into_inner());
                let fig = g.current_figure_mut();
                if args.is_empty() {
                    return Ok(vec![Value::Double(DenseArray::empty())]);
                }
                let (x, y) = if args.len() >= 2 {
                    (Self::extract_vec(&args[0]), Self::extract_vec(&args[1]))
                } else {
                    let y = Self::extract_vec(&args[0]);
                    let x = (1..=y.len()).map(|i| i as f64).collect();
                    (x, y)
                };
                let series_idx = fig.items.len() + 1;
                let colors: [[u8; 4]; 5] = [
                    [0, 114, 189, 255],  // MATLAB blue
                    [217, 83, 25, 255],  // MATLAB orange
                    [237, 177, 32, 255], // MATLAB yellow
                    [126, 47, 142, 255], // MATLAB purple
                    [119, 172, 48, 255], // MATLAB green
                ];
                let color = colors[(series_idx - 1) % colors.len()];
                fig.items.push(PlotItem::Line {
                    name: format!("Series {series_idx}"),
                    x,
                    y,
                    color,
                    width: 2.0,
                });
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_scatter = Arc::clone(&graphics);
        interp.register_builtin(
            "scatter",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                let mut g = g_scatter.lock().unwrap_or_else(|e| e.into_inner());
                let fig = g.current_figure_mut();
                let (x, y) = if args.len() >= 2 {
                    (Self::extract_vec(&args[0]), Self::extract_vec(&args[1]))
                } else if !args.is_empty() {
                    let y = Self::extract_vec(&args[0]);
                    let x = (1..=y.len()).map(|i| i as f64).collect();
                    (x, y)
                } else {
                    (Vec::new(), Vec::new())
                };
                fig.items.push(PlotItem::Scatter {
                    name: format!("Scatter {}", fig.items.len() + 1),
                    x,
                    y,
                    color: [217, 83, 25, 255],
                    radius: 4.0,
                });
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_bar = Arc::clone(&graphics);
        interp.register_builtin(
            "bar",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                let mut g = g_bar.lock().unwrap_or_else(|e| e.into_inner());
                let fig = g.current_figure_mut();
                let (x, y) = if args.len() >= 2 {
                    (Self::extract_vec(&args[0]), Self::extract_vec(&args[1]))
                } else if !args.is_empty() {
                    let y = Self::extract_vec(&args[0]);
                    let x = (1..=y.len()).map(|i| i as f64).collect();
                    (x, y)
                } else {
                    (Vec::new(), Vec::new())
                };
                fig.items.push(PlotItem::Bar {
                    name: format!("Bar {}", fig.items.len() + 1),
                    x,
                    y,
                    color: [0, 114, 189, 255],
                });
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_title = Arc::clone(&graphics);
        interp.register_builtin(
            "title",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                if let Some(s) = args.first().and_then(Self::extract_str) {
                    let mut g = g_title.lock().unwrap_or_else(|e| e.into_inner());
                    g.current_figure_mut().title = s;
                }
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_xlabel = Arc::clone(&graphics);
        interp.register_builtin(
            "xlabel",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                if let Some(s) = args.first().and_then(Self::extract_str) {
                    let mut g = g_xlabel.lock().unwrap_or_else(|e| e.into_inner());
                    g.current_figure_mut().x_label = s;
                }
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_ylabel = Arc::clone(&graphics);
        interp.register_builtin(
            "ylabel",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                if let Some(s) = args.first().and_then(Self::extract_str) {
                    let mut g = g_ylabel.lock().unwrap_or_else(|e| e.into_inner());
                    g.current_figure_mut().y_label = s;
                }
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_grid = Arc::clone(&graphics);
        interp.register_builtin(
            "grid",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                let mut g = g_grid.lock().unwrap_or_else(|e| e.into_inner());
                let fig = g.current_figure_mut();
                if let Some(s) = args.first().and_then(Self::extract_str) {
                    if s == "on" {
                        fig.grid = true;
                    } else if s == "off" {
                        fig.grid = false;
                    }
                } else {
                    fig.grid = !fig.grid;
                }
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_figure = Arc::clone(&graphics);
        interp.register_builtin(
            "figure",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                let mut g = g_figure.lock().unwrap_or_else(|e| e.into_inner());
                let id = args
                    .first()
                    .and_then(Self::to_f64_scalar)
                    .map(|f| f.max(1.0) as usize)
                    .unwrap_or_else(|| g.figures.iter().map(|f| f.id).max().unwrap_or(0) + 1);
                g.get_or_create_figure(id);
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_clf = Arc::clone(&graphics);
        interp.register_builtin(
            "clf",
            move |_: &[Value], _: usize| -> Result<Vec<Value>, EvalError> {
                let mut g = g_clf.lock().unwrap_or_else(|e| e.into_inner());
                let fig = g.current_figure_mut();
                fig.items.clear();
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );

        let g_close = Arc::clone(&graphics);
        interp.register_builtin(
            "close",
            move |args: &[Value], _| -> Result<Vec<Value>, EvalError> {
                let mut g = g_close.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(s) = args.first().and_then(Self::extract_str) {
                    if s == "all" {
                        g.close_all();
                        return Ok(vec![Value::Double(DenseArray::empty())]);
                    }
                }
                if let Some(id) = args.first().and_then(Self::to_f64_scalar) {
                    g.close_figure(id as usize);
                } else if let Some(curr) = g.current_figure {
                    g.close_figure(curr);
                }
                Ok(vec![Value::Double(DenseArray::empty())])
            },
        );
    }

    fn to_f64_scalar(val: &Value) -> Option<f64> {
        match val {
            Value::Double(d) if d.numel() == 1 => d.as_slice().first().copied(),
            Value::Single(s) if s.numel() == 1 => s.as_slice().first().map(|&x| x as f64),
            Value::Int32(i) if i.numel() == 1 => i.as_slice().first().map(|&x| x as f64),
            Value::UInt32(u) if u.numel() == 1 => u.as_slice().first().map(|&x| x as f64),
            _ => None,
        }
    }

    fn extract_vec(val: &Value) -> Vec<f64> {
        match val {
            Value::Double(d) => d.as_slice().to_vec(),
            Value::Single(s) => s.as_slice().iter().map(|&x| x as f64).collect(),
            Value::Int32(i) => i.as_slice().iter().map(|&x| x as f64).collect(),
            Value::UInt32(u) => u.as_slice().iter().map(|&x| x as f64).collect(),
            Value::Int64(i) => i.as_slice().iter().map(|&x| x as f64).collect(),
            Value::UInt64(u) => u.as_slice().iter().map(|&x| x as f64).collect(),
            _ => Vec::new(),
        }
    }

    fn extract_str(val: &Value) -> Option<String> {
        match val {
            Value::Char(c) => Some(c.as_slice().iter().collect()),
            Value::String(s) => s.as_slice().first().cloned(),
            _ => None,
        }
    }

    /// Executes code safely with panic isolation and cancellation checking.
    pub fn exec(&mut self, source: &str, cancel: Option<&Arc<AtomicBool>>) -> RunResult {
        self.exec_internal(source, cancel, None, None)
    }

    /// Executes code with statement-level breakpoint debugging.
    pub fn exec_debug(
        &mut self,
        source: &str,
        cancel: Option<&Arc<AtomicBool>>,
        debug_sync: &Arc<DebugSync>,
        on_pause: &(dyn Fn(usize, &[VariableRow]) + Send + Sync),
    ) -> RunResult {
        self.exec_internal(source, cancel, Some(debug_sync), Some(on_pause))
    }

    fn exec_internal(
        &mut self,
        source: &str,
        cancel: Option<&Arc<AtomicBool>>,
        debug_sync: Option<&Arc<DebugSync>>,
        on_pause: Option<PauseCallback>,
    ) -> RunResult {
        let start = Instant::now();
        let mut output = String::new();
        let mut paused_line = None;

        // Catch potential execution panics to isolate worker thread
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Some(flag) = cancel {
                if flag.load(Ordering::SeqCst) {
                    return Err(EvalError::UserError {
                        identifier: "MATLAB:cancelled".to_string(),
                        message: "Operation cancelled by user".to_string(),
                        span: rustlab_parse::Span { start: 0, end: 0 },
                    });
                }
            }

            if let (Some(sync), Some(pause_cb)) = (debug_sync, on_pause) {
                let stmts = rustlab_parse::parse(source)?;
                let bps = {
                    let lock = self.breakpoints.lock().unwrap_or_else(|e| e.into_inner());
                    lock.clone()
                };

                for stmt in &stmts {
                    if let Some(flag) = cancel {
                        if flag.load(Ordering::SeqCst) {
                            return Err(EvalError::UserError {
                                identifier: "MATLAB:cancelled".to_string(),
                                message: "Operation cancelled by user".to_string(),
                                span: stmt.span(),
                            });
                        }
                    }

                    let line_no = offset_to_line(source, stmt.span().start);
                    if bps.contains(&line_no) {
                        if let Ok(mut l) = self.current_line.lock() {
                            *l = Some(line_no);
                        }
                        paused_line = Some(line_no);
                        pause_cb(line_no, &self.variable_rows());

                        // Block worker thread until UI signals continue or stop
                        let mut lock = sync.signal.lock().unwrap_or_else(|e| e.into_inner());
                        while lock.is_none() {
                            lock = sync.cvar.wait(lock).unwrap_or_else(|e| e.into_inner());
                        }
                        let cmd = lock.take().unwrap_or(DebugCommand::Continue);
                        if cmd == DebugCommand::Stop {
                            return Err(EvalError::UserError {
                                identifier: "MATLAB:debugStopped".to_string(),
                                message: "Execution stopped at breakpoint".to_string(),
                                span: stmt.span(),
                            });
                        }
                    }

                    let _ = self.interp.eval_stmt(stmt)?;
                }
                Ok(None)
            } else {
                self.interp.eval_script(source)
            }
        }));

        let (_exec_res, error) = match result {
            Ok(inner_res) => match inner_res {
                Ok(_) => {
                    for line in self.interp.output_log.drain(..) {
                        output.push_str(&line);
                    }
                    if let Ok(mut lock) = self.custom_output.lock() {
                        for line in lock.drain(..) {
                            output.push_str(&line);
                        }
                    }
                    (Ok(()), None)
                }
                Err(err) => {
                    let formatted = format_eval_error(&err, source);
                    self.interp.output_log.clear();
                    (Err(()), Some(formatted))
                }
            },
            Err(panic_err) => {
                let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    format!("Worker panic: {s}")
                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    format!("Worker panic: {s}")
                } else {
                    "Worker panicked during execution".to_string()
                };
                (Err(()), Some(msg))
            }
        };

        let elapsed = start.elapsed();
        let variables = self.variable_rows();
        let graphics = self
            .graphics
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();

        RunResult {
            output,
            error,
            execution_time: elapsed,
            variables,
            graphics,
            paused_at_line: None,
        }
    }

    /// Extracts variable metadata from the interpreter's local environment.
    pub fn variable_rows(&self) -> Vec<VariableRow> {
        let locals = self.interp.env.locals();
        let mut names: Vec<_> = locals.keys().cloned().collect();
        names.sort();

        names
            .into_iter()
            .filter_map(|name| {
                locals.get(&name).map(|val| {
                    let shape = val.shape();
                    let size_str = if shape.is_empty() {
                        "0x0".to_string()
                    } else {
                        shape
                            .iter()
                            .map(|d| d.to_string())
                            .collect::<Vec<_>>()
                            .join("x")
                    };
                    let bytes = Self::estimate_bytes(val);
                    let type_name = val.type_name().to_string();
                    let preview = Self::format_preview(val);

                    VariableRow {
                        name,
                        type_name,
                        size_str,
                        bytes,
                        preview,
                    }
                })
            })
            .collect()
    }

    /// Formats a safe preview string for variable inspector.
    pub fn format_preview(val: &Value) -> String {
        match val {
            Value::Double(d) => {
                let slice = d.as_slice();
                if slice.is_empty() {
                    "[]".to_string()
                } else if slice.len() == 1 {
                    format!("{:.4}", slice[0])
                } else if slice.len() <= 6 {
                    let s: Vec<_> = slice.iter().map(|v| format!("{:.2}", v)).collect();
                    format!("[{}]", s.join("  "))
                } else {
                    format!(
                        "[{:.2}  {:.2}  ... {} elements]",
                        slice[0],
                        slice[1],
                        slice.len()
                    )
                }
            }
            Value::Char(c) => {
                let s: String = c.as_slice().iter().collect();
                if s.len() > 30 {
                    format!("'{}...'", &s[..27])
                } else {
                    format!("'{s}'")
                }
            }
            Value::String(sa) => {
                if let Some(first) = sa.as_slice().first() {
                    format!("\"{first}\"")
                } else {
                    "\"\"".to_string()
                }
            }
            Value::Logical(l) => {
                let slice = l.as_slice();
                if slice.len() == 1 {
                    if slice[0] {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                } else {
                    format!("[{} logicals]", slice.len())
                }
            }
            Value::Sparse(s) => format!(
                "<{}x{} sparse, {} nonzeros>",
                s.shape[0],
                s.shape[1],
                s.values.len()
            ),
            Value::Cell(c) => {
                let shp = c.shape();
                let dims = shp
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("x");
                format!("{{{dims} cell}}")
            }
            Value::Struct(s) => format!("struct with fields: {}", s.fields.join(", ")),
            Value::FunctionHandle(fh) => match &**fh {
                rustlab_math::FunctionHandle::Named(n) => format!("@{n}"),
                rustlab_math::FunctionHandle::Anonymous { params, .. } => {
                    format!("@({}) ...", params.join(","))
                }
            },
            _ => format!("<{}>", val.type_name()),
        }
    }

    fn estimate_bytes(val: &Value) -> usize {
        let count: usize = val.shape().iter().product();
        match val {
            Value::Double(_) => count * 8,
            Value::ComplexDouble(_) => count * 16,
            Value::Single(_) => count * 4,
            Value::Logical(_) => count,
            Value::Int8(_) | Value::UInt8(_) => count,
            Value::Int16(_) | Value::UInt16(_) => count * 2,
            Value::Int32(_) | Value::UInt32(_) => count * 4,
            Value::Int64(_) | Value::UInt64(_) => count * 8,
            Value::Sparse(s) => s.col_ptr.len() * 8 + s.row_ind.len() * 8 + s.values.len() * 8,
            Value::Char(_) => count * 4,
            Value::String(sa) => sa.as_slice().iter().map(|s| s.len() + 24).sum(),
            Value::Cell(_) => count * 24,
            Value::Struct(_) => count * 64,
            Value::FunctionHandle(_) => 32,
        }
    }

    /// Imports a CSV into the persistent session environment as a Matrix value.
    pub fn import_csv(
        &mut self,
        var_name: &str,
        path: &str,
        delimiter: u8,
        has_header: bool,
    ) -> Result<VariableRow, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read file '{}': {e}", path))?;

        let delim_char = delimiter as char;
        let mut rows = Vec::new();
        let start_row = if has_header { 1 } else { 0 };

        for (idx, line) in content.lines().enumerate() {
            if idx < start_row {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parsed_row: Result<Vec<f64>, _> = trimmed
                .split(delim_char)
                .map(|tok| tok.trim().parse::<f64>())
                .collect();
            let row_vals =
                parsed_row.map_err(|_| format!("Non-numeric data found on line {}", idx + 1))?;
            rows.push(row_vals);
        }

        if rows.is_empty() {
            return Err("CSV file contains no numeric rows to import".to_string());
        }

        let num_rows = rows.len();
        let num_cols = rows[0].len();
        for (i, r) in rows.iter().enumerate() {
            if r.len() != num_cols {
                return Err(format!(
                    "Inconsistent column count at row {}: expected {}, found {}",
                    i + 1,
                    num_cols,
                    r.len()
                ));
            }
        }

        // Convert to Column-Major DenseArray
        let mut col_major_data = vec![0.0; num_rows * num_cols];
        for (r_idx, r) in rows.iter().enumerate() {
            for (c_idx, &val) in r.iter().enumerate() {
                col_major_data[c_idx * num_rows + r_idx] = val;
            }
        }

        let arr = DenseArray::from_vec(vec![num_rows, num_cols], col_major_data)
            .map_err(|e| format!("Failed to create matrix: {e}"))?;
        let value = Value::Double(arr);

        // Store into session environment
        self.interp.env.set(var_name, value.clone());

        let size_str = format!("{num_rows}x{num_cols}");
        let bytes = Self::estimate_bytes(&value);
        let preview = Self::format_preview(&value);

        Ok(VariableRow {
            name: var_name.to_string(),
            type_name: "double".to_string(),
            size_str,
            bytes,
            preview,
        })
    }

    /// Clears local workspace variables.
    pub fn clear(&mut self) {
        self.interp.env.clear_locals();
    }
}

/// Request transmitted from the UI thread to the Worker.
#[derive(Debug)]
pub enum WorkerRequest {
    Execute {
        request_id: u64,
        code: String,
    },
    ExecuteDebug {
        request_id: u64,
        code: String,
    },
    DebugContinue,
    DebugStop,
    Stop,
    Clear,
    RefreshWorkspace,
    ImportCsv {
        request_id: u64,
        var_name: String,
        path: String,
        delimiter: u8,
        has_header: bool,
    },
    SetBreakpoints(HashSet<usize>),
    InjectPanicForTesting,
    Shutdown,
}

/// Response returned from the Worker to the UI thread.
#[derive(Debug)]
pub enum WorkerResponse {
    ExecutionFinished {
        request_id: u64,
        result: RunResult,
    },
    BreakpointHit {
        request_id: u64,
        line: usize,
        variables: Vec<VariableRow>,
    },
    WorkspaceUpdated(Vec<VariableRow>),
    ImportFinished {
        request_id: u64,
        result: Result<VariableRow, String>,
    },
    WorkerPanic {
        request_id: u64,
        message: String,
    },
}

/// Client managing the background numerical execution worker thread.
pub struct WorkerClient {
    pub tx: Sender<WorkerRequest>,
    pub rx: Receiver<WorkerResponse>,
    pub cancel_flag: Arc<AtomicBool>,
    pub is_running: Arc<AtomicBool>,
    pub debug_sync: Arc<DebugSync>,
    pub next_request_id: u64,
}

impl Default for WorkerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerClient {
    pub fn new() -> Self {
        let (tx_req, rx_req) = channel::<WorkerRequest>();
        let (tx_resp, rx_resp) = channel::<WorkerResponse>();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(false));
        let debug_sync = Arc::new(DebugSync::new());

        let cancel_thread = Arc::clone(&cancel_flag);
        let is_running_thread = Arc::clone(&is_running);
        let debug_sync_thread = Arc::clone(&debug_sync);

        thread::Builder::new()
            .name("rustlab-worker".to_string())
            .spawn(move || {
                let mut session = Session::new();

                while let Ok(req) = rx_req.recv() {
                    let req_id = match &req {
                        WorkerRequest::Execute { request_id, .. } => *request_id,
                        WorkerRequest::ExecuteDebug { request_id, .. } => *request_id,
                        WorkerRequest::ImportCsv { request_id, .. } => *request_id,
                        _ => 0,
                    };

                    let tx_resp_clone = tx_resp.clone();
                    let cancel_thread_clone = Arc::clone(&cancel_thread);
                    let is_running_thread_clone = Arc::clone(&is_running_thread);
                    let debug_sync_thread_clone = Arc::clone(&debug_sync_thread);

                    let panic_result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match req {
                            WorkerRequest::Execute { request_id, code } => {
                                is_running_thread_clone.store(true, Ordering::SeqCst);
                                cancel_thread_clone.store(false, Ordering::SeqCst);

                                let result = session.exec(&code, Some(&cancel_thread_clone));
                                is_running_thread_clone.store(false, Ordering::SeqCst);

                                let _ = tx_resp_clone
                                    .send(WorkerResponse::ExecutionFinished { request_id, result });
                            }
                            WorkerRequest::ExecuteDebug { request_id, code } => {
                                is_running_thread_clone.store(true, Ordering::SeqCst);
                                cancel_thread_clone.store(false, Ordering::SeqCst);

                                let tx_pause = tx_resp_clone.clone();
                                let on_pause = move |line: usize, vars: &[VariableRow]| {
                                    let _ = tx_pause.send(WorkerResponse::BreakpointHit {
                                        request_id,
                                        line,
                                        variables: vars.to_vec(),
                                    });
                                };

                                let result = session.exec_debug(
                                    &code,
                                    Some(&cancel_thread_clone),
                                    &debug_sync_thread_clone,
                                    &on_pause,
                                );
                                is_running_thread_clone.store(false, Ordering::SeqCst);

                                let _ = tx_resp_clone
                                    .send(WorkerResponse::ExecutionFinished { request_id, result });
                            }
                            WorkerRequest::DebugContinue => {
                                debug_sync_thread_clone.send(DebugCommand::Continue);
                            }
                            WorkerRequest::DebugStop => {
                                debug_sync_thread_clone.send(DebugCommand::Stop);
                            }
                            WorkerRequest::Stop => {
                                cancel_thread_clone.store(true, Ordering::SeqCst);
                            }
                            WorkerRequest::Clear => {
                                session.clear();
                                let vars = session.variable_rows();
                                let _ = tx_resp_clone.send(WorkerResponse::WorkspaceUpdated(vars));
                            }
                            WorkerRequest::RefreshWorkspace => {
                                let vars = session.variable_rows();
                                let _ = tx_resp_clone.send(WorkerResponse::WorkspaceUpdated(vars));
                            }
                            WorkerRequest::ImportCsv {
                                request_id,
                                var_name,
                                path,
                                delimiter,
                                has_header,
                            } => {
                                let res =
                                    session.import_csv(&var_name, &path, delimiter, has_header);
                                let _ = tx_resp_clone.send(WorkerResponse::ImportFinished {
                                    request_id,
                                    result: res,
                                });
                            }
                            WorkerRequest::SetBreakpoints(bps) => {
                                if let Ok(mut lock) = session.breakpoints.lock() {
                                    *lock = bps;
                                }
                            }
                            WorkerRequest::InjectPanicForTesting => {
                                panic!("Controlled worker test panic");
                            }
                            WorkerRequest::Shutdown => {}
                        }));

                    if let Err(panic_err) = panic_result {
                        is_running_thread.store(false, Ordering::SeqCst);
                        cancel_thread.store(false, Ordering::SeqCst);

                        let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unhandled worker execution panic".to_string()
                        };

                        // Reinitialize session to recover worker with clean state
                        session = Session::new();

                        let _ = tx_resp.send(WorkerResponse::WorkerPanic {
                            request_id: req_id,
                            message: msg,
                        });
                    }
                }
            })
            .expect("Failed to spawn background numerical execution worker thread");

        Self {
            tx: tx_req,
            rx: rx_resp,
            cancel_flag,
            is_running,
            debug_sync,
            next_request_id: 1,
        }
    }

    pub fn execute(&mut self, code: String) -> u64 {
        let req_id = self.next_request_id;
        self.next_request_id += 1;
        let _ = self.tx.send(WorkerRequest::Execute {
            request_id: req_id,
            code,
        });
        req_id
    }

    pub fn execute_debug(&mut self, code: String) -> u64 {
        let req_id = self.next_request_id;
        self.next_request_id += 1;
        let _ = self.tx.send(WorkerRequest::ExecuteDebug {
            request_id: req_id,
            code,
        });
        req_id
    }

    pub fn debug_continue(&self) {
        self.debug_sync.send(DebugCommand::Continue);
    }

    pub fn debug_stop(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        self.debug_sync.send(DebugCommand::Stop);
    }

    pub fn stop(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        let _ = self.tx.send(WorkerRequest::Stop);
    }

    pub fn clear(&self) {
        let _ = self.tx.send(WorkerRequest::Clear);
    }

    pub fn refresh_workspace(&self) {
        let _ = self.tx.send(WorkerRequest::RefreshWorkspace);
    }

    pub fn set_breakpoints(&self, bps: HashSet<usize>) {
        let _ = self.tx.send(WorkerRequest::SetBreakpoints(bps));
    }

    pub fn inject_panic_for_testing(&self) {
        let _ = self.tx.send(WorkerRequest::InjectPanicForTesting);
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(WorkerRequest::Shutdown);
    }

    pub fn import_csv(
        &mut self,
        var_name: String,
        path: String,
        delimiter: u8,
        has_header: bool,
    ) -> u64 {
        let req_id = self.next_request_id;
        self.next_request_id += 1;
        let _ = self.tx.send(WorkerRequest::ImportCsv {
            request_id: req_id,
            var_name,
            path,
            delimiter,
            has_header,
        });
        req_id
    }
}
