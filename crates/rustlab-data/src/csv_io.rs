//! B2.1 CSV matrix I/O builtins (`readmatrix`, `writematrix`).
//!
//! Uses the `csv` crate with support for:
//! - Name-value options: `Delimiter`, `NumHeaderLines`
//! - Empty fields mapped to `NaN`
//! - RFC 4180 quote handling
//! - UTF-8 BOM stripping
//! - CRLF / LF line endings
//! - Ragged row detection with 1-based line number in error message
//! - Shortest round-trip float representation on write via `ryu`

use crate::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

fn extract_path(v: &Value) -> Result<String, EvalError> {
    match v {
        Value::Char(ca) => Ok(ca.as_slice().iter().collect()),
        Value::String(sa) => sa
            .as_slice()
            .first()
            .cloned()
            .ok_or_else(|| user_err("RUSTLAB:readmatrix:badPath", "Empty filename.")),
        _ => Err(user_err(
            "RUSTLAB:readmatrix:badPath",
            "Filename must be a character vector or string scalar.",
        )),
    }
}

fn extract_string_opt(v: &Value) -> Result<String, EvalError> {
    match v {
        Value::Char(ca) => Ok(ca.as_slice().iter().collect()),
        Value::String(sa) => sa
            .as_slice()
            .first()
            .cloned()
            .ok_or_else(|| user_err("RUSTLAB:data:badOption", "Option value cannot be empty.")),
        _ => Err(user_err(
            "RUSTLAB:data:badOption",
            "Option value must be a character vector or string scalar.",
        )),
    }
}

fn parse_delimiter(s: &str) -> Result<u8, EvalError> {
    if s.eq_ignore_ascii_case("tab") || s == "\t" {
        Ok(b'\t')
    } else if s.eq_ignore_ascii_case("space") || s == " " {
        Ok(b' ')
    } else if s.eq_ignore_ascii_case("comma") || s == "," {
        Ok(b',')
    } else if s.eq_ignore_ascii_case("semi") || s.eq_ignore_ascii_case("semicolon") || s == ";" {
        Ok(b';')
    } else if s.len() == 1 {
        let b = s.as_bytes();
        if let Some(&byte) = b.first() {
            Ok(byte)
        } else {
            Err(user_err(
                "RUSTLAB:data:badDelimiter",
                "Empty delimiter specified.",
            ))
        }
    } else {
        Err(user_err(
            "RUSTLAB:data:badDelimiter",
            &format!("Unsupported delimiter '{s}'."),
        ))
    }
}

fn parse_cell_f64(s: &str) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return f64::NAN;
    }
    if trimmed.eq_ignore_ascii_case("nan") {
        return f64::NAN;
    }
    if trimmed.eq_ignore_ascii_case("inf") || trimmed.eq_ignore_ascii_case("+inf") {
        return f64::INFINITY;
    }
    if trimmed.eq_ignore_ascii_case("-inf") {
        return f64::NEG_INFINITY;
    }
    trimmed.parse::<f64>().unwrap_or(f64::NAN)
}

/// `readmatrix(filename [, 'Delimiter', d] [, 'NumHeaderLines', n])`
pub fn readmatrix(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let filename_val = args
        .first()
        .ok_or_else(|| user_err("MATLAB:minrhs", "readmatrix requires at least 1 input."))?;
    let filename = extract_path(filename_val)?;

    let mut delimiter: u8 = b',';
    let mut num_header_lines: usize = 0;

    let mut i = 1;
    while i < args.len() {
        let opt_name = extract_string_opt(&args[i])?;
        if i + 1 >= args.len() {
            return Err(user_err(
                "RUSTLAB:data:missingOptionValue",
                &format!("Missing value for option '{opt_name}'."),
            ));
        }
        let opt_val = &args[i + 1];
        if opt_name.eq_ignore_ascii_case("delimiter") {
            let delim_str = extract_string_opt(opt_val)?;
            delimiter = parse_delimiter(&delim_str)?;
        } else if opt_name.eq_ignore_ascii_case("numheaderlines") {
            let n = opt_val.as_scalar_f64().map_err(|e| {
                user_err(
                    "RUSTLAB:data:badNumHeaderLines",
                    &format!("NumHeaderLines must be a nonnegative integer: {e}"),
                )
            })?;
            if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
                return Err(user_err(
                    "RUSTLAB:data:badNumHeaderLines",
                    "NumHeaderLines must be a nonnegative integer scalar.",
                ));
            }
            num_header_lines = n as usize;
        } else {
            return Err(user_err(
                "RUSTLAB:data:badOption",
                &format!("Unsupported readmatrix option '{opt_name}'."),
            ));
        }
        i += 2;
    }

    let path = Path::new(&filename);
    let file = File::open(path).map_err(|e| {
        user_err(
            "RUSTLAB:readmatrix:fileNotFound",
            &format!("Failed to open file '{filename}': {e}"),
        )
    })?;
    let mut reader = BufReader::new(file);

    // UTF-8 BOM check [0xEF, 0xBB, 0xBF]
    let mut head = [0u8; 3];
    let bytes_read = reader.read(&mut head).map_err(|e| {
        user_err(
            "RUSTLAB:readmatrix:readError",
            &format!("Failed reading file header: {e}"),
        )
    })?;
    let mut prefix = Vec::new();
    if bytes_read == 3 && head == [0xEF, 0xBB, 0xBF] {
        // Strip BOM, do not prepend
    } else {
        prefix.extend_from_slice(&head[..bytes_read]);
    }

    // Skip NumHeaderLines
    let mut current_line = 0usize;
    let mut combined_reader: Box<dyn Read> = if !prefix.is_empty() {
        Box::new(std::io::Cursor::new(prefix).chain(reader))
    } else {
        Box::new(reader)
    };

    if num_header_lines > 0 {
        let mut line_buf = Vec::new();
        let mut buf_rdr = BufReader::new(combined_reader);
        for _ in 0..num_header_lines {
            line_buf.clear();
            let n = buf_rdr.read_until(b'\n', &mut line_buf).map_err(|e| {
                user_err(
                    "RUSTLAB:readmatrix:readError",
                    &format!("Error skipping header lines: {e}"),
                )
            })?;
            if n == 0 {
                break; // EOF reached during header skipping
            }
            current_line += 1;
        }
        combined_reader = Box::new(buf_rdr);
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(combined_reader);

    let mut rows_data: Vec<Vec<f64>> = Vec::new();
    let mut expected_cols: Option<usize> = None;

    for result in csv_reader.records() {
        current_line += 1;
        let record = result.map_err(|e| {
            user_err(
                "RUSTLAB:readmatrix:parseError",
                &format!("Line {current_line}: CSV parse error: {e}"),
            )
        })?;

        if record.is_empty() {
            continue;
        }

        let num_fields = record.len();
        if let Some(expected) = expected_cols {
            if num_fields != expected {
                return Err(user_err(
                    "RUSTLAB:readmatrix:raggedData",
                    &format!("Row {current_line} has {num_fields} fields, expected {expected}."),
                ));
            }
        } else {
            expected_cols = Some(num_fields);
        }

        let mut row_vals = Vec::with_capacity(num_fields);
        for field in record.iter() {
            row_vals.push(parse_cell_f64(field));
        }
        rows_data.push(row_vals);
    }

    let num_rows = rows_data.len();
    let num_cols = expected_cols.unwrap_or(0);
    if num_rows == 0 || num_cols == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    // Convert row-major rows_data to column-major DenseArray
    let mut col_major = vec![0.0; num_rows * num_cols];
    for (r, row) in rows_data.iter().enumerate() {
        for (c, &val) in row.iter().enumerate() {
            col_major[r + c * num_rows] = val;
        }
    }

    let da = DenseArray::from_vec_unchecked(vec![num_rows, num_cols], col_major);
    Ok(vec![Value::Double(da)])
}

