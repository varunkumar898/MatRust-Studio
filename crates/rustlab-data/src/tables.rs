//! Tabular dataset backend: readtable, writetable, head, tail, summary.

use crate::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn extract_path(v: &Value) -> Result<String, EvalError> {
    match v {
        Value::Char(ca) => Ok(ca.as_slice().iter().collect()),
        Value::String(sa) => sa
            .as_slice()
            .first()
            .cloned()
            .ok_or_else(|| user_err("MATLAB:readtable:badPath", "Empty filename.")),
        _ => Err(user_err(
            "MATLAB:readtable:badPath",
            "Filename must be a character vector or string scalar.",
        )),
    }
}

/// Cleans a header name to be a valid identifier.
fn sanitize_var_name(raw: &str, index: usize) -> String {
    let trimmed = raw.trim().trim_matches('"');
    let mut clean = String::new();
    for (i, ch) in trimmed.chars().enumerate() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            if i == 0 && ch.is_ascii_digit() {
                clean.push('x');
            }
            clean.push(ch);
        } else {
            clean.push('_');
        }
    }
    if clean.is_empty() {
        format!("Var{}", index + 1)
    } else {
        clean
    }
}

/// Reads tabular data from a CSV/TSV/delimited text file into a Table structure.
pub fn readtable(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "readtable requires at least 1 input.",
        ));
    }
    let path_str = extract_path(&args[0])?;
    let path = Path::new(&path_str);
    if !path.exists() {
        return Err(user_err(
            "MATLAB:readtable:fileNotFound",
            &format!("File '{path_str}' not found."),
        ));
    }

    let file = File::open(path).map_err(|e| {
        user_err(
            "MATLAB:readtable:ioError",
            &format!("Cannot open file '{path_str}': {e}"),
        )
    })?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let first_line = match lines.next() {
        Some(Ok(l)) => l,
        Some(Err(e)) => return Err(user_err("MATLAB:readtable:ioError", &format!("{e}"))),
        None => {
            return Ok(vec![Value::Struct(StructArray::empty())]);
        }
    };

    // Strip UTF-8 BOM if present
    let header_str = first_line.strip_prefix('\u{feff}').unwrap_or(&first_line);
    // Detect delimiter
    let delim = if header_str.contains('\t') {
        '\t'
    } else if header_str.contains(';') {
        ';'
    } else {
        ','
    };

    let raw_headers: Vec<&str> = header_str.split(delim).collect();
    let col_count = raw_headers.len();
    let mut col_names = Vec::with_capacity(col_count);
    for (i, h) in raw_headers.iter().enumerate() {
        let name = sanitize_var_name(h, i);
        col_names.push(name);
    }

    let mut raw_rows: Vec<Vec<String>> = Vec::new();
    for line_res in lines {
        let line = line_res.map_err(|e| user_err("MATLAB:readtable:ioError", &format!("{e}")))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let fields: Vec<String> = trimmed
            .split(delim)
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();
        raw_rows.push(fields);
    }

    let row_count = raw_rows.len();
    let mut col_values: HashMap<String, Value> = HashMap::new();

    for (col_idx, name) in col_names.iter().enumerate() {
        // Test if column is purely numeric
        let mut is_numeric = true;
        let mut num_col = Vec::with_capacity(row_count);
        for row in &raw_rows {
            let field = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            if field.is_empty() || field.eq_ignore_ascii_case("nan") {
                num_col.push(f64::NAN);
            } else if let Ok(val) = field.parse::<f64>() {
                num_col.push(val);
            } else {
                is_numeric = false;
                break;
            }
        }

        if is_numeric {
            let da = DenseArray::from_vec_unchecked(vec![row_count, 1], num_col);
            col_values.insert(name.clone(), Value::Double(da));
        } else {
            let mut str_col = Vec::with_capacity(row_count);
            for row in &raw_rows {
                let field = row.get(col_idx).cloned().unwrap_or_default();
                str_col.push(field);
            }
            let da = DenseArray::from_vec_unchecked(vec![row_count, 1], str_col);
            col_values.insert(name.clone(), Value::String(da));
        }
    }

    // Add metadata
    let var_names_da = DenseArray::from_vec_unchecked(
        vec![1, col_names.len()],
        col_names.iter().map(|s| Value::string_scalar(s)).collect(),
    );
    col_values.insert("_VariableNames".to_string(), Value::Cell(var_names_da));
    col_values.insert(
        "_NumRows".to_string(),
        Value::Double(DenseArray::from_vec_unchecked(
            vec![1, 1],
            vec![row_count as f64],
        )),
    );

    let mut fields: Vec<String> = col_names;
    fields.push("_VariableNames".to_string());
    fields.push("_NumRows".to_string());

    let sa = StructArray::new(vec![1, 1], fields, vec![col_values]);
    Ok(vec![Value::Struct(sa)])
}

