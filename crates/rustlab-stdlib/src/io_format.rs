//! Formatting, printing, and display builtins: sprintf, fprintf, disp, num2str, mat2str.
//! Implements MATLAB format-recycling semantics across array elements.

use crate::helpers::{extract_f64_array, extract_string, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{Complex64, Value};
use std::io::Write;

#[derive(Debug, Clone)]
enum FormatToken {
    Literal(String),
    Specifier {
        flags: String,
        width: Option<usize>,
        precision: Option<usize>,
        spec_type: char,
    },
}

/// Parses a C/MATLAB format string into literal chunks and conversion specifiers.
fn parse_format_string(fmt: &str) -> Vec<FormatToken> {
    let mut tokens = Vec::new();
    let mut chars = fmt.chars().peekable();
    let mut lit_buf = String::new();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Process escape sequences
            if let Some(&next) = chars.peek() {
                chars.next();
                match next {
                    'n' => lit_buf.push('\n'),
                    't' => lit_buf.push('\t'),
                    'r' => lit_buf.push('\r'),
                    '\\' => lit_buf.push('\\'),
                    '\'' => lit_buf.push('\''),
                    '"' => lit_buf.push('"'),
                    other => {
                        lit_buf.push('\\');
                        lit_buf.push(other);
                    }
                }
            } else {
                lit_buf.push('\\');
            }
        } else if ch == '%' {
            if chars.peek() == Some(&'%') {
                chars.next();
                lit_buf.push('%');
                continue;
            }

            if !lit_buf.is_empty() {
                tokens.push(FormatToken::Literal(std::mem::take(&mut lit_buf)));
            }

            // Parse specifier: %[flags][width][.precision]type
            let mut flags = String::new();
            while let Some(&c) = chars.peek() {
                if c == '-' || c == '+' || c == ' ' || c == '0' || c == '#' {
                    flags.push(c);
                    chars.next();
                } else {
                    break;
                }
            }

            let mut width_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    width_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let width = width_str.parse::<usize>().ok();

            let mut precision = None;
            if chars.peek() == Some(&'.') {
                chars.next();
                let mut prec_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        prec_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                precision = prec_str.parse::<usize>().ok();
            }

            if let Some(spec_type) = chars.next() {
                tokens.push(FormatToken::Specifier {
                    flags,
                    width,
                    precision,
                    spec_type,
                });
            }
        } else {
            lit_buf.push(ch);
        }
    }

    if !lit_buf.is_empty() {
        tokens.push(FormatToken::Literal(lit_buf));
    }

    tokens
}

#[derive(Debug, Clone)]
enum FormattableItem {
    Num(f64),
    Complex(Complex64),
    Text(String),
}

/// Flattens input arguments into formattable items in column-major order.
fn flatten_args(args: &[Value]) -> Vec<FormattableItem> {
    let mut items = Vec::new();
    for arg in args {
        match arg {
            Value::Double(da) => {
                for &val in da.as_slice() {
                    items.push(FormattableItem::Num(val));
                }
            }
            Value::ComplexDouble(da) => {
                for &c in da.as_slice() {
                    items.push(FormattableItem::Complex(c));
                }
            }
            Value::Logical(la) => {
                for &b in la.as_slice() {
                    items.push(FormattableItem::Num(if b { 1.0 } else { 0.0 }));
                }
            }
            Value::Int32(ia) => {
                for &i in ia.as_slice() {
                    items.push(FormattableItem::Num(i as f64));
                }
            }
            Value::Int64(ia) => {
                for &i in ia.as_slice() {
                    items.push(FormattableItem::Num(i as f64));
                }
            }
            Value::Char(ca) => {
                let s: String = ca.as_slice().iter().collect();
                items.push(FormattableItem::Text(s));
            }
            Value::String(sa) => {
                for s in sa.as_slice() {
                    items.push(FormattableItem::Text(s.clone()));
                }
            }
            Value::Cell(ca) => {
                for v in ca.as_slice() {
                    items.extend(flatten_args(std::slice::from_ref(v)));
                }
            }
            other => {
                items.push(FormattableItem::Text(format!("{other}")));
            }
        }
    }
    items
}

