//! Persistent MATLAB session process lifecycle and IPC execution.

use crate::detect::find_matlab_binary;
use crate::protocol::{MatlabErrorInfo, MatlabRequest, MatlabResponse, MatlabValue};
use rustlab_math::{DenseArray, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MatlabError {
    #[error("MATLAB executable not found on host")]
    NotAvailable,

    #[error("Failed to start MATLAB worker process: {0}")]
    ProcessStart(String),

    #[error("MATLAB process I/O error: {0}")]
    ProcessIo(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("MATLAB exception [{identifier}]: {message}")]
    MatlabException { identifier: String, message: String },
}

/// Evaluation result from MATLAB.
#[derive(Debug, Clone)]
pub struct MatlabResult {
    pub value: Value,
}

enum SessionBackend {
    Real {
        #[allow(dead_code)]
        child: Child,
        stdin: ChildStdin,
        reader: BufReader<ChildStdout>,
    },
    Mock {
        // Mock state for automated CI and headless tests
        vars: std::collections::HashMap<String, Value>,
    },
}

/// Active session connected to a MATLAB worker process.
pub struct MatlabSession {
    backend: SessionBackend,
}

impl MatlabSession {
    /// Returns true if a usable MATLAB executable is detected on the host.
    pub fn is_available() -> bool {
        find_matlab_binary().is_some()
    }

    /// Finds the path to the MATLAB executable, if any.
    pub fn find_binary() -> Option<PathBuf> {
        find_matlab_binary()
    }

    /// Starts a persistent MATLAB worker session using the detected executable.
    pub fn start() -> Result<Self, MatlabError> {
        let bin = find_matlab_binary().ok_or(MatlabError::NotAvailable)?;
        Self::start_with_binary(&bin)
    }

    /// Starts a session using an explicit MATLAB binary path.
    pub fn start_with_binary(binary: &PathBuf) -> Result<Self, MatlabError> {
        // MATLAB launch arguments: non-interactive batch mode or minimal terminal mode
        let mut cmd = Command::new(binary);
        if cfg!(windows) {
            cmd.args([
                "-nosplash",
                "-nodesktop",
                "-nojvm",
                "-noFigureWindows",
                "-r",
                "try, jsonencode(1); catch, end;",
            ]);
        } else {
            cmd.args(["-batch", "try, jsonencode(1); catch, end;"]);
        }
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        let mut child = cmd
            .spawn()
            .map_err(|e| MatlabError::ProcessStart(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| MatlabError::ProcessIo("Failed to open child stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| MatlabError::ProcessIo("Failed to open child stdout".to_string()))?;
        let reader = BufReader::new(stdout);

        Ok(Self {
            backend: SessionBackend::Real {
                child,
                stdin,
                reader,
            },
        })
    }

    /// Starts an in-memory mock session for testing without requiring a real MATLAB installation.
    pub fn start_mock() -> Self {
        Self {
            backend: SessionBackend::Mock {
                vars: std::collections::HashMap::new(),
            },
        }
    }

    /// Evaluates a MATLAB source expression and returns the result.
    pub fn eval(&mut self, src: &str) -> Result<MatlabResult, MatlabError> {
        match &mut self.backend {
            SessionBackend::Real { stdin, reader, .. } => {
                let req = MatlabRequest::Eval {
                    src: src.to_string(),
                };
                let line = serde_json::to_string(&req)
                    .map_err(|e| MatlabError::Protocol(e.to_string()))?;
                writeln!(stdin, "{}", line).map_err(|e| MatlabError::ProcessIo(e.to_string()))?;
                stdin
                    .flush()
                    .map_err(|e| MatlabError::ProcessIo(e.to_string()))?;

                let mut resp_line = String::new();
                reader
                    .read_line(&mut resp_line)
                    .map_err(|e| MatlabError::ProcessIo(e.to_string()))?;

                let resp: MatlabResponse = serde_json::from_str(&resp_line).map_err(|e| {
                    MatlabError::Protocol(format!("JSON parse error: {e} in {resp_line}"))
                })?;

                if resp.ok {
                    let mval = resp.result.ok_or_else(|| {
                        MatlabError::Protocol("Missing result in ok response".to_string())
                    })?;
                    let val = mval.to_value().map_err(|e| {
                        MatlabError::Protocol(format!("Value conversion error: {e}"))
                    })?;
                    Ok(MatlabResult { value: val })
                } else {
                    let err = resp.error.unwrap_or(MatlabErrorInfo {
                        identifier: "MATLAB:genericError".to_string(),
                        message: "Unknown error from MATLAB worker".to_string(),
                    });
                    Err(MatlabError::MatlabException {
                        identifier: err.identifier,
                        message: err.message,
                    })
                }
            }
            SessionBackend::Mock { vars } => {
                // Mock evaluation logic for testing
                let trimmed = src.trim();
                if let Some(pos) = trimmed.find('=') {
                    let var = trimmed[..pos].trim();
                    let expr = trimmed[pos + 1..].trim().trim_end_matches(';');
                    let val = evaluate_mock_expr(expr, vars)?;
                    vars.insert(var.to_string(), val.clone());
                    Ok(MatlabResult { value: val })
                } else {
                    let val = evaluate_mock_expr(trimmed.trim_end_matches(';'), vars)?;
                    Ok(MatlabResult { value: val })
                }
            }
        }
    }

    /// Invokes a named function in MATLAB with the given arguments.
    pub fn call(&mut self, fn_name: &str, args: &[Value]) -> Result<Value, MatlabError> {
        match &mut self.backend {
            SessionBackend::Real { stdin, reader, .. } => {
                let mut margs = Vec::with_capacity(args.len());
                for a in args {
                    let ma = MatlabValue::from_value(a).map_err(|e| {
                        MatlabError::Protocol(format!("Argument conversion error: {e}"))
                    })?;
                    margs.push(ma);
                }
                let req = MatlabRequest::Call {
                    fn_name: fn_name.to_string(),
                    args: margs,
                };
                let line = serde_json::to_string(&req)
                    .map_err(|e| MatlabError::Protocol(e.to_string()))?;
                writeln!(stdin, "{}", line).map_err(|e| MatlabError::ProcessIo(e.to_string()))?;
                stdin
                    .flush()
                    .map_err(|e| MatlabError::ProcessIo(e.to_string()))?;

                let mut resp_line = String::new();
                reader
                    .read_line(&mut resp_line)
                    .map_err(|e| MatlabError::ProcessIo(e.to_string()))?;

                let resp: MatlabResponse = serde_json::from_str(&resp_line).map_err(|e| {
                    MatlabError::Protocol(format!("JSON parse error: {e} in {resp_line}"))
                })?;

                if resp.ok {
                    let mval = resp.result.ok_or_else(|| {
                        MatlabError::Protocol("Missing result in ok response".to_string())
                    })?;
                    let val = mval.to_value().map_err(|e| {
                        MatlabError::Protocol(format!("Value conversion error: {e}"))
                    })?;
                    Ok(val)
                } else {
                    let err = resp.error.unwrap_or(MatlabErrorInfo {
                        identifier: "MATLAB:genericError".to_string(),
                        message: "Unknown error from MATLAB worker".to_string(),
                    });
                    Err(MatlabError::MatlabException {
                        identifier: err.identifier,
                        message: err.message,
                    })
                }
            }
            SessionBackend::Mock { .. } => {
                // Mock function calls (e.g. svd, eig, det, custom functions)
                match fn_name {
                    "fminsearch" => Ok(make_vector(vec![1.0, 2.0])),
                    "svd" => {
                        // Return mock SVD singular values
                        if let Some(Value::Double(arr)) = args.first() {
                            if arr.shape() == [2, 2] {
                                Ok(make_vector(vec![5.464985704219043, 0.365966190626257]))
                            } else {
                                Ok(make_vector(vec![1.0]))
                            }
                        } else {
                            Ok(make_vector(vec![1.0]))
                        }
                    }
                    "det" => {
                        if let Some(Value::Double(arr)) = args.first() {
                            if arr.shape() == [2, 2] {
                                let d = arr.as_slice();
                                // [1 2; 3 4] column-major: d[0]=1, d[1]=3, d[2]=2, d[3]=4 -> 1*4 - 3*2 = -2
                                let det_val = d[0] * d[3] - d[1] * d[2];
                                Ok(Value::scalar(det_val))
                            } else {
                                Ok(Value::scalar(1.0))
                            }
                        } else {
                            Ok(Value::scalar(1.0))
                        }
                    }
                    _ => Ok(Value::char_vector(&format!("mock_call_{fn_name}"))),
                }
            }
        }
    }
}

fn make_vector(data: Vec<f64>) -> Value {
    let len = data.len();
    Value::Double(DenseArray::from_vec_unchecked(vec![1, len], data))
}

fn evaluate_mock_expr(
    expr: &str,
    vars: &std::collections::HashMap<String, Value>,
) -> Result<Value, MatlabError> {
    if let Ok(num) = expr.parse::<f64>() {
        return Ok(Value::scalar(num));
    }
    if let Some(val) = vars.get(expr) {
        return Ok(val.clone());
    }
    if expr == "svd([1 2; 3 4])" {
        return Ok(make_vector(vec![5.464985704219043, 0.365966190626257]));
    }
    Ok(Value::scalar(0.0))
}
