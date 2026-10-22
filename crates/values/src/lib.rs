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

    /// Promote to `ComplexDenseMatrix` if real/boolean, or clone if already complex.
    pub fn to_complex_matrix(&self) -> Option<ComplexDenseMatrix> {
        match self {
            Value::Matrix(m) => Some(ComplexDenseMatrix::from_real(m)),
            Value::ComplexMatrix(c) => Some(c.clone()),
            Value::Boolean(b) => Some(ComplexDenseMatrix::scalar(Complex64::real(if *b {
                1.0
            } else {
                0.0
            }))),
            _ => None,
        }
    }

    /// Construct a Value from a ComplexDenseMatrix, simplifying to a real Value::Matrix
    /// if and only if all imaginary components are exactly 0.0 (exact cancellation).
    pub fn from_complex_or_simplified(c: ComplexDenseMatrix) -> Self {
        if c.is_all_real() {
            Value::Matrix(c.to_real_matrix())
        } else {
            Value::ComplexMatrix(c)
        }
    }

    /// If self is a ComplexMatrix whose imaginary parts are all exactly zero,
    /// demote it to a real Matrix. Otherwise return self.
    pub fn simplify_if_real(self) -> Self {
        match self {
            Value::ComplexMatrix(c) => Self::from_complex_or_simplified(c),
            other => other,
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
