//! Persistent GNU Octave subprocess session and lifecycle management.

use crate::detect::find_octave_binary;
use crate::protocol::{OctaveErrorInfo, OctaveRequest, OctaveResponse, OctaveValue};
use crate::safety::check_call_safety;
use rustlab_math::{DenseArray, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use thiserror::Error;

/// Configuration parameters for an active Octave session.
#[derive(Debug, Clone)]
pub struct OctaveConfig {
    /// If true, disables the safety sandbox allow-list and permits dangerous functions.
    pub allow_unsafe: bool,
    /// Execution timeout per call in milliseconds.
    pub timeout_ms: u64,
}

impl Default for OctaveConfig {
    fn default() -> Self {
        Self {
            allow_unsafe: false,
            timeout_ms: 10_000,
        }
    }
}

/// Errors occurring during Octave subprocess communication or evaluation.
#[derive(Debug, Error)]
pub enum OctaveError {
    #[error("GNU Octave executable (octave-cli) not found on host system")]
    NotAvailable,

    #[error("Failed to start Octave process: {0}")]
    ProcessStart(String),

    #[error("Octave process I/O error: {0}")]
    ProcessIo(String),

    #[error("Octave protocol error: {0}")]
    Protocol(String),

    #[error("Octave execution timed out after {0} ms")]
    Timeout(u64),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Octave exception [{identifier}]: {message}")]
    OctaveException { identifier: String, message: String },
}

enum SessionBackend {
    Real {
        child: Child,
        stdin: ChildStdin,
        reader: BufReader<ChildStdout>,
    },
    Mock {
        vars: HashMap<String, Value>,
    },
}

/// Active session connected to a GNU Octave subprocess or in-memory mock engine.
pub struct OctaveSession {
    backend: SessionBackend,
    config: OctaveConfig,
}

impl OctaveSession {
    /// Returns true if a usable `octave-cli` is detected on the host.
    pub fn is_available() -> bool {
        find_octave_binary().is_some()
    }

    /// Finds the path to the Octave binary, if available.
    pub fn find_binary() -> Option<PathBuf> {
        find_octave_binary()
    }

    /// Starts a persistent Octave session using default configuration.
    pub fn start() -> Result<Self, OctaveError> {
        Self::start_with_config(OctaveConfig::default())
    }

    /// Starts a persistent Octave session with explicit configuration.
    pub fn start_with_config(config: OctaveConfig) -> Result<Self, OctaveError> {
        let bin = find_octave_binary().ok_or(OctaveError::NotAvailable)?;
        Self::start_with_binary(&bin, config)
    }

    /// Starts a persistent session with explicit binary path and configuration.
    pub fn start_with_binary(binary: &Path, config: OctaveConfig) -> Result<Self, OctaveError> {
        let mut cmd = Command::new(binary);
        cmd.args([
            "--no-gui",
            "--norc",
            "--no-init-file",
            "--quiet",
            "--no-window-system",
            "--eval",
            "more off; while ~feof(stdin), __line__ = fgetl(stdin); if ~ischar(__line__), break; end; __line__ = strtrim(__line__); if isempty(__line__), continue; end; if ~strncmp(__line__, '{', 1), try eval(__line__); catch end; continue; end; if strcmp(__line__, '{\"cmd\":\"ping\"}'), printf('{\"ok\":true}\\n'); fflush(stdout); continue; end; if strcmp(__line__, '{\"cmd\":\"shutdown\"}'), break; end; try, __req__ = jsondecode(__line__); if isfield(__req__, 'cmd') && strcmp(__req__.cmd, 'eval'), __res__ = eval(__req__.src); if islogical(__res__), __dims__ = sprintf('%d,', size(__res__)); __data__ = sprintf('%d,', __res__(:)); if ~isempty(__data__), __data__ = strrep(strrep(__data__(1:end-1), '1', 'true'), '0', 'false'); end; printf('{\"ok\":true,\"result\":{\"type\":\"logical\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); fflush(stdout); elseif ischar(__res__), printf('{\"ok\":true,\"result\":{\"type\":\"char\",\"data\":\"%s\"}}\\n', strrep(strrep(__res__, '\\', '\\\\'), '\"', '\\\"')); fflush(stdout); elseif isnumeric(__res__), __dims__ = sprintf('%d,', size(__res__)); __data__ = sprintf('%.17g,', __res__(:)); if ~isempty(__data__), __data__ = __data__(1:end-1); end; printf('{\"ok\":true,\"result\":{\"type\":\"double\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); fflush(stdout); else, printf('{\"ok\":true}\\n'); fflush(stdout); end; elseif isfield(__req__, 'cmd') && strcmp(__req__.cmd, 'call'), __args__ = {}; if isfield(__req__, 'args'), for __i__ = 1:length(__req__.args), __a__ = __req__.args{__i__}; if strcmp(__a__.type, 'double'), __args__{__i__} = reshape(__a__.data, __a__.dims); elseif strcmp(__a__.type, 'char'), __args__{__i__} = __a__.data; elseif strcmp(__a__.type, 'logical'), __args__{__i__} = logical(reshape(__a__.data, __a__.dims)); end; end; end; __res__ = feval(__req__.fn_name, __args__{:}); if isnumeric(__res__), __dims__ = sprintf('%d,', size(__res__)); __data__ = sprintf('%.17g,', __res__(:)); if ~isempty(__data__), __data__ = __data__(1:end-1); end; printf('{\"ok\":true,\"result\":{\"type\":\"double\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); fflush(stdout); else, printf('{\"ok\":true}\\n'); fflush(stdout); end; else, printf('{\"ok\":false,\"error\":{\"identifier\":\"Octave:unknownCmd\",\"message\":\"Unknown command\"}}\\n'); fflush(stdout); end; catch __err__, printf('{\"ok\":false,\"error\":{\"identifier\":\"%s\",\"message\":\"%s\"}}\\n', __err__.identifier, strrep(strrep(__err__.message, '\\', '\\\\'), '\"', '\\\"')); fflush(stdout); end; end;",
        ]);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        let mut child = cmd
            .spawn()
            .map_err(|e| OctaveError::ProcessStart(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| OctaveError::ProcessIo("Failed to open child stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| OctaveError::ProcessIo("Failed to open child stdout".to_string()))?;
        let reader = BufReader::new(stdout);

        Ok(Self {
            backend: SessionBackend::Real {
                child,
                stdin,
                reader,
            },
            config,
        })
    }

    /// Starts an in-memory mock session for testing without requiring Octave on the runner.
    pub fn start_mock() -> Self {
        Self::start_mock_with_config(OctaveConfig::default())
    }

    /// Starts a mock session with custom configuration.
    pub fn start_mock_with_config(config: OctaveConfig) -> Self {
        Self {
            backend: SessionBackend::Mock {
                vars: HashMap::new(),
            },
            config,
        }
    }

    /// Sends a ping command to confirm the Octave process is responsive.
    pub fn ping(&mut self) -> Result<(), OctaveError> {
        match &mut self.backend {
            SessionBackend::Real { stdin, reader, .. } => {
                let req = OctaveRequest::Ping;
                let line = serde_json::to_string(&req)
                    .map_err(|e| OctaveError::Protocol(e.to_string()))?;
                writeln!(stdin, "{}", line).map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                stdin
                    .flush()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let mut resp_line = String::new();
                reader
                    .read_line(&mut resp_line)
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                Ok(())
            }
            SessionBackend::Mock { .. } => Ok(()),
        }
    }

    /// Stores a variable in the Octave workspace.
    pub fn put(&mut self, name: &str, val: &Value) -> Result<(), OctaveError> {
        let oct_val = OctaveValue::from_value(val)
            .map_err(|e| OctaveError::Protocol(format!("Value conversion error: {e}")))?;

        match &mut self.backend {
            SessionBackend::Real { stdin, .. } => {
                let literal = oct_val.to_octave_literal();
                let cmd = format!("{name} = {literal};\n");
                stdin
                    .write_all(cmd.as_bytes())
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                stdin
                    .flush()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                Ok(())
            }
            SessionBackend::Mock { vars } => {
                vars.insert(name.to_string(), val.clone());
                Ok(())
            }
        }
    }

    /// Retrieves a variable from the Octave workspace.
    pub fn get(&mut self, name: &str) -> Result<Value, OctaveError> {
        match &mut self.backend {
            SessionBackend::Real { .. } => {
                // In real Octave, evaluate the variable name to fetch its serialized form
                self.eval(name)
            }
            SessionBackend::Mock { vars } => {
                vars.get(name)
                    .cloned()
                    .ok_or_else(|| OctaveError::OctaveException {
                        identifier: "Octave:undefinedVariable".to_string(),
                        message: format!("Variable '{name}' is not defined in workspace"),
                    })
            }
        }
    }

    /// Evaluates a MATLAB/Octave expression and returns the parsed result.
    pub fn eval(&mut self, src: &str) -> Result<Value, OctaveError> {
        match &mut self.backend {
            SessionBackend::Real { stdin, reader, .. } => {
                let req = OctaveRequest::Eval {
                    src: src.to_string(),
                };
                let line = serde_json::to_string(&req)
                    .map_err(|e| OctaveError::Protocol(e.to_string()))?;
                writeln!(stdin, "{}", line).map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                stdin
                    .flush()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let mut resp_line = String::new();
                reader
                    .read_line(&mut resp_line)
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let resp: OctaveResponse = serde_json::from_str(&resp_line).map_err(|e| {
                    OctaveError::Protocol(format!("JSON parse error: {e} in {resp_line}"))
                })?;

                if resp.ok {
                    let oval = resp.result.ok_or_else(|| {
                        OctaveError::Protocol("Missing result in ok response".to_string())
                    })?;
                    oval.to_value()
                        .map_err(|e| OctaveError::Protocol(format!("Value conversion error: {e}")))
                } else {
                    let err = resp.error.unwrap_or(OctaveErrorInfo {
                        identifier: "Octave:genericError".to_string(),
                        message: "Unknown error from Octave worker".to_string(),
                    });
                    Err(OctaveError::OctaveException {
                        identifier: err.identifier,
                        message: err.message,
                    })
                }
            }
            SessionBackend::Mock { vars } => {
                let trimmed = src.trim();
                if let Some(pos) = trimmed.find('=') {
                    let var = trimmed[..pos].trim();
                    let expr = trimmed[pos + 1..].trim().trim_end_matches(';');
                    let val = evaluate_mock_expr(expr, vars)?;
                    vars.insert(var.to_string(), val.clone());
                    Ok(val)
                } else {
                    evaluate_mock_expr(trimmed.trim_end_matches(';'), vars)
                }
            }
        }
    }

    /// Invokes a named function in Octave with the provided arguments, enforcing safety allow-list policies.
    pub fn call(&mut self, fn_name: &str, args: &[Value]) -> Result<Value, OctaveError> {
        // Enforce safety allow-list policy first
        check_call_safety(fn_name, self.config.allow_unsafe)
            .map_err(OctaveError::SecurityViolation)?;

        match &mut self.backend {
            SessionBackend::Real { stdin, reader, .. } => {
                let mut oargs = Vec::with_capacity(args.len());
                for a in args {
                    let oa = OctaveValue::from_value(a).map_err(|e| {
                        OctaveError::Protocol(format!("Argument conversion error: {e}"))
                    })?;
                    oargs.push(oa);
                }

                let req = OctaveRequest::Call {
                    fn_name: fn_name.to_string(),
                    args: oargs,
                };
                let line = serde_json::to_string(&req)
                    .map_err(|e| OctaveError::Protocol(e.to_string()))?;
                writeln!(stdin, "{}", line).map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                stdin
                    .flush()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let mut resp_line = String::new();
                reader
                    .read_line(&mut resp_line)
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let resp: OctaveResponse = serde_json::from_str(&resp_line).map_err(|e| {
                    OctaveError::Protocol(format!("JSON parse error: {e} in {resp_line}"))
                })?;

                if resp.ok {
                    let oval = resp.result.ok_or_else(|| {
                        OctaveError::Protocol("Missing result in ok response".to_string())
                    })?;
                    oval.to_value()
                        .map_err(|e| OctaveError::Protocol(format!("Value conversion error: {e}")))
                } else {
                    let err = resp.error.unwrap_or(OctaveErrorInfo {
                        identifier: "Octave:genericError".to_string(),
                        message: "Unknown error from Octave worker".to_string(),
                    });
                    Err(OctaveError::OctaveException {
                        identifier: err.identifier,
                        message: err.message,
                    })
                }
            }
            SessionBackend::Mock { .. } => evaluate_mock_call(fn_name, args),
        }
    }

    /// Shuts down the Octave worker process cleanly.
    pub fn shutdown(&mut self) {
        if let SessionBackend::Real { child, stdin, .. } = &mut self.backend {
            let req = OctaveRequest::Shutdown;
            if let Ok(line) = serde_json::to_string(&req) {
                let _ = writeln!(stdin, "{}", line);
                let _ = stdin.flush();
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for OctaveSession {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn evaluate_mock_expr(expr: &str, vars: &HashMap<String, Value>) -> Result<Value, OctaveError> {
    if let Ok(num) = expr.parse::<f64>() {
        return Ok(Value::scalar(num));
    }
    if let Some(val) = vars.get(expr) {
        return Ok(val.clone());
    }
    if expr.starts_with('\'') && expr.ends_with('\'') && expr.len() >= 2 {
        let inner = &expr[1..expr.len() - 1];
        return Ok(Value::char_vector(inner));
    }
    if expr == "svd([1 2; 3 4])" {
        return Ok(make_vector(vec![5.464985704219043, 0.365966190626257]));
    }
    if expr == "det([1 2; 3 4])" {
        return Ok(Value::scalar(-2.0));
    }
    Ok(Value::scalar(0.0))
}

fn evaluate_mock_call(fn_name: &str, args: &[Value]) -> Result<Value, OctaveError> {
    match fn_name {
        "svd" => {
            if let Some(Value::Double(arr)) = args.first() {
                if arr.shape() == [2, 2] {
                    return Ok(make_vector(vec![5.464985704219043, 0.365966190626257]));
                }
            }
            Ok(make_vector(vec![1.0]))
        }
        "det" => {
            if let Some(Value::Double(arr)) = args.first() {
                if arr.shape() == [2, 2] {
                    let d = arr.as_slice();
                    let det_val = d[0] * d[3] - d[1] * d[2];
                    return Ok(Value::scalar(det_val));
                }
            }
            Ok(Value::scalar(1.0))
        }
        "sin" => {
            if let Some(val) = args.first() {
                if let Ok(x) = val.as_scalar_f64() {
                    return Ok(Value::scalar(x.sin()));
                }
            }
            Ok(Value::scalar(0.0))
        }
        "cos" => {
            if let Some(val) = args.first() {
                if let Ok(x) = val.as_scalar_f64() {
                    return Ok(Value::scalar(x.cos()));
                }
            }
            Ok(Value::scalar(1.0))
        }
        "fminsearch" => Ok(make_vector(vec![1.0, 2.0])),
        _ => Ok(Value::char_vector(&format!("mock_octave_{fn_name}"))),
    }
}

fn make_vector(data: Vec<f64>) -> Value {
    let len = data.len();
    Value::Double(DenseArray::from_vec_unchecked(vec![1, len], data))
}
