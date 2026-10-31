//! Constants, tags, and header definitions for Level 5 MAT-files.

// MAT-File Data Types
pub const MI_INT8: u32 = 1;
pub const MI_UINT8: u32 = 2;
pub const MI_INT16: u32 = 3;
pub const MI_UINT16: u32 = 4;
pub const MI_INT32: u32 = 5;
pub const MI_UINT32: u32 = 6;
pub const MI_SINGLE: u32 = 7;
pub const MI_DOUBLE: u32 = 9;
pub const MI_INT64: u32 = 12;
pub const MI_UINT64: u32 = 13;
pub const MI_MATRIX: u32 = 14;
pub const MI_COMPRESSED: u32 = 15;
pub const MI_UTF8: u32 = 16;
pub const MI_UTF16: u32 = 17;

// Array Classes (mxClassID)
pub const MX_CELL_CLASS: u8 = 1;
pub const MX_STRUCT_CLASS: u8 = 2;
pub const MX_LOGICAL_CLASS: u8 = 3;
pub const MX_CHAR_CLASS: u8 = 4;
pub const MX_SPARSE_CLASS: u8 = 5;
pub const MX_DOUBLE_CLASS: u8 = 6;
pub const MX_SINGLE_CLASS: u8 = 7;
pub const MX_INT8_CLASS: u8 = 8;
pub const MX_UINT8_CLASS: u8 = 9;
pub const MX_INT16_CLASS: u8 = 10;
pub const MX_UINT16_CLASS: u8 = 11;
pub const MX_INT32_CLASS: u8 = 12;
pub const MX_UINT32_CLASS: u8 = 13;
pub const MX_INT64_CLASS: u8 = 14;
pub const MX_UINT64_CLASS: u8 = 15;

// Array Flags bitmasks
pub const FLAG_COMPLEX: u8 = 0x08;
pub const FLAG_GLOBAL: u8 = 0x04;
pub const FLAG_LOGICAL: u8 = 0x02;

/// 128-Byte Level 5 MAT-file header.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MatHeader {
    pub text: [u8; 116],
    pub subsystem_offset: [u8; 8],
    pub version: u16,
    pub endian: [u8; 2],
}

impl Default for MatHeader {
    fn default() -> Self {
        let mut text = [b' '; 116];
        let desc = b"MATLAB 5.0 MAT-file, Platform: PCWIN64, Created by RustLab";
        text[..desc.len()].copy_from_slice(desc);

        Self {
            text,
            subsystem_offset: [0; 8],
            version: 0x0100, // Version 1.0 (Level 5)
            endian: *b"IM",  // Little-endian indicator
        }
    }
}
