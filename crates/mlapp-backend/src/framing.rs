//! Length-prefixed framing for the backend IPC protocol.
//!
//! Every message on the wire is:
//!   [4 bytes: u32 little-endian payload length][payload bytes]
//! The payload is a UTF-8 JSON envelope (see `crate::protocol`).
//!
//! The 4-byte length prefix makes the stream self-synchronizing: a reader can
//! detect and skip malformed frames without panicking, which is what the
//! protocol fuzz test exercises.

use std::io::{self, Read, Write};

/// Number of bytes in the length prefix.
pub const LEN_PREFIX_LEN: usize = 4;

/// Errors produced by the framing layer.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("I/O error while framing: {0}")]
    Io(#[from] io::Error),
    /// The peer closed the stream before a complete frame arrived.
    #[error("unexpected end of stream while reading frame")]
    UnexpectedEof,
    /// The length prefix advertised a payload larger than the allowed maximum.
    #[error("frame payload length {0} exceeds maximum {1}")]
    PayloadTooLarge(u32, u32),
}

/// Maximum allowed frame payload size (64 MiB). Matches the K2 64 MB payload test.
pub const MAX_PAYLOAD: u32 = 64 * 1024 * 1024;

/// Write a length-prefixed frame to `writer`.
pub fn write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), FrameError> {
    if payload.len() > u32::MAX as usize {
        return Err(FrameError::PayloadTooLarge(
            payload.len() as u32,
            u32::MAX,
        ));
    }
    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

/// Read exactly `n` bytes from `reader` into `buf`.
fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<(), FrameError> {
    if buf.is_empty() {
        return Ok(());
    }
    let mut read = 0;
    while read < buf.len() {
        let n = reader.read(&mut buf[read..]).map_err(FrameError::Io)?;
        if n == 0 {
            return Err(FrameError::UnexpectedEof);
        }
        read += n;
    }
    Ok(())
}

/// Read one length-prefixed frame from `reader`.
/// Returns `Err(UnexpectedEof)` only on a clean stream end before any bytes;
/// a mid-frame end is also `UnexpectedEof` (caller decides whether to retry).
pub fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, FrameError> {
    let mut len_buf = [0u8; LEN_PREFIX_LEN];
    match read_exact(reader, &mut len_buf) {
        Ok(()) => {}
        Err(FrameError::UnexpectedEof) => return Err(FrameError::UnexpectedEof),
        Err(e) => return Err(e),
    }
    let len = u32::from_le_bytes(len_buf);
    if len > MAX_PAYLOAD {
        return Err(FrameError::PayloadTooLarge(len, MAX_PAYLOAD));
    }
    let mut payload = vec![0u8; len as usize];
    read_exact(reader, &mut payload)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_small_payload() {
        let payload = b"{\"ok\":true}";
        let mut buf = Vec::new();
        write_frame(&mut buf, payload).unwrap();
        let mut reader: &[u8] = &buf;
        let back = read_frame(&mut reader).unwrap();
        assert_eq!(back, payload);
        assert!(reader.is_empty());
    }

    #[test]
    fn empty_payload() {
        let mut buf = Vec::new();
        write_frame(&mut buf, b"").unwrap();
        let mut reader: &[u8] = &buf;
        let back = read_frame(&mut reader).unwrap();
        assert_eq!(back, b"");
    }

    #[test]
    fn oversized_length_is_rejected() {
        let mut buf = Vec::new();
        // Length prefix claims 65 MiB (> 64 MiB max).
        buf.extend_from_slice(&((65 * 1024 * 1024) as u32).to_le_bytes());
        let mut reader: &[u8] = &buf;
        let err = read_frame(&mut reader).unwrap_err();
        assert!(matches!(err, FrameError::PayloadTooLarge(_, _)));
    }

    #[test]
    fn truncated_payload_is_eof() {
        let mut buf = Vec::new();
        write_frame(&mut buf, b"hello").unwrap();
        // Cut off the last byte of the payload.
        buf.pop();
        let mut reader: &[u8] = &buf;
        let err = read_frame(&mut reader).unwrap_err();
        assert!(matches!(err, FrameError::UnexpectedEof));
    }

    #[test]
    fn missing_length_prefix_is_eof() {
        let mut reader: &[u8] = b"";
        let err = read_frame(&mut reader).unwrap_err();
        assert!(matches!(err, FrameError::UnexpectedEof));
    }

    #[test]
    fn multiple_frames_in_sequence() {
        let mut buf = Vec::new();
        write_frame(&mut buf, b"one").unwrap();
        write_frame(&mut buf, b"two").unwrap();
        write_frame(&mut buf, b"three").unwrap();
        let mut reader: &[u8] = &buf;
        assert_eq!(read_frame(&mut reader).unwrap(), b"one");
        assert_eq!(read_frame(&mut reader).unwrap(), b"two");
        assert_eq!(read_frame(&mut reader).unwrap(), b"three");
    }
}
