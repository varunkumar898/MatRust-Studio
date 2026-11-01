//! JSON Lines typed IPC protocol and marshaling between Rust and MATLAB.

use rustlab_math::{DenseArray, StructArray, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request envelope sent from Rust to MATLAB worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum MatlabRequest {
    Eval {
        src: String,
    },
    Call {
        fn_name: String,
        args: Vec<MatlabValue>,
    },
}

/// Response envelope returned by the MATLAB worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatlabResponse {
    pub ok: bool,
    #[serde(default)]
    pub result: Option<MatlabValue>,
    #[serde(default)]
    pub error: Option<MatlabErrorInfo>,
}

/// Error details from MATLAB exception.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatlabErrorInfo {
    pub identifier: String,
    pub message: String,
}

/// Typed representation of MATLAB values over the wire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MatlabValue {
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
        data: Vec<MatlabValue>,
    },
    Struct {
        fields: HashMap<String, MatlabValue>,
    },
}

impl MatlabValue {
    /// Converts a `rustlab_math::Value` into a serializable `MatlabValue`.
    pub fn from_value(val: &Value) -> Result<Self, String> {
        match val {
            Value::Double(arr) => Ok(MatlabValue::Double {
                dims: arr.shape().to_vec(),
                data: arr.as_slice().to_vec(),
            }),
            Value::Single(arr) => Ok(MatlabValue::Double {
                dims: arr.shape().to_vec(),
                data: arr.as_slice().iter().map(|&x| x as f64).collect(),
            }),
            Value::Char(arr) => {
                let s: String = arr.as_slice().iter().collect();
                Ok(MatlabValue::Char { data: s })
            }
            Value::String(arr) => {
                let s = arr.as_slice().first().cloned().unwrap_or_default();
                Ok(MatlabValue::Char { data: s })
            }
            Value::Logical(arr) => Ok(MatlabValue::Logical {
                dims: arr.shape().to_vec(),
                data: arr.as_slice().to_vec(),
            }),
            Value::Cell(cell) => {
                let mut data = Vec::with_capacity(cell.numel());
                for item in cell.as_slice() {
                    data.push(Self::from_value(item)?);
                }
                Ok(MatlabValue::Cell {
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
                Ok(MatlabValue::Struct { fields })
            }
            _ => Err(format!(
                "Unsupported Value variant for MATLAB marshaling: {:?}",
                val
            )),
        }
    }

    /// Converts a `MatlabValue` into a `rustlab_math::Value`.
    pub fn to_value(&self) -> Result<Value, String> {
        match self {
            MatlabValue::Double { dims, data } => {
                let arr = DenseArray::from_vec(dims.clone(), data.clone())
                    .map_err(|e| format!("DenseArray shape error: {e}"))?;
                Ok(Value::Double(arr))
            }
            MatlabValue::Char { data } => Ok(Value::char_vector(data)),
            MatlabValue::Logical { dims, data } => {
                let arr = DenseArray::from_vec(dims.clone(), data.clone())
                    .map_err(|e| format!("DenseArray shape error: {e}"))?;
                Ok(Value::Logical(arr))
            }
            MatlabValue::Cell { dims, data } => {
                let mut items = Vec::with_capacity(data.len());
                for item in data {
                    items.push(item.to_value()?);
                }
                let cell = DenseArray::from_vec(dims.clone(), items)
                    .map_err(|e| format!("CellArray shape error: {e}"))?;
                Ok(Value::Cell(cell))
            }
            MatlabValue::Struct { fields } => {
                let mut map = HashMap::new();
                for (k, v) in fields {
                    map.insert(k.clone(), v.to_value()?);
                }
                Ok(Value::Struct(StructArray::scalar(map)))
            }
        }
    }
}
