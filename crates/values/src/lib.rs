pub mod matrix;

pub use matrix::DenseMatrix;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Matrix(DenseMatrix),
    String(String),
    Boolean(bool),
}

impl Value {
    pub fn scalar(n: f64) -> Self {
        Value::Matrix(DenseMatrix::scalar(n))
    }

    pub fn is_matrix(&self) -> bool {
        matches!(self, Value::Matrix(_))
    }

    pub fn as_matrix(&self) -> Option<&DenseMatrix> {
        match self {
            Value::Matrix(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_matrix_mut(&mut self) -> Option<&mut DenseMatrix> {
        match self {
            Value::Matrix(m) => Some(m),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Matrix(m) => {
                if m.is_scalar() {
                    "double"
                } else {
                    "matrix"
                }
            }
            Value::String(_) => "char",
            Value::Boolean(_) => "logical",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Matrix(m) => write!(f, "{}", m),
            Value::String(s) => write!(f, "'{}'", s),
            Value::Boolean(b) => write!(f, "{}", if *b { "1" } else { "0" }),
        }
    }
}
