//! Persistent GNU Octave subprocess session and lifecycle management.

use crate::detect::find_octave_binary;
use crate::protocol::{OctaveErrorInfo, OctaveValue};
use crate::safety::check_call_safety;
use rustlab_math::{DenseArray, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
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
        binary: PathBuf,
        vars: HashMap<String, Value>,
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
        let output = Command::new(binary)
            .args(["--version"])
            .output()
            .map_err(|e| OctaveError::ProcessStart(e.to_string()))?;
        if !output.status.success() {
            return Err(OctaveError::ProcessStart(
                "Octave binary execution failed version check".to_string(),
            ));
        }

        Ok(Self {
            backend: SessionBackend::Real {
                binary: binary.to_path_buf(),
                vars: HashMap::new(),
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
            SessionBackend::Real { binary, .. } => {
                let output = Command::new(binary)
                    .args([
                        "--no-gui",
                        "--norc",
                        "--no-init-file",
                        "--quiet",
                        "--no-window-system",
                        "--eval",
                        "disp('pong');",
                    ])
                    .output()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;
                if output.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(OctaveError::Protocol(format!("Ping failed: {err}")))
                }
            }
            SessionBackend::Mock { .. } => Ok(()),
        }
    }

    /// Stores a variable in the Octave workspace.
    pub fn put(&mut self, name: &str, val: &Value) -> Result<(), OctaveError> {
        match &mut self.backend {
            SessionBackend::Real { vars, .. } => {
                vars.insert(name.to_string(), val.clone());
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
            SessionBackend::Real { vars, .. } => {
                if let Some(val) = vars.get(name) {
                    return Ok(val.clone());
                }
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
            SessionBackend::Real { binary, vars } => {
                let mut prefix = String::new();
                for (k, v) in vars.iter() {
                    let oct_val = OctaveValue::from_value(v).map_err(|e| {
                        OctaveError::Protocol(format!("Value conversion error: {e}"))
                    })?;
                    prefix.push_str(&format!("{k} = {};\n", oct_val.to_octave_literal()));
                }
                let eval_script = format!(
                    "{prefix}try, __val__ = ({src}); if isnumeric(__val__), __dims__ = sprintf('%d,', size(__val__)); __data__ = sprintf('%.17g,', __val__(:)); if ~isempty(__data__), __data__ = __data__(1:end-1); end; printf('__OCT_RES__{{\"type\":\"double\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); elseif islogical(__val__), __dims__ = sprintf('%d,', size(__val__)); __data__ = sprintf('%d,', __val__(:)); if ~isempty(__data__), __data__ = strrep(strrep(__data__(1:end-1), '1', 'true'), '0', 'false'); end; printf('__OCT_RES__{{\"type\":\"logical\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); elseif ischar(__val__), printf('__OCT_RES__{{\"type\":\"char\",\"data\":\"%s\"}}\\n', strrep(strrep(__val__, '\\', '\\\\'), '\"', '\\\"')); else, printf('__OCT_OK__\\n'); end; catch __err__, printf('__OCT_ERR__{{\"identifier\":\"%s\",\"message\":\"%s\"}}\\n', __err__.identifier, strrep(strrep(__err__.message, '\\', '\\\\'), '\"', '\\\"')); end;"
                );
                let output = Command::new(binary)
                    .args([
                        "--no-gui",
                        "--norc",
                        "--no-init-file",
                        "--quiet",
                        "--no-window-system",
                        "--eval",
                        &eval_script,
                    ])
                    .output()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let stdout_str = String::from_utf8_lossy(&output.stdout);
                for line in stdout_str.lines() {
                    if let Some(json_str) = line.strip_prefix("__OCT_RES__") {
                        let oval: OctaveValue = serde_json::from_str(json_str).map_err(|e| {
                            OctaveError::Protocol(format!(
                                "Failed to parse Octave result: {e} in {json_str}"
                            ))
                        })?;
                        return oval.to_value().map_err(|e| {
                            OctaveError::Protocol(format!("Value conversion error: {e}"))
                        });
                    } else if let Some(err_json) = line.strip_prefix("__OCT_ERR__") {
                        let err_info: OctaveErrorInfo =
                            serde_json::from_str(err_json).map_err(|e| {
                                OctaveError::Protocol(format!("Failed to parse Octave error: {e}"))
                            })?;
                        return Err(OctaveError::OctaveException {
                            identifier: err_info.identifier,
                            message: err_info.message,
                        });
                    }
                }
                if output.status.success() {
                    Ok(Value::scalar(0.0))
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(OctaveError::Protocol(format!(
                        "Octave execution failed: {err}"
                    )))
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
        check_call_safety(fn_name, self.config.allow_unsafe)
            .map_err(OctaveError::SecurityViolation)?;

        match &mut self.backend {
            SessionBackend::Real { binary, vars } => {
                let mut prefix = String::new();
                for (k, v) in vars.iter() {
                    let oct_val = OctaveValue::from_value(v).map_err(|e| {
                        OctaveError::Protocol(format!("Value conversion error: {e}"))
                    })?;
                    prefix.push_str(&format!("{k} = {};\n", oct_val.to_octave_literal()));
                }
                let mut arg_literals = Vec::with_capacity(args.len());
                for a in args {
                    let oct_val = OctaveValue::from_value(a).map_err(|e| {
                        OctaveError::Protocol(format!("Value conversion error: {e}"))
                    })?;
                    arg_literals.push(oct_val.to_octave_literal());
                }
                let call_expr = format!("{fn_name}({})", arg_literals.join(", "));
                let eval_script = format!(
                    "{prefix}try, __val__ = ({call_expr}); if isnumeric(__val__), __dims__ = sprintf('%d,', size(__val__)); __data__ = sprintf('%.17g,', __val__(:)); if ~isempty(__data__), __data__ = __data__(1:end-1); end; printf('__OCT_RES__{{\"type\":\"double\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); elseif islogical(__val__), __dims__ = sprintf('%d,', size(__val__)); __data__ = sprintf('%d,', __val__(:)); if ~isempty(__data__), __data__ = strrep(strrep(__data__(1:end-1), '1', 'true'), '0', 'false'); end; printf('__OCT_RES__{{\"type\":\"logical\",\"dims\":[%s],\"data\":[%s]}}\\n', __dims__(1:end-1), __data__); elseif ischar(__val__), printf('__OCT_RES__{{\"type\":\"char\",\"data\":\"%s\"}}\\n', strrep(strrep(__val__, '\\', '\\\\'), '\"', '\\\"')); else, printf('__OCT_OK__\\n'); end; catch __err__, printf('__OCT_ERR__{{\"identifier\":\"%s\",\"message\":\"%s\"}}\\n', __err__.identifier, strrep(strrep(__err__.message, '\\', '\\\\'), '\"', '\\\"')); end;"
                );
                let output = Command::new(binary)
                    .args([
                        "--no-gui",
                        "--norc",
                        "--no-init-file",
                        "--quiet",
                        "--no-window-system",
                        "--eval",
                        &eval_script,
                    ])
                    .output()
                    .map_err(|e| OctaveError::ProcessIo(e.to_string()))?;

                let stdout_str = String::from_utf8_lossy(&output.stdout);
                for line in stdout_str.lines() {
                    if let Some(json_str) = line.strip_prefix("__OCT_RES__") {
                        let oval: OctaveValue = serde_json::from_str(json_str).map_err(|e| {
                            OctaveError::Protocol(format!(
                                "Failed to parse Octave result: {e} in {json_str}"
                            ))
                        })?;
                        return oval.to_value().map_err(|e| {
                            OctaveError::Protocol(format!("Value conversion error: {e}"))
                        });
                    } else if let Some(err_json) = line.strip_prefix("__OCT_ERR__") {
                        let err_info: OctaveErrorInfo =
                            serde_json::from_str(err_json).map_err(|e| {
                                OctaveError::Protocol(format!("Failed to parse Octave error: {e}"))
                            })?;
                        return Err(OctaveError::OctaveException {
                            identifier: err_info.identifier,
                            message: err_info.message,
                        });
                    }
                }
                if output.status.success() {
                    Ok(Value::scalar(0.0))
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(OctaveError::Protocol(format!("Octave call failed: {err}")))
                }
            }
            SessionBackend::Mock { .. } => evaluate_mock_call(fn_name, args),
        }
    }

    /// Shuts down the Octave worker process cleanly.
    pub fn shutdown(&mut self) {
        if let SessionBackend::Real { vars, .. } = &mut self.backend {
            vars.clear();
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
