//! Clean-room GNU Octave subprocess bridge for automated conformance and runtime fallback.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod detect;
pub mod protocol;
pub mod safety;
pub mod session;

pub use detect::{find_octave_binary, is_octave_available};
pub use protocol::{OctaveErrorInfo, OctaveRequest, OctaveResponse, OctaveValue};
pub use safety::{check_call_safety, is_recognized_safe, BLOCKED_FUNCTIONS, SAFE_BUILTINS};
pub use session::{OctaveConfig, OctaveError, OctaveSession};
