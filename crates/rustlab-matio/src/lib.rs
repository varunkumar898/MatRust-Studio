//! # `rustlab-matio`
//!
//! Clean-room Level 5 and v7 MAT-file reader and writer for RustLab.
//! Supports:
//! - 128-byte Level 5 headers with endian validation
//! - Standard 8-byte tags and 4-byte Small Data Elements (SDE)
//! - `miCOMPRESSED` zlib deflate decompression and compression
//! - Double, Complex Double, Logical, and Char arrays
//! - Roundtrip lossless serialization and deserialization

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod error;
pub mod reader;
pub mod types;
pub mod writer;

pub use error::MatError;
pub use reader::read_mat;
pub use types::{MatHeader, MI_COMPRESSED, MI_DOUBLE, MI_MATRIX};
pub use writer::write_mat;

use rustlab_math::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Saves a dictionary of MATLAB variables to a `.mat` file.
pub fn save_mat(
    path: impl AsRef<Path>,
    vars: &HashMap<String, Value>,
    compress: bool,
) -> Result<(), MatError> {
    let bytes = write_mat(vars, compress)?;
    fs::write(path, bytes)?;
    Ok(())
}

/// Loads a `.mat` file into a dictionary of MATLAB variables.
pub fn load_mat(path: impl AsRef<Path>) -> Result<HashMap<String, Value>, MatError> {
    let bytes = fs::read(path)?;
    read_mat(&bytes)
}