/// `writematrix(A, filename [, 'Delimiter', d])`
pub fn writematrix(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "writematrix requires at least 2 inputs: array and filename.",
        ));
    }
    let da = args[0]
        .to_double_array()
        .map_err(|e| user_err("RUSTLAB:data:badInput", &format!("{e}")))?;
    let filename = extract_path(&args[1])?;

    let mut delimiter: u8 = b',';
    let mut i = 2;
    while i < args.len() {
        let opt_name = extract_string_opt(&args[i])?;
        if i + 1 >= args.len() {
            return Err(user_err(
                "RUSTLAB:data:missingOptionValue",
                &format!("Missing value for option '{opt_name}'."),
            ));
        }
        let opt_val = &args[i + 1];
        if opt_name.eq_ignore_ascii_case("delimiter") {
            let delim_str = extract_string_opt(opt_val)?;
            delimiter = parse_delimiter(&delim_str)?;
        } else {
            return Err(user_err(
                "RUSTLAB:data:badOption",
                &format!("Unsupported writematrix option '{opt_name}'."),
            ));
        }
        i += 2;
    }

    let shape = da.shape();
    if shape.len() > 2 {
        return Err(user_err(
            "RUSTLAB:writematrix:badDimension",
            "writematrix only supports 2-D arrays.",
        ));
    }
    let rows = shape.first().copied().unwrap_or(0);
    let cols = shape
        .get(1)
        .copied()
        .unwrap_or(if rows > 0 { 1 } else { 0 });

    let file = File::create(&filename).map_err(|e| {
        user_err(
            "RUSTLAB:writematrix:createFailed",
            &format!("Failed creating file '{filename}': {e}"),
        )
    })?;
    let mut writer = BufWriter::new(file);
    let delim_char = delimiter as char;

    let mut ryu_buf = ryu::Buffer::new();

    for r in 0..rows {
        for c in 0..cols {
            if c > 0 {
                write!(writer, "{delim_char}").map_err(|e| {
                    user_err(
                        "RUSTLAB:writematrix:writeFailed",
                        &format!("Write error: {e}"),
                    )
                })?;
            }
            let val = da.as_slice()[r + c * rows];
            if val.is_nan() {
                write!(writer, "NaN").map_err(|e| {
                    user_err(
                        "RUSTLAB:writematrix:writeFailed",
                        &format!("Write error: {e}"),
                    )
                })?;
            } else if val.is_infinite() {
                if val.is_sign_positive() {
                    write!(writer, "Inf").map_err(|e| {
                        user_err(
                            "RUSTLAB:writematrix:writeFailed",
                            &format!("Write error: {e}"),
                        )
                    })?;
                } else {
                    write!(writer, "-Inf").map_err(|e| {
                        user_err(
                            "RUSTLAB:writematrix:writeFailed",
                            &format!("Write error: {e}"),
                        )
                    })?;
                }
            } else {
                let formatted = ryu_buf.format(val);
                write!(writer, "{formatted}").map_err(|e| {
                    user_err(
                        "RUSTLAB:writematrix:writeFailed",
                        &format!("Write error: {e}"),
                    )
                })?;
            }
        }
        writeln!(writer).map_err(|e| {
            user_err(
                "RUSTLAB:writematrix:writeFailed",
                &format!("Write error: {e}"),
            )
        })?;
    }

    writer.flush().map_err(|e| {
        user_err(
            "RUSTLAB:writematrix:flushFailed",
            &format!("Flush error: {e}"),
        )
    })?;

    Ok(Vec::new())
}
