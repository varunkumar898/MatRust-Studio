//! Error types for MAT-file I/O.

use thiserror::Error;

/// Errors produced during MAT-file reading and writing.
#[derive(Debug, Error)]
pub enum MatError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid MAT-file header: {0}")]
    InvalidHeader(String),

    #[error("Unsupported endianness or corrupted endian tag: {0:?}")]
    UnsupportedEndian([u8; 2]),

    #[error("Decompression failed: {0}")]
    Decompress(String),

    #[error("Unexpected end of file while reading {element}")]
    UnexpectedEof { element: &'static str },

    #[error("Unsupported MAT data type: {0}")]
    UnsupportedDataType(u32),

    #[error("Unsupported array class: {0}")]
    UnsupportedClass(u8),

    #[error("Data padding error: expected 8-byte alignment")]
    AlignmentError,

    #[error("Mathematical value conversion error: {0}")]
    Math(#[from] rustlab_math::MathError),
}
