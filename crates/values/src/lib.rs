pub mod complex;
pub mod matrix;

pub use complex::{Complex64, ComplexDenseMatrix};
pub use matrix::DenseMatrix;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Matrix(DenseMatrix),
    ComplexMatrix(ComplexDenseMatrix),
    String(String),
    Boolean(bool),
}

impl Value {
    pub fn scalar(n: f64) -> Self {
        Value::Matrix(DenseMatrix::scalar(n))
    }

    pub fn complex_scalar(re: f64, im: f64) -> Self {
        Value::ComplexMatrix(ComplexDenseMatrix::scalar(Complex64::new(re, im)))
    }

    pub fn is_matrix(&self) -> bool {
        matches!(self, Value::Matrix(_) | Value::ComplexMatrix(_))
    }

    pub fn is_complex(&self) -> bool {
        matches!(self, Value::ComplexMatrix(_))
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

    pub fn as_complex_matrix(&self) -> Option<&ComplexDenseMatrix> {
        match self {
            Value::ComplexMatrix(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_complex_matrix_mut(&mut self) -> Option<&mut ComplexDenseMatrix> {
        match self {
            Value::ComplexMatrix(m) => Some(m),
            _ => None,
        }
    }

    /// Promote to `ComplexDenseMatrix` if real, or clone if already complex.
    pub fn to_complex_matrix(&self) -> Option<ComplexDenseMatrix> {
        match self {
            Value::Matrix(m) => Some(ComplexDenseMatrix::from_real(m)),
            Value::ComplexMatrix(c) => Some(c.clone()),
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
            Value::ComplexMatrix(m) => {
                if m.is_scalar() {
                    "complex double"
                } else {
                    "complex matrix"
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
            Value::ComplexMatrix(m) => write!(f, "{}", m),
            Value::String(s) => write!(f, "'{}'", s),
            Value::Boolean(b) => write!(f, "{}", if *b { "1" } else { "0" }),
        }
    }
}