/// Writes tabular data struct or matrix to a CSV file.
pub fn writetable(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "writetable requires at least 2 inputs (table, filename).",
        ));
    }
    let path_str = extract_path(&args[1])?;
    let path = Path::new(&path_str);
    let file = File::create(path).map_err(|e| {
        user_err(
            "MATLAB:writetable:ioError",
            &format!("Cannot create file '{path_str}': {e}"),
        )
    })?;
    let mut writer = BufWriter::new(file);

    match &args[0] {
        Value::Struct(sa) => {
            if sa.data.is_empty() {
                return Ok(vec![Value::Double(DenseArray::empty())]);
            }
            let map = &sa.data[0];
            let cols: Vec<&String> = sa.fields.iter().filter(|f| !f.starts_with('_')).collect();

            // Write header
            let header_line = cols
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(",");
            writer
                .write_all(header_line.as_bytes())
                .map_err(|e| user_err("MATLAB:writetable:ioError", &format!("{e}")))?;
            writer
                .write_all(b"\n")
                .map_err(|e| user_err("MATLAB:writetable:ioError", &format!("{e}")))?;

            let num_rows = if let Some(Value::Double(d)) = map.get("_NumRows") {
                d.as_slice().first().copied().unwrap_or(0.0) as usize
            } else if let Some(first_col) = cols.first().and_then(|c| map.get(*c)) {
                first_col.shape().first().copied().unwrap_or(0)
            } else {
                0
            };

            for r in 0..num_rows {
                let mut row_items = Vec::with_capacity(cols.len());
                for c in &cols {
                    if let Some(col_val) = map.get(*c) {
                        match col_val {
                            Value::Double(d) => {
                                if let Some(&v) = d.as_slice().get(r) {
                                    row_items.push(format!("{v}"));
                                } else {
                                    row_items.push(String::new());
                                }
                            }
                            Value::String(s) => {
                                if let Some(item) = s.as_slice().get(r) {
                                    row_items.push(format!("\"{item}\""));
                                } else {
                                    row_items.push(String::new());
                                }
                            }
                            other => row_items.push(format!("{other}")),
                        }
                    } else {
                        row_items.push(String::new());
                    }
                }
                let line = row_items.join(",") + "\n";
                writer
                    .write_all(line.as_bytes())
                    .map_err(|e| user_err("MATLAB:writetable:ioError", &format!("{e}")))?;
            }
            writer
                .flush()
                .map_err(|e| user_err("MATLAB:writetable:ioError", &format!("{e}")))?;
            Ok(vec![Value::Double(DenseArray::empty())])
        }
        _ => Err(user_err(
            "MATLAB:writetable:invalidInput",
            "First argument must be a table or struct.",
        )),
    }
}

/// Returns the first k rows of a table or matrix (default k = 8).
pub fn head_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "head requires at least 1 input."));
    }
    let k = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(|_| {
            user_err(
                "MATLAB:head:invalidK",
                "Row count k must be a positive integer scalar.",
            )
        })? as usize
    } else {
        8
    };

    match &args[0] {
        Value::Struct(sa) => {
            if sa.data.is_empty() {
                return Ok(vec![Value::Struct(StructArray::empty())]);
            }
            let map = &sa.data[0];
            let mut new_map = HashMap::new();
            let mut new_fields = Vec::new();

            for f in &sa.fields {
                new_fields.push(f.clone());
                if f.starts_with('_') {
                    if f == "_NumRows" {
                        new_map.insert(
                            f.clone(),
                            Value::Double(DenseArray::from_vec_unchecked(
                                vec![1, 1],
                                vec![k as f64],
                            )),
                        );
                    } else if let Some(v) = map.get(f) {
                        new_map.insert(f.clone(), v.clone());
                    }
                    continue;
                }
                if let Some(col_val) = map.get(f) {
                    match col_val {
                        Value::Double(d) => {
                            let end = k.min(d.numel());
                            let sub = d.as_slice()[..end].to_vec();
                            new_map.insert(
                                f.clone(),
                                Value::Double(DenseArray::from_vec_unchecked(vec![end, 1], sub)),
                            );
                        }
                        Value::String(s) => {
                            let end = k.min(s.numel());
                            let sub = s.as_slice()[..end].to_vec();
                            new_map.insert(
                                f.clone(),
                                Value::String(DenseArray::from_vec_unchecked(vec![end, 1], sub)),
                            );
                        }
                        other => {
                            new_map.insert(f.clone(), other.clone());
                        }
                    }
                }
            }
            Ok(vec![Value::Struct(StructArray::new(
                vec![1, 1],
                new_fields,
                vec![new_map],
            ))])
        }
        Value::Double(d) => {
            let shp = d.shape();
            let rows = shp.first().copied().unwrap_or(0);
            let cols = shp.get(1).copied().unwrap_or(1);
            let take_rows = k.min(rows);
            let mut sub = Vec::with_capacity(take_rows * cols);
            for c in 0..cols {
                for r in 0..take_rows {
                    sub.push(d.as_slice()[r + c * rows]);
                }
            }
            Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
                vec![take_rows, cols],
                sub,
            ))])
        }
        _ => Err(user_err(
            "MATLAB:head:invalidInput",
            "Input must be a table or matrix.",
        )),
    }
}

