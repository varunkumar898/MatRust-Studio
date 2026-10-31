//! Reader for Level 5 and v7 MAT-files.

#![allow(clippy::chunks_exact_to_as_chunks)]

use crate::error::MatError;
use crate::types::*;
use flate2::read::ZlibDecoder;
use rustlab_math::{Complex64, DenseArray, Value};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// Decodes an in-memory MAT-file byte slice into a variable dictionary.
pub fn read_mat(bytes: &[u8]) -> Result<HashMap<String, Value>, MatError> {
    if bytes.len() < 128 {
        return Err(MatError::InvalidHeader(
            "File smaller than 128-byte Level 5 header.".to_string(),
        ));
    }

    // Verify endianness
    let endian = [bytes[126], bytes[127]];
    if endian != *b"IM" {
        return Err(MatError::UnsupportedEndian(endian));
    }

    // Version
    let version = u16::from_le_bytes([bytes[124], bytes[125]]);
    if version != 0x0100 {
        return Err(MatError::InvalidHeader(format!(
            "Unsupported MAT version: 0x{:04x}",
            version
        )));
    }

    let mut cursor = Cursor::new(&bytes[128..]);
    let mut vars = HashMap::new();

    while (cursor.position() as usize) < cursor.get_ref().len() {
        if let Some((name, val)) = read_top_level_element(&mut cursor)? {
            vars.insert(name, val);
        }
    }

    Ok(vars)
}

