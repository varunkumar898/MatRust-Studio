//! # `rustlab-math`
//!
//! Numerical computing core for rustlab:
//! - Pure column-major multidimensional tensor storage (`DenseArray<T>`)
//! - Zero-copy implicit expansion (broadcasting) coordinate iterators
//! - Vectorized 2D matrix multiplication (`matmul_2d`)
//! - Unified MATLAB runtime array representation (`Value`)
//! - 1-based indexing, slicing, dynamic array growth, and colon range generation

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod array;
pub mod broadcast;
pub mod error;
pub mod ops;
pub mod shape;
pub mod sparse;
pub mod value;

pub use array::DenseArray;
pub use broadcast::{broadcast_shapes, compute_broadcast_strides, BroadcastPairIterator};
pub use error::MathError;
pub use num_complex::Complex64;
pub use ops::{colon_range, elementwise_binary, elementwise_unary, matmul_2d};
pub use shape::{
    compute_column_major_strides, coords_to_offset, normalize_shape, numel, offset_to_coords,
};
pub use sparse::CscMatrix;
pub use value::{FunctionHandle, StructArray, Value};