/// Formats a single item according to a specifier.
fn format_single_item(
    item: &FormattableItem,
    spec_type: char,
    flags: &str,
    width: Option<usize>,
    precision: Option<usize>,
) -> String {
    let prec = precision.unwrap_or(6);
    let s = match spec_type {
        'd' | 'i' => {
            let n = match item {
                FormattableItem::Num(x) => x.round() as i64,
                FormattableItem::Complex(c) => c.re.round() as i64,
                FormattableItem::Text(t) => t.parse::<i64>().unwrap_or(0),
            };
            format!("{n}")
        }
        'u' => {
            let n = match item {
                FormattableItem::Num(x) => x.round() as u64,
                FormattableItem::Complex(c) => c.re.round() as u64,
                FormattableItem::Text(t) => t.parse::<u64>().unwrap_or(0),
            };
            format!("{n}")
        }
        'o' => {
            let n = match item {
                FormattableItem::Num(x) => x.round() as u64,
                FormattableItem::Complex(c) => c.re.round() as u64,
                FormattableItem::Text(t) => t.parse::<u64>().unwrap_or(0),
            };
            format!("{n:o}")
        }
        'x' => {
            let n = match item {
                FormattableItem::Num(x) => x.round() as u64,
                FormattableItem::Complex(c) => c.re.round() as u64,
                FormattableItem::Text(t) => t.parse::<u64>().unwrap_or(0),
            };
            format!("{n:x}")
        }
        'X' => {
            let n = match item {
                FormattableItem::Num(x) => x.round() as u64,
                FormattableItem::Complex(c) => c.re.round() as u64,
                FormattableItem::Text(t) => t.parse::<u64>().unwrap_or(0),
            };
            format!("{n:X}")
        }
        'f' | 'F' => {
            let x = match item {
                FormattableItem::Num(x) => *x,
                FormattableItem::Complex(c) => c.re,
                FormattableItem::Text(t) => t.parse::<f64>().unwrap_or(0.0),
            };
            format!("{:.prec$}", x, prec = prec)
        }
        'e' => {
            let x = match item {
                FormattableItem::Num(x) => *x,
                FormattableItem::Complex(c) => c.re,
                FormattableItem::Text(t) => t.parse::<f64>().unwrap_or(0.0),
            };
            format!("{:.prec$e}", x, prec = prec)
        }
        'E' => {
            let x = match item {
                FormattableItem::Num(x) => *x,
                FormattableItem::Complex(c) => c.re,
                FormattableItem::Text(t) => t.parse::<f64>().unwrap_or(0.0),
            };
            format!("{:.prec$E}", x, prec = prec)
        }
        'g' | 'G' => {
            let x = match item {
                FormattableItem::Num(x) => *x,
                FormattableItem::Complex(c) => c.re,
                FormattableItem::Text(t) => t.parse::<f64>().unwrap_or(0.0),
            };
            let prec = precision.unwrap_or(4);
            format!("{:.prec$}", x, prec = prec)
        }
        'c' => match item {
            FormattableItem::Num(x) => {
                let ch = char::from_u32(*x as u32).unwrap_or(' ');
                ch.to_string()
            }
            FormattableItem::Complex(c) => {
                let ch = char::from_u32(c.re as u32).unwrap_or(' ');
                ch.to_string()
            }
            FormattableItem::Text(t) => t.chars().next().map(|c| c.to_string()).unwrap_or_default(),
        },
        's' => match item {
            FormattableItem::Text(t) => t.clone(),
            FormattableItem::Num(x) => format!("{x}"),
            FormattableItem::Complex(c) => format!("{c}"),
        },
        other => {
            format!("%{other}")
        }
    };

    if let Some(w) = width {
        if s.len() < w {
            let pad = " ".repeat(w - s.len());
            if flags.contains('-') {
                return format!("{s}{pad}");
            } else if flags.contains('0') && matches!(spec_type, 'd' | 'i' | 'u' | 'x' | 'X' | 'o')
            {
                let zero_pad = "0".repeat(w - s.len());
                return format!("{zero_pad}{s}");
            } else {
                return format!("{pad}{s}");
            }
        }
    }
    s
}

