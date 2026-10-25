//! Mathematical error types with canonical MATLAB identifiers.

use thiserror::Error;

/// Numerical and array operation errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MathError {
    // MATLAB documentation reference: "Arithmetic with Integer and Float" (R2024b / GNU Octave 9.2.0)
    // "Integers can only be combined with integers of the same class, or scalar doubles."
    #[error("Integers can only be combined with integers of the same class, or scalar doubles.")]
    IntegerClassMismatch { identifier: &'static str },

    // MATLAB reference: "In an assignment  A(I) = B, the number of elements in B and I must be the same."
    #[error("In an assignment  A(I) = B, the number of elements in B and I must be the same.")]
    SubscriptedAssignmentCountMismatch { identifier: &'static str },

    #[error("Array dimensions must match for binary array op or logical indexing.")]
    LogicalMaskSizeMismatch { identifier: &'static str },

    #[error("Arrays have incompatible sizes for this operation (identifier: {identifier})")]
    DimensionMismatch {
        identifier: &'static str,
        shape_a: Vec<usize>,
        shape_b: Vec<usize>,
    },

    #[error("Incorrect dimensions for matrix multiplication. Check that the number of columns in the first matrix ({cols_a}) matches the number of rows in the second matrix ({rows_b})")]
    InnerDimensionMismatch {
        identifier: &'static str,
        cols_a: usize,
        rows_b: usize,
    },

    #[error(
        "Index exceeds the number of array elements ({max_len}): requested 1-based index {index}"
    )]
    IndexOutOfBounds {
        identifier: &'static str,
        index: usize,
        max_len: usize,
    },

    #[error("Array indices must be positive integers or logical values: invalid index {index}")]
    InvalidSubscript {
        identifier: &'static str,
        index: isize,
    },

    #[error("Matrix must be square: shape is {rows}x{cols}")]
    NonSquareMatrix {
        identifier: &'static str,
        rows: usize,
        cols: usize,
    },

    #[error("To RESHAPE the number of elements must not change: from {original} to {requested}")]
    ReshapeElementCountMismatch {
        identifier: &'static str,
        original: usize,
        requested: usize,
    },

    #[error("Unsupported operation '{op}' for type '{type_name}'")]
    UnsupportedOperation {
        op: &'static str,
        type_name: &'static str,
    },
}

impl MathError {
    /// Dimension mismatch for elementwise operations (`MATLAB:dimagree`).
    pub fn dim_agree(shape_a: &[usize], shape_b: &[usize]) -> Self {
        Self::DimensionMismatch {
            identifier: "MATLAB:dimagree",
            shape_a: shape_a.to_vec(),
            shape_b: shape_b.to_vec(),
        }
    }

    /// Inner dimension mismatch for matrix multiplication (`MATLAB:innerdim`).
    pub fn inner_dim(cols_a: usize, rows_b: usize) -> Self {
        Self::InnerDimensionMismatch {
            identifier: "MATLAB:innerdim",
            cols_a,
            rows_b,
        }
    }

    /// Index out of bounds (`MATLAB:badsubscript`).
    pub fn index_out_of_bounds(index: usize, max_len: usize) -> Self {
        Self::IndexOutOfBounds {
            identifier: "MATLAB:badsubscript",
            index,
            max_len,
        }
    }

    /// Non-positive index (`MATLAB:subscriptNotPositiveInteger`).
    pub fn invalid_subscript(index: isize) -> Self {
        Self::InvalidSubscript {
            identifier: "MATLAB:subscriptNotPositiveInteger",
            index,
        }
    }

    /// Matrix must be square (`MATLAB:square`).
    pub fn non_square(rows: usize, cols: usize) -> Self {
        Self::NonSquareMatrix {
            identifier: "MATLAB:square",
            rows,
            cols,
        }
    }

    /// Integer class mismatch (`MATLAB:integerTypeMismatch`).
    pub fn integer_class_mismatch() -> Self {
        Self::IntegerClassMismatch {
            identifier: "MATLAB:integerTypeMismatch",
        }
    }

    /// Subscripted assignment count mismatch (`MATLAB:subsasgn:dimensionMismatch`).
    pub fn assignment_count_mismatch() -> Self {
        Self::SubscriptedAssignmentCountMismatch {
            identifier: "MATLAB:subsasgn:dimensionMismatch",
        }
    }

    /// Logical mask size mismatch (`MATLAB:dimagree`).
    pub fn mask_size_mismatch() -> Self {
        Self::LogicalMaskSizeMismatch {
            identifier: "MATLAB:dimagree",
        }
    }

    /// Reshape element count mismatch (`MATLAB:getReshapeDims:notSameNumel`).
    pub fn reshape_numel(original: usize, requested: usize) -> Self {
        Self::ReshapeElementCountMismatch {
            identifier: "MATLAB:getReshapeDims:notSameNumel",
            original,
            requested,
        }
    }
}
