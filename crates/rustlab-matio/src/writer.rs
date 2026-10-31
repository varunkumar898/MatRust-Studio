//! Writer for Level 5 and v7 MAT-files.

use crate::error::MatError;
use crate::types::*;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rustlab_math::Value;
use std::collections::HashMap;
use std::io::Write;

/// Encodes a dictionary of variables into a MAT-file byte vector.
pub fn write_mat(vars: &HashMap<String, Value>, compress: bool) -> Result<Vec<u8>, MatError> {
    let mut out = Vec::new();

    // 1. Write 128-byte File Header
    let header = MatHeader::default();
    out.extend_from_slice(&header.text);
    out.extend_from_slice(&header.subsystem_offset);
    out.extend_from_slice(&header.version.to_le_bytes());
    out.extend_from_slice(&header.endian);

    // 2. Write each variable as a tagged element
    for (name, val) in vars {
        let matrix_bytes = encode_matrix(name, val)?;

        if compress {
            // Compress with zlib
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&matrix_bytes)?;
            let compressed_payload = encoder.finish()?;

            write_tagged_element(&mut out, MI_COMPRESSED, &compressed_payload);
        } else {
            out.extend_from_slice(&matrix_bytes);
        }
    }

    Ok(out)
}

/// Encodes a single named `Value` into a `miMATRIX` byte block (including its 8-byte tag).
fn encode_matrix(name: &str, val: &Value) -> Result<Vec<u8>, MatError> {
    let mut payload = Vec::new();

    let (class_id, flags) = match val {
        Value::Double(_) => (MX_DOUBLE_CLASS, 0u8),
        Value::ComplexDouble(_) => (MX_DOUBLE_CLASS, FLAG_COMPLEX),
        Value::Single(_) => (MX_SINGLE_CLASS, 0u8),
        Value::Logical(_) => (MX_LOGICAL_CLASS, FLAG_LOGICAL),
        Value::Int8(_) => (MX_INT8_CLASS, 0u8),
        Value::UInt8(_) => (MX_UINT8_CLASS, 0u8),
        Value::Int16(_) => (MX_INT16_CLASS, 0u8),
        Value::UInt16(_) => (MX_UINT16_CLASS, 0u8),
        Value::Int32(_) => (MX_INT32_CLASS, 0u8),
        Value::UInt32(_) => (MX_UINT32_CLASS, 0u8),
        Value::Int64(_) => (MX_INT64_CLASS, 0u8),
        Value::UInt64(_) => (MX_UINT64_CLASS, 0u8),
        Value::Sparse(_) => (MX_SPARSE_CLASS, 0u8),
        Value::Char(_) => (MX_CHAR_CLASS, 0u8),
        Value::Cell(_) => (MX_CELL_CLASS, 0u8),
        Value::Struct(_) => (MX_STRUCT_CLASS, 0u8),
        _ => (MX_DOUBLE_CLASS, 0u8),
    };

    // 1. Array Flags (miUINT32, 8 bytes payload)
    let mut flags_buf = [0u8; 8];
    flags_buf[0] = class_id;
    flags_buf[1] = flags;
    write_tagged_element(&mut payload, MI_UINT32, &flags_buf);

    // 2. Dimensions Array (miINT32)
    let shp = val.shape();
    let mut dims_buf = Vec::with_capacity(shp.len() * 4);
    for &d in shp {
        dims_buf.extend_from_slice(&(d as i32).to_le_bytes());
    }
    write_tagged_element(&mut payload, MI_INT32, &dims_buf);

    // 3. Array Name (miINT8)
    write_tagged_element(&mut payload, MI_INT8, name.as_bytes());

    // 4. Data Payload
    match val {
        Value::Double(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 8);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_DOUBLE, &pr_buf);
        }
        Value::ComplexDouble(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 8);
            let mut pi_buf = Vec::with_capacity(da.numel() * 8);
            for c in da.as_slice() {
                pr_buf.extend_from_slice(&c.re.to_le_bytes());
                pi_buf.extend_from_slice(&c.im.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_DOUBLE, &pr_buf);
            write_tagged_element(&mut payload, MI_DOUBLE, &pi_buf);
        }
        Value::Single(sa) => {
            let mut pr_buf = Vec::with_capacity(sa.numel() * 4);
            for &x in sa.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_SINGLE, &pr_buf);
        }
        Value::Int8(da) => {
            let pr_buf: Vec<u8> = da.as_slice().iter().map(|&x| x as u8).collect();
            write_tagged_element(&mut payload, MI_INT8, &pr_buf);
        }
        Value::UInt8(da) => {
            write_tagged_element(&mut payload, MI_UINT8, da.as_slice());
        }
        Value::Int16(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 2);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_INT16, &pr_buf);
        }
        Value::UInt16(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 2);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_UINT16, &pr_buf);
        }
        Value::Int32(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 4);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_INT32, &pr_buf);
        }
        Value::UInt32(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 4);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_UINT32, &pr_buf);
        }
        Value::Int64(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 8);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_INT64, &pr_buf);
        }
        Value::UInt64(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel() * 8);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_UINT64, &pr_buf);
        }
        Value::Logical(da) => {
            let mut pr_buf = Vec::with_capacity(da.numel());
            for &b in da.as_slice() {
                pr_buf.push(if b { 1 } else { 0 });
            }
            write_tagged_element(&mut payload, MI_UINT8, &pr_buf);
        }
        Value::Char(ca) => {
            let mut pr_buf = Vec::with_capacity(ca.numel() * 2);
            for &c in ca.as_slice() {
                pr_buf.extend_from_slice(&(c as u16).to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_UINT16, &pr_buf);
        }
        Value::Sparse(s) => {
            let mut ir_buf = Vec::with_capacity(s.row_ind.len() * 4);
            for &r in &s.row_ind {
                ir_buf.extend_from_slice(&(r as i32).to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_INT32, &ir_buf);

            let mut jc_buf = Vec::with_capacity(s.col_ptr.len() * 4);
            for &c in &s.col_ptr {
                jc_buf.extend_from_slice(&(c as i32).to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_INT32, &jc_buf);

            let mut pr_buf = Vec::with_capacity(s.values.len() * 8);
            for &v in &s.values {
                pr_buf.extend_from_slice(&v.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_DOUBLE, &pr_buf);
        }
        Value::Cell(ca) => {
            for item in ca.as_slice() {
                let cell_sub_bytes = encode_matrix("", item)?;
                payload.extend_from_slice(&cell_sub_bytes);
            }
        }
        Value::Struct(s) => {
            let fn_len: i32 = 32;
            write_tagged_element(&mut payload, MI_INT32, &fn_len.to_le_bytes());

            let mut fn_buf = Vec::with_capacity(s.fields.len() * 32);
            for f in &s.fields {
                let mut chunk = [0u8; 32];
                let bytes = f.as_bytes();
                let len = bytes.len().min(31);
                chunk[..len].copy_from_slice(&bytes[..len]);
                fn_buf.extend_from_slice(&chunk);
            }
            write_tagged_element(&mut payload, MI_INT8, &fn_buf);

            let num_elems: usize = if s.shape.is_empty() {
                0
            } else {
                s.shape.iter().product()
            };
            for f in &s.fields {
                for i in 0..num_elems {
                    let field_val = s
                        .data
                        .get(i)
                        .and_then(|m| m.get(f))
                        .cloned()
                        .unwrap_or_else(Value::empty);
                    let sub_bytes = encode_matrix("", &field_val)?;
                    payload.extend_from_slice(&sub_bytes);
                }
            }
        }
        _ => {
            // Fallback for doubles
            let da = val.to_double_array()?;
            let mut pr_buf = Vec::with_capacity(da.numel() * 8);
            for &x in da.as_slice() {
                pr_buf.extend_from_slice(&x.to_le_bytes());
            }
            write_tagged_element(&mut payload, MI_DOUBLE, &pr_buf);
        }
    }

    // Wrap the entire payload in a miMATRIX tag
    let mut matrix_data = Vec::new();
    write_tagged_element(&mut matrix_data, MI_MATRIX, &payload);
    Ok(matrix_data)
}

/// Writes a standard 8-byte tag or SDE followed by payload and null padding.
fn write_tagged_element(out: &mut Vec<u8>, data_type: u32, payload: &[u8]) {
    let num_bytes = payload.len();

    if num_bytes > 0 && num_bytes <= 4 && data_type <= 0xFFFF {
        // Encode as Small Data Element (SDE)
        let first_u32 = (data_type & 0xFFFF) | ((num_bytes as u32) << 16);
        out.extend_from_slice(&first_u32.to_le_bytes());
        let mut sde_buf = [0u8; 4];
        sde_buf[..num_bytes].copy_from_slice(payload);
        out.extend_from_slice(&sde_buf);
    } else {
        // Standard 8-byte tag
        out.extend_from_slice(&data_type.to_le_bytes());
        out.extend_from_slice(&(num_bytes as u32).to_le_bytes());
        out.extend_from_slice(payload);

        // 8-byte boundary padding (miCOMPRESSED streams are packed contiguously without padding)
        if data_type != MI_COMPRESSED {
            let padding = (8 - (num_bytes % 8)) % 8;
            if padding > 0 {
                out.extend_from_slice(&vec![0u8; padding]);
            }
        }
    }
}