/// Reads a single top-level data element (either uncompressed miMATRIX or miCOMPRESSED).
fn read_top_level_element(cursor: &mut Cursor<&[u8]>) -> Result<Option<(String, Value)>, MatError> {
    if (cursor.position() as usize) >= cursor.get_ref().len() {
        return Ok(None);
    }

    let (data_type, payload) = read_tagged_element(cursor)?;

    if data_type == MI_COMPRESSED {
        // Inflate with zlib
        let mut decoder = ZlibDecoder::new(&payload[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| MatError::Decompress(e.to_string()))?;

        let mut sub_cursor = Cursor::new(&decompressed[..]);
        return read_top_level_element(&mut sub_cursor);
    }

    if data_type == MI_MATRIX {
        let mut sub_cursor = Cursor::new(&payload[..]);
        let (name, val) = decode_matrix(&mut sub_cursor)?;
        return Ok(Some((name, val)));
    }

    Ok(None)
}

/// Reads a tagged element, supporting both standard 8-byte tags and SDEs.
fn read_tagged_element(cursor: &mut Cursor<&[u8]>) -> Result<(u32, Vec<u8>), MatError> {
    let mut header_bytes = [0u8; 4];
    if cursor.read_exact(&mut header_bytes).is_err() {
        return Err(MatError::UnexpectedEof { element: "tag" });
    }
    let first_u32 = u32::from_le_bytes(header_bytes);

    let upper_16 = (first_u32 >> 16) as u16;
    let data_type;
    let num_bytes;
    let payload;

    if upper_16 != 0 {
        // Compressed Small Data Element (SDE): [data_type: u16, num_bytes: u16, data: 4 bytes]
        data_type = first_u32 & 0xFFFF;
        num_bytes = upper_16 as usize;

        let mut sde_data = [0u8; 4];
        cursor
            .read_exact(&mut sde_data)
            .map_err(|_| MatError::UnexpectedEof {
                element: "SDE data",
            })?;
        payload = sde_data[..num_bytes.min(4)].to_vec();
    } else {
        // Standard 8-byte tag
        data_type = first_u32;
        let mut size_bytes = [0u8; 4];
        cursor
            .read_exact(&mut size_bytes)
            .map_err(|_| MatError::UnexpectedEof {
                element: "tag size",
            })?;
        num_bytes = u32::from_le_bytes(size_bytes) as usize;

        let mut buf = vec![0u8; num_bytes];
        cursor
            .read_exact(&mut buf)
            .map_err(|_| MatError::UnexpectedEof { element: "payload" })?;
        payload = buf;

        // 8-byte alignment padding (miCOMPRESSED streams are packed contiguously without padding)
        if data_type != MI_COMPRESSED {
            let padding = (8 - (num_bytes % 8)) % 8;
            if padding > 0 {
                let mut pad_buf = vec![0u8; padding];
                let _ = cursor.read_exact(&mut pad_buf);
            }
        }
    }

    Ok((data_type, payload))
}

/// Decodes an uncompressed `miMATRIX` sub-element stream.
fn decode_matrix(cursor: &mut Cursor<&[u8]>) -> Result<(String, Value), MatError> {
    // 1. Array Flags
    let (_, flags_payload) = read_tagged_element(cursor)?;
    if flags_payload.len() < 8 {
        return Err(MatError::InvalidHeader("Array flags too short".to_string()));
    }
    let class_id = flags_payload[0];
    let flags = flags_payload[1];
    let is_complex = (flags & FLAG_COMPLEX) != 0;
    let is_logical = (flags & FLAG_LOGICAL) != 0;

    // 2. Dimensions Array
    let (_, dims_payload) = read_tagged_element(cursor)?;
    let num_dims = dims_payload.len() / 4;
    let mut dims = Vec::with_capacity(num_dims);
    for i in 0..num_dims {
        let chunk = &dims_payload[i * 4..(i + 1) * 4];
        dims.push(i32::from_le_bytes(chunk.try_into().unwrap_or([0; 4])) as usize);
    }
    if dims.len() == 1 {
        dims.push(1);
    }

    // 3. Array Name
    let (_, name_payload) = read_tagged_element(cursor)?;
    let name = String::from_utf8_lossy(&name_payload)
        .trim_matches('\0')
        .to_string();

    let total_elements: usize = dims.iter().product();

    if class_id == MX_SPARSE_CLASS {
        let (_, ir_payload) = read_tagged_element(cursor)?;
        let row_ind: Vec<usize> = ir_payload
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes(c.try_into().unwrap_or([0; 4])) as usize)
            .collect();

        let (_, jc_payload) = read_tagged_element(cursor)?;
        let col_ptr: Vec<usize> = jc_payload
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes(c.try_into().unwrap_or([0; 4])) as usize)
            .collect();

        let (_, pr_payload) = read_tagged_element(cursor)?;
        let values: Vec<f64> = pr_payload
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
            .collect();

        let m = if !dims.is_empty() { dims[0] } else { 0 };
        let n = if dims.len() > 1 { dims[1] } else { 0 };

        let sp = rustlab_math::CscMatrix {
            shape: [m, n],
            col_ptr,
            row_ind,
            values,
        };
        return Ok((name, Value::Sparse(sp)));
    }

    // Structure array (mxSTRUCT_CLASS): the element stream after the name is
    //   - Field Name Length (miINT32): N bytes per field name (typically 32),
    //   - Field Names (miINT8): num_fields * N bytes, each NUL-padded,
    //   - for every array element: num_fields consecutive nested miMATRIX
    //     elements in field-name order.
    // Reference: MATLAB Level 5 MAT-file format spec (mxSTRUCT_CLASS layout).
    if class_id == MX_STRUCT_CLASS {
        let (_, field_name_len_payload) = read_tagged_element(cursor)?;
        let field_name_len: usize = field_name_len_payload
            .first_chunk::<4>()
            .map(|c| i32::from_le_bytes(*c).max(1) as usize)
            .unwrap_or(32);

        let (_, field_names_payload) = read_tagged_element(cursor)?;
        let num_fields = field_names_payload
            .len()
            .checked_div(field_name_len)
            .unwrap_or(0);

        let mut field_names = Vec::with_capacity(num_fields);
        for chunk in field_names_payload.chunks_exact(field_name_len) {
            let end = chunk.iter().position(|&b| b == 0).unwrap_or(chunk.len());
            let name_str = String::from_utf8_lossy(&chunk[..end]).trim().to_string();
            field_names.push(name_str);
        }

        // In MATLAB v5, for each field there are total_elements miMATRIX sub-elements
        let mut struct_elements: Vec<HashMap<String, Value>> = vec![HashMap::new(); total_elements];
        for f in &field_names {
            for elem_idx in 0..total_elements {
                if let Ok(Some((_, v))) = read_top_level_element(cursor) {
                    if elem_idx < struct_elements.len() {
                        struct_elements[elem_idx].insert(f.clone(), v);
                    }
                }
            }
        }

        let st = rustlab_math::StructArray::new(dims, field_names, struct_elements);
        return Ok((name, Value::Struct(st)));
    }

    // 4. Real Data Part
    let (data_type, pr_payload) = read_tagged_element(cursor)?;

    if class_id == MX_CELL_CLASS {
        // Cell array contains total_elements consecutive miMATRIX entries
        let mut sub_cell_cursor = Cursor::new(&pr_payload[..]);
        let mut cell_items = Vec::with_capacity(total_elements);
        for _ in 0..total_elements {
            if let Ok(Some((_, v))) = read_top_level_element(&mut sub_cell_cursor) {
                cell_items.push(v);
            } else {
                cell_items.push(Value::empty());
            }
        }
        let arr = DenseArray::from_vec(dims, cell_items)?;
        return Ok((name, Value::Cell(arr)));
    }

    if class_id == MX_CHAR_CLASS {
        // Character vector/matrix
        let chars: Vec<char> = if data_type == MI_UINT16 {
            pr_payload
                .chunks_exact(2)
                .map(|c| {
                    char::from_u32(u16::from_le_bytes(c.try_into().unwrap_or([0; 2])) as u32)
                        .unwrap_or(' ')
                })
                .collect()
        } else {
            pr_payload.iter().map(|&b| b as char).collect()
        };
        let arr = DenseArray::from_vec(dims, chars)?;
        return Ok((name, Value::Char(arr)));
    }

    if class_id == MX_SINGLE_CLASS {
        let single_data: Vec<f32> = if data_type == MI_SINGLE {
            pr_payload
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4])))
                .collect()
        } else if data_type == MI_DOUBLE {
            pr_payload
                .chunks_exact(8)
                .map(|c| f64::from_le_bytes(c.try_into().unwrap_or([0; 8])) as f32)
                .collect()
        } else {
            return Err(MatError::UnsupportedDataType(data_type));
        };
        let arr = DenseArray::from_vec(dims, single_data)?;
        return Ok((name, Value::Single(arr)));
    }

    if class_id == MX_INT8_CLASS {
        let int_data: Vec<i8> = pr_payload.iter().map(|&b| b as i8).collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::Int8(arr)));
    }

    if class_id == MX_UINT8_CLASS && !is_logical {
        let arr = DenseArray::from_vec(dims, pr_payload)?;
        return Ok((name, Value::UInt8(arr)));
    }

    if class_id == MX_INT16_CLASS {
        let int_data: Vec<i16> = pr_payload
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes(c.try_into().unwrap_or([0; 2])))
            .collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::Int16(arr)));
    }

    if class_id == MX_UINT16_CLASS {
        let int_data: Vec<u16> = pr_payload
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes(c.try_into().unwrap_or([0; 2])))
            .collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::UInt16(arr)));
    }

    if class_id == MX_INT32_CLASS {
        let int_data: Vec<i32> = pr_payload
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes(c.try_into().unwrap_or([0; 4])))
            .collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::Int32(arr)));
    }

    if class_id == MX_UINT32_CLASS {
        let int_data: Vec<u32> = pr_payload
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap_or([0; 4])))
            .collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::UInt32(arr)));
    }

    if class_id == MX_INT64_CLASS {
        let int_data: Vec<i64> = pr_payload
            .chunks_exact(8)
            .map(|c| i64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
            .collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::Int64(arr)));
    }

    if class_id == MX_UINT64_CLASS {
        let int_data: Vec<u64> = pr_payload
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
            .collect();
        let arr = DenseArray::from_vec(dims, int_data)?;
        return Ok((name, Value::UInt64(arr)));
    }

    // Numeric parsing for Real part (double fallback or complex/logical)
    let real_f64: Vec<f64> = if data_type == MI_DOUBLE {
        pr_payload
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
            .collect()
    } else if data_type == MI_SINGLE {
        pr_payload
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4])) as f64)
            .collect()
    } else if data_type == MI_INT32 {
        pr_payload
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes(c.try_into().unwrap_or([0; 4])) as f64)
            .collect()
    } else if data_type == MI_UINT8 {
        pr_payload.iter().map(|&b| b as f64).collect()
    } else {
        return Err(MatError::UnsupportedDataType(data_type));
    };

    if is_complex {
        // 5. Imaginary Data Part
        let (_, pi_payload) = read_tagged_element(cursor)?;
        let imag_f64: Vec<f64> = pi_payload
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
            .collect();

        let mut complex_data = Vec::with_capacity(real_f64.len());
        for (i, &re) in real_f64.iter().enumerate() {
            let im = imag_f64.get(i).copied().unwrap_or(0.0);
            complex_data.push(Complex64::new(re, im));
        }
        let arr = DenseArray::from_vec(dims, complex_data)?;
        return Ok((name, Value::ComplexDouble(arr)));
    }

    if is_logical {
        let bool_data: Vec<bool> = real_f64.iter().map(|&x| x != 0.0).collect();
        let arr = DenseArray::from_vec(dims, bool_data)?;
        return Ok((name, Value::Logical(arr)));
    }

    let arr = DenseArray::from_vec(dims, real_f64)?;
    Ok((name, Value::Double(arr)))
}