/// Applies MATLAB format recycling across all elements.
pub fn apply_format_recycling(fmt_str: &str, args: &[Value]) -> String {
    let tokens = parse_format_string(fmt_str);
    let spec_count = tokens
        .iter()
        .filter(|t| matches!(t, FormatToken::Specifier { .. }))
        .count();

    if spec_count == 0 {
        // Just literals
        let mut out = String::new();
        for t in tokens {
            if let FormatToken::Literal(s) = t {
                out.push_str(&s);
            }
        }
        return out;
    }

    // Check if multiple arguments are passed and match spec_count
    let mut flat_items = Vec::new();
    if args.len() == spec_count
        && args.iter().all(|a| a.numel() == args[0].numel())
        && !args.is_empty()
    {
        // Zip arguments element-by-element
        let num_elements = args[0].numel();
        let flattened_args: Vec<Vec<FormattableItem>> = args
            .iter()
            .map(|a| flatten_args(std::slice::from_ref(a)))
            .collect();
        for i in 0..num_elements {
            for arg_items in &flattened_args {
                if let Some(item) = arg_items.get(i) {
                    flat_items.push(item.clone());
                }
            }
        }
    } else {
        flat_items = flatten_args(args);
    }

    if flat_items.is_empty() {
        // Output leading literals until first specifier
        let mut out = String::new();
        for t in &tokens {
            match t {
                FormatToken::Literal(s) => out.push_str(s),
                FormatToken::Specifier { .. } => break,
            }
        }
        return out;
    }

    let mut out = String::new();
    let mut item_idx = 0;

    // Cycle through tokens until all items are consumed
    'outer: while item_idx < flat_items.len() {
        for token in &tokens {
            match token {
                FormatToken::Literal(s) => out.push_str(s),
                FormatToken::Specifier {
                    flags,
                    width,
                    precision,
                    spec_type,
                } => {
                    if let Some(item) = flat_items.get(item_idx) {
                        let formatted =
                            format_single_item(item, *spec_type, flags, *width, *precision);
                        out.push_str(&formatted);
                        item_idx += 1;
                    } else {
                        // Stop at first specifier with no remaining data
                        break 'outer;
                    }
                }
            }
        }
    }

    out
}

/// `sprintf(format, A, ...)`
pub fn sprintf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "sprintf requires at least 1 input.",
        ));
    }
    let fmt_str = extract_string(&args[0])?;
    let res = apply_format_recycling(&fmt_str, &args[1..]);
    Ok(vec![Value::char_vector(&res)])
}

/// `fprintf([fid], format, A, ...)`
pub fn fprintf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "fprintf requires at least 1 input.",
        ));
    }

    let (fid, fmt_arg_idx) = if args[0].is_scalar()
        && !matches!(args[0], Value::Char(_) | Value::String(_))
    {
        let id = args[0]
            .as_scalar_f64()
            .map_err(|e| user_err("MATLAB:badFID", &format!("{e}")))? as i64;
        (id, 1)
    } else {
        (1, 0)
    };

    if args.len() <= fmt_arg_idx {
        return Err(user_err("MATLAB:minrhs", "fprintf missing format string."));
    }

    let fmt_str = extract_string(&args[fmt_arg_idx])?;
    let res = apply_format_recycling(&fmt_str, &args[fmt_arg_idx + 1..]);

    match fid {
        1 => {
            print!("{res}");
            let _ = std::io::stdout().flush();
        }
        2 => {
            eprint!("{res}");
            let _ = std::io::stderr().flush();
        }
        _ => {
            // Unhandled file descriptor, default to stdout
            print!("{res}");
            let _ = std::io::stdout().flush();
        }
    }

    Ok(vec![Value::scalar(res.len() as f64)])
}

/// `disp(A)` prints the value to standard output.
pub fn disp_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "disp requires at least 1 input."));
    }
    match &args[0] {
        Value::Char(ca) => {
            let s: String = ca.as_slice().iter().collect();
            println!("{s}");
        }
        Value::String(sa) => {
            if sa.is_scalar() {
                let s = sa.as_slice().first().cloned().unwrap_or_default();
                println!("    \"{s}\"");
            } else {
                println!("{args_0}", args_0 = args[0]);
            }
        }
        other => {
            println!("{other}");
        }
    }
    Ok(Vec::new())
}

