//! MATLAB inter-process bridge for automated oracle conformance testing
//! and runtime fallback execution.

#![deny(clippy::unwrap_used)]

pub mod detect;
pub mod protocol;
pub mod session;

pub use detect::{find_matlab_binary, is_matlab_available};
pub use protocol::{MatlabErrorInfo, MatlabRequest, MatlabResponse, MatlabValue};
pub use session::{MatlabError, MatlabResult, MatlabSession};
