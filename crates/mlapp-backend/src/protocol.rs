//! IPC envelopes for the length-prefixed JSON protocol between the host and a
//! backend worker process. Every message is a single JSON object framed with a
//! 4-byte little-endian u32 length prefix (see `crate::framing`).
//!
//! Data (arrays, values) is marshalled through `.mat` files in a per-session
//! temp directory using `rustlab-matio`. The envelope therefore carries file
//! paths rather than inlining large payloads, keeping frames small and the
//! round-trip bit-exact for doubles (NaN, Inf, -0, subnormals, int64 extremes).

use rustlab_math::Value;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A request sent from the host to the backend worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Request {
    /// Probe that the worker is alive.
    Ping,
    /// Evaluate source code in the backend's workspace.
    Eval {
        /// Source code to evaluate.
        code: String,
        /// Variables to write to `.mat` files before evaluating (name -> path).
        #[serde(default)]
        put: Vec<PutVar>,
        /// Number of expected outputs.
        #[serde(default)]
        nargout: usize,
    },
    /// Call a function by name with arguments.
    Call {
        /// Function name.
        fn_name: String,
        /// Arguments to write to `.mat` files before calling (name -> path).
        #[serde(default)]
        args: Vec<PutVar>,
        /// Number of expected outputs.
        #[serde(default)]
        nargout: usize,
    },
    /// Interrupt the currently running computation (kill switch).
    Interrupt,
    /// Shut down the worker process.
    Shutdown,
}

/// A variable to be written to a `.mat` file before an eval/call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutVar {
    /// Variable name in the backend workspace.
    pub name: String,
    /// Path to a `.mat` file containing the variable value.
    pub path: PathBuf,
}

/// A response returned from the backend worker to the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    /// Echo of the request id (u64 serialized as a string for JSON safety).
    pub id: u64,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Path to a `.mat` file holding the result(s), when `ok` is true.
    #[serde(default)]
    pub get: Option<PathBuf>,
    /// Error information when `ok` is false.
    #[serde(default)]
    pub error: Option<BackendErrorInfo>,
}

/// Structured error info returned by the worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendErrorInfo {
    /// Error id, e.g. `RUSTLAB:WorkerCrashed`, `Octave:SyntaxError`.
    pub id: String,
    /// Human-readable message.
    pub message: String,
}

/// Monotonically increasing request id generator.
#[derive(Debug, Default)]
pub struct IdGenerator {
    next: u64,
}

impl IdGenerator {
    /// Returns the next request id and advances the counter.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let id = self.next;
        self.next = id.wrapping_add(1);
        id
    }
}

/// JSON-serializable mirror of a subset of `rustlab_math::Value` used for
/// small in-frame values (scalars, short strings) when `.mat` files are not
/// desired. Covers Double, Logical, and Char for test convenience.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ValueEnvelope {
    Double { dims: Vec<usize>, data: Vec<f64> },
    Logical { dims: Vec<usize>, data: Vec<bool> },
    Char { data: String },
}

impl ValueEnvelope {
    pub fn scalar_double(val: f64) -> Self {
        ValueEnvelope::Double {
            dims: vec![1, 1],
            data: vec![val],
        }
    }
}

/// Convert a `ValueEnvelope` into a `rustlab_math::Value`.
pub fn envelope_to_value(env: &ValueEnvelope) -> Result<Value, String> {
    match env {
        ValueEnvelope::Double { dims, data } => {
            let arr = rustlab_math::DenseArray::from_vec(dims.clone(), data.clone())
                .map_err(|e| format!("invalid double array shape: {e}"))?;
            Ok(Value::Double(arr))
        }
        ValueEnvelope::Logical { dims, data } => {
            let arr = rustlab_math::DenseArray::from_vec(dims.clone(), data.clone())
                .map_err(|e| format!("invalid logical array shape: {e}"))?;
            Ok(Value::Logical(arr))
        }
        ValueEnvelope::Char { data } => Ok(Value::char_vector(data)),
    }
}

/// Convert a `rustlab_math::Value` into a `ValueEnvelope`.
/// Only Double, Logical, and Char are supported; other types return an error.
pub fn value_to_envelope(val: &Value) -> Result<ValueEnvelope, String> {
    match val {
        Value::Double(a) => Ok(ValueEnvelope::Double {
            dims: a.shape().to_vec(),
            data: a.as_slice().to_vec(),
        }),
        Value::Logical(a) => Ok(ValueEnvelope::Logical {
            dims: a.shape().to_vec(),
            data: a.as_slice().to_vec(),
        }),
        Value::Char(a) => {
            let s: String = a.as_slice().iter().collect();
            Ok(ValueEnvelope::Char { data: s })
        }
        other => Err(format!(
            "value type '{}' cannot be encoded as a protocol envelope",
            other.type_name()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_round_trip_double() {
        let env = ValueEnvelope::scalar_double(3.5);
        let json = serde_json::to_string(&env).unwrap();
        let back: ValueEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(env, back);
    }

    #[test]
    fn request_serialization_shapes() {
        let req = Request::Eval {
            code: "1+1".to_string(),
            put: vec![],
            nargout: 1,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn response_serialization_shapes() {
        let resp = Response {
            id: 7,
            ok: true,
            get: None,
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }
}