/// Returns the last k rows of a table or matrix (default k = 8).
pub fn tail_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "tail requires at least 1 input."));
    }
    let k = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(|_| {
            user_err(
                "MATLAB:tail:invalidK",
                "Row count k must be a positive integer scalar.",
            )
        })? as usize
    } else {
        8
    };

    match &args[0] {
        Value::Struct(sa) => {
            if sa.data.is_empty() {
                return Ok(vec![Value::Struct(StructArray::empty())]);
            }
            let map = &sa.data[0];
            let mut new_map = HashMap::new();
            let mut new_fields = Vec::new();

            for f in &sa.fields {
                new_fields.push(f.clone());
                if f.starts_with('_') {
                    if f == "_NumRows" {
                        new_map.insert(
                            f.clone(),
                            Value::Double(DenseArray::from_vec_unchecked(
                                vec![1, 1],
                                vec![k as f64],
                            )),
                        );
                    } else if let Some(v) = map.get(f) {
                        new_map.insert(f.clone(), v.clone());
                    }
                    continue;
                }
                if let Some(col_val) = map.get(f) {
                    match col_val {
                        Value::Double(d) => {
                            let total = d.numel();
                            let start = total.saturating_sub(k);
                            let sub = d.as_slice()[start..].to_vec();
                            new_map.insert(
                                f.clone(),
                                Value::Double(DenseArray::from_vec_unchecked(
                                    vec![sub.len(), 1],
                                    sub,
                                )),
                            );
                        }
                        Value::String(s) => {
                            let total = s.numel();
                            let start = total.saturating_sub(k);
                            let sub = s.as_slice()[start..].to_vec();
                            new_map.insert(
                                f.clone(),
                                Value::String(DenseArray::from_vec_unchecked(
                                    vec![sub.len(), 1],
                                    sub,
                                )),
                            );
                        }
                        other => {
                            new_map.insert(f.clone(), other.clone());
                        }
                    }
                }
            }
            Ok(vec![Value::Struct(StructArray::new(
                vec![1, 1],
                new_fields,
                vec![new_map],
            ))])
        }
        Value::Double(d) => {
            let shp = d.shape();
            let rows = shp.first().copied().unwrap_or(0);
            let cols = shp.get(1).copied().unwrap_or(1);
            let start_r = rows.saturating_sub(k);
            let take_rows = rows - start_r;
            let mut sub = Vec::with_capacity(take_rows * cols);
            for c in 0..cols {
                for r in start_r..rows {
                    sub.push(d.as_slice()[r + c * rows]);
                }
            }
            Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
                vec![take_rows, cols],
                sub,
            ))])
        }
        _ => Err(user_err(
            "MATLAB:tail:invalidInput",
            "Input must be a table or matrix.",
        )),
    }
}

/// Computes summary statistics for table columns.
pub fn summary_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "summary requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Struct(sa) => {
            if sa.data.is_empty() {
                return Ok(vec![Value::Struct(StructArray::empty())]);
            }
            let map = &sa.data[0];
            let cols: Vec<&String> = sa.fields.iter().filter(|f| !f.starts_with('_')).collect();
            let mut summary_map = HashMap::new();
            let mut summary_fields = Vec::new();

            for c in cols {
                if let Some(Value::Double(d)) = map.get(c) {
                    let s = d.as_slice();
                    let n = s.len();
                    if n > 0 {
                        let mut min_val = f64::INFINITY;
                        let mut max_val = f64::NEG_INFINITY;
                        let mut sum_val = 0.0;
                        for &v in s {
                            if !v.is_nan() {
                                if v < min_val {
                                    min_val = v;
                                }
                                if v > max_val {
                                    max_val = v;
                                }
                                sum_val += v;
                            }
                        }
                        let mean_val = sum_val / (n as f64);
                        let mut stat_inner = HashMap::new();
                        stat_inner.insert("Min".to_string(), Value::scalar(min_val));
                        stat_inner.insert("Mean".to_string(), Value::scalar(mean_val));
                        stat_inner.insert("Max".to_string(), Value::scalar(max_val));
                        let inner_sa = StructArray::scalar(stat_inner);
                        summary_map.insert((*c).clone(), Value::Struct(inner_sa));
                        summary_fields.push((*c).clone());
                    }
                }
            }
            Ok(vec![Value::Struct(StructArray::new(
                vec![1, 1],
                summary_fields,
                vec![summary_map],
            ))])
        }
        _ => Err(user_err(
            "MATLAB:summary:invalidInput",
            "Input must be a table.",
        )),
    }
}
