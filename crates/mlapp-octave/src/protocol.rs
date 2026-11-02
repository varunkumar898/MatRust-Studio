//! IPC envelopes and bidirectional marshaling between Rust and GNU Octave.

use rustlab_math::{DenseArray, StructArray, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IPC request envelope sent to Octave subprocess or mock engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum OctaveRequest {
    Ping,
    Eval {
        src: String,
    },
    Put {
        name: String,
        val: OctaveValue,
    },
    Get {
        name: String,
    },
    Call {
        fn_name: String,
        args: Vec<OctaveValue>,
    },
    Shutdown,
}

/// IPC response envelope returned from Octave.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OctaveResponse {
    pub ok: bool,
    #[serde(default)]
    pub result: Option<OctaveValue>,
    #[serde(default)]
    pub error: Option<OctaveErrorInfo>,
}

/// Detailed error info returned from Octave.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OctaveErrorInfo {
    pub identifier: String,
    pub message: String,
}

/// Strongly typed value representation over IPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OctaveValue {
    Double {
        dims: Vec<usize>,
        data: Vec<f64>,
    },
    Char {
        data: String,
    },
    Logical {
        dims: Vec<usize>,
        data: Vec<bool>,
    },
    Cell {
        dims: Vec<usize>,
        data: Vec<OctaveValue>,
    },
    Struct {
        fields: HashMap<String, OctaveValue>,
    },
}

impl OctaveValue {
    /// Creates a scalar Double value.
    pub fn scalar(val: f64) -> Self {
        OctaveValue::Double {
            dims: vec![1, 1],
            data: vec![val],
        }
    }

    /// Converts a `rustlab_math::Value` into an `OctaveValue`.
    pub fn from_value(val: &Value) -> Result<Self, String> {
        match val {
            Value::Double(arr) => Ok(OctaveValue::Double {
                dims: arr.shape().to_vec(),
                data: arr.as_slice().to_vec(),
            }),
            Value::Single(arr) => Ok(OctaveValue::Double {
                dims: arr.shape().to_vec(),
                data: arr.as_slice().iter().map(|&x| x as f64).collect(),
            }),
            Value::Char(arr) => {
                let s: String = arr.as_slice().iter().collect();
                Ok(OctaveValue::Char { data: s })
            }
            Value::String(arr) => {
                let s = arr.as_slice().first().cloned().unwrap_or_default();
                Ok(OctaveValue::Char { data: s })
            }
            Value::Logical(arr) => Ok(OctaveValue::Logical {
                dims: arr.shape().to_vec(),
                data: arr.as_slice().to_vec(),
            }),
            Value::Cell(cell) => {
                let mut data = Vec::with_capacity(cell.numel());
                for item in cell.as_slice() {
                    data.push(Self::from_value(item)?);
                }
                Ok(OctaveValue::Cell {
                    dims: cell.shape().to_vec(),
                    data,
                })
            }
            Value::Struct(st) => {
                let mut fields = HashMap::new();
                for field_name in &st.fields {
                    if let Some(v) = st.get_field(field_name) {
                        fields.insert(field_name.clone(), Self::from_value(v)?);
                    }
                }
                Ok(OctaveValue::Struct { fields })
            }
            other => Err(format!(
                "Cannot marshal value type '{}' to Octave over IPC",
                other.type_name()
            )),
        }
    }

    /// Converts an `OctaveValue` back into a `rustlab_math::Value`.
    pub fn to_value(&self) -> Result<Value, String> {
        match self {
            OctaveValue::Double { dims, data } => {
                let arr = DenseArray::from_vec(dims.clone(), data.clone())
                    .map_err(|e| format!("Invalid Double array shape: {e}"))?;
                Ok(Value::Double(arr))
            }
            OctaveValue::Char { data } => Ok(Value::char_vector(data)),
            OctaveValue::Logical { dims, data } => {
                let arr = DenseArray::from_vec(dims.clone(), data.clone())
                    .map_err(|e| format!("Invalid Logical array shape: {e}"))?;
                Ok(Value::Logical(arr))
            }
            OctaveValue::Cell { dims, data } => {
                let mut vals = Vec::with_capacity(data.len());
                for d in data {
                    vals.push(d.to_value()?);
                }
                let arr = DenseArray::from_vec(dims.clone(), vals)
                    .map_err(|e| format!("Invalid Cell array shape: {e}"))?;
                Ok(Value::Cell(arr))
            }
            OctaveValue::Struct { fields } => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in fields {
                    map.insert(k.clone(), v.to_value()?);
                }
                Ok(Value::Struct(StructArray::scalar(map)))
            }
        }
    }

    /// Generates valid Octave syntax representing this value for CLI stdin injection.
    pub fn to_octave_literal(&self) -> String {
        match self {
            OctaveValue::Double { dims, data } => {
                if dims == &[1, 1] || dims.is_empty() {
                    return data.first().copied().unwrap_or(0.0).to_string();
                }
                let nums: Vec<String> = data.iter().map(|x| x.to_string()).collect();
                format!(
                    "reshape([{}], [{}])",
                    nums.join(", "),
                    dims.iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            OctaveValue::Char { data } => {
                let escaped = data.replace('\'', "''");
                format!("'{escaped}'")
            }
            OctaveValue::Logical { dims, data } => {
                let nums: Vec<String> = data
                    .iter()
                    .map(|b| if *b { "1" } else { "0" }.to_string())
                    .collect();
                format!(
                    "logical(reshape([{}], [{}]))",
                    nums.join(", "),
                    dims.iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            OctaveValue::Cell { dims, data } => {
                let items: Vec<String> = data.iter().map(|v| v.to_octave_literal()).collect();
                format!(
                    "reshape({{{}}}, [{}])",
                    items.join(", "),
                    dims.iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            OctaveValue::Struct { fields } => {
                let mut args = Vec::new();
                for (k, v) in fields {
                    args.push(format!("'{}'", k.replace('\'', "''")));
                    args.push(v.to_octave_literal());
                }
                format!("struct({})", args.join(", "))
            }
        }
    }
}