/// `num2str(A, [precision_or_format])`
pub fn num2str_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "num2str requires at least 1 input.",
        ));
    }

    if args.len() >= 2 {
        if let Ok(fmt) = extract_string(&args[1]) {
            if fmt.contains('%') {
                let res = apply_format_recycling(&fmt, std::slice::from_ref(&args[0]));
                return Ok(vec![Value::char_vector(&res)]);
            }
        } else if let Ok(prec) = args[1].as_scalar_f64() {
            let p = prec.max(1.0) as usize;
            let da = extract_f64_array(&args[0])?;
            if da.is_scalar() {
                let s = format!("{:.prec$}", da.as_slice()[0], prec = p);
                return Ok(vec![Value::char_vector(&s)]);
            }
        }
    }

    // Default formatting
    match &args[0] {
        Value::Double(da) => {
            if da.is_empty() {
                return Ok(vec![Value::char_vector("")]);
            }
            if da.is_scalar() {
                let x = da.as_slice()[0];
                let s = if x.fract() == 0.0 && x.abs() < 1e12 {
                    format!("{}", x as i64)
                } else {
                    format!("{x:.4}")
                };
                return Ok(vec![Value::char_vector(&s)]);
            }

            let rows = da.shape()[0];
            let cols = da.shape()[1];
            let mut col_widths = vec![0; cols];
            let mut col_entries: Vec<Vec<String>> = vec![vec![String::new(); rows]; cols];
            for c in 0..cols {
                for r in 0..rows {
                    let x = da.as_slice()[c * rows + r];
                    let s = if x.fract() == 0.0 && x.abs() < 1e12 {
                        format!("{}", x as i64)
                    } else {
                        format!("{x:.4}")
                    };
                    col_widths[c] = col_widths[c].max(s.len());
                    col_entries[c][r] = s;
                }
            }
            let mut row_strings = Vec::new();
            for r in 0..rows {
                let mut parts = Vec::new();
                for c in 0..cols {
                    let s = &col_entries[c][r];
                    let padded = format!("{:>width$}", s, width = col_widths[c]);
                    parts.push(padded);
                }
                row_strings.push(parts.join("  "));
            }
            let res = row_strings.join("\n");
            Ok(vec![Value::char_vector(&res)])
        }
        other => Ok(vec![Value::char_vector(&format!("{other}"))]),
    }
}

/// `mat2str(A, [precision])` formats 2D matrix into executable MATLAB syntax.
pub fn mat2str_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "mat2str requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Double(da) => {
            if da.is_empty() {
                return Ok(vec![Value::char_vector("[]")]);
            }
            if da.is_scalar() {
                return Ok(vec![Value::char_vector(&format!("{}", da.as_slice()[0]))]);
            }
            let rows = da.shape()[0];
            let cols = da.shape()[1];
            let mut row_strs = Vec::new();
            for r in 0..rows {
                let mut col_strs = Vec::new();
                for c in 0..cols {
                    let val = da.as_slice()[c * rows + r];
                    col_strs.push(format!("{val}"));
                }
                row_strs.push(col_strs.join(" "));
            }
            let res = format!("[{}]", row_strs.join("; "));
            Ok(vec![Value::char_vector(&res)])
        }
        Value::ComplexDouble(ca) => {
            if ca.is_empty() {
                return Ok(vec![Value::char_vector("[]")]);
            }
            let rows = ca.shape()[0];
            let cols = ca.shape()[1];
            let mut row_strs = Vec::new();
            for r in 0..rows {
                let mut col_strs = Vec::new();
                for c in 0..cols {
                    let val = ca.as_slice()[c * rows + r];
                    let sign_str = if val.im >= 0.0 {
                        format!("{}+{}i", val.re, val.im)
                    } else {
                        format!("{}-{}i", val.re, -val.im)
                    };
                    col_strs.push(sign_str);
                }
                row_strs.push(col_strs.join(" "));
            }
            let res = format!("[{}]", row_strs.join("; "));
            Ok(vec![Value::char_vector(&res)])
        }
        _ => Err(user_err(
            "MATLAB:mat2str:invalidInput",
            "mat2str input must be a numeric matrix.",
        )),
    }
}
