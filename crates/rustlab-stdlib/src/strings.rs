//! String and regular expression library: strsplit, strjoin, strrep, strtrim,
//! upper, lower, strcmp, strcmpi, strncmp, contains, startsWith, endsWith,
//! regexp, regexprep.

use crate::helpers::{extract_string, user_err};
use regex::Regex;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;

/// Helper to extract string from Value (char row vector or string scalar).
fn to_string_lossy(val: &Value) -> Result<String, EvalError> {
    extract_string(val)
}

/// Helper to parse optional case-insensitivity option: 'IgnoreCase', true/false.
fn parse_ignore_case(args: &[Value], start_idx: usize) -> bool {
    let mut i = start_idx;
    while i < args.len() {
        if let Ok(key) = extract_string(&args[i]) {
            if key.eq_ignore_ascii_case("ignorecase") && i + 1 < args.len() {
                if let Value::Logical(b) = &args[i + 1] {
                    return b.as_slice().first().copied().unwrap_or(false);
                } else if let Ok(d) = args[i + 1].as_scalar_f64() {
                    return d != 0.0;
                }
            }
        }
        i += 1;
    }
    false
}

/// `strsplit(str, [delimiter])` splits `str` by delimiter (default: whitespace).
/// Returns a 1 x K cell array of character vectors.
pub fn strsplit_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "strsplit requires at least 1 input.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let tokens: Vec<String> = if args.len() >= 2 {
        let delim = to_string_lossy(&args[1])?;
        text.split(&delim).map(|s| s.to_string()).collect()
    } else {
        text.split_whitespace().map(|s| s.to_string()).collect()
    };

    let cell_values: Vec<Value> = tokens.into_iter().map(|s| Value::char_vector(&s)).collect();
    let n = cell_values.len();
    let arr = DenseArray::from_vec(vec![1, n], cell_values).unwrap_or_else(|_| DenseArray::empty());
    Ok(vec![Value::Cell(arr)])
}

/// `strjoin(C, [delimiter])` joins cell array or string array with delimiter (default: `' '`).
pub fn strjoin_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "strjoin requires at least 1 input.",
        ));
    }
    let delim = if args.len() >= 2 {
        to_string_lossy(&args[1])?
    } else {
        " ".to_string()
    };

    let mut parts = Vec::new();
    match &args[0] {
        Value::Cell(cells) => {
            for v in cells.as_slice() {
                parts.push(to_string_lossy(v)?);
            }
        }
        Value::String(strs) => {
            for s in strs.as_slice() {
                parts.push(s.clone());
            }
        }
        Value::Char(_) => {
            parts.push(to_string_lossy(&args[0])?);
        }
        _ => {
            return Err(user_err(
                "MATLAB:strjoin:invalidInput",
                "strjoin requires cell array of character vectors or string array.",
            ));
        }
    }

    let joined = parts.join(&delim);
    Ok(vec![Value::char_vector(&joined)])
}

/// `strrep(str, old, new)` replaces all occurrences of `old` with `new`.
pub fn strrep_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err("MATLAB:minrhs", "strrep requires 3 inputs."));
    }
    let text = to_string_lossy(&args[0])?;
    let old = to_string_lossy(&args[1])?;
    let new = to_string_lossy(&args[2])?;
    let res = text.replace(&old, &new);
    Ok(vec![Value::char_vector(&res)])
}

/// `strtrim(str)` removes leading and trailing whitespace.
pub fn strtrim_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "strtrim requires at least 1 input.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let trimmed = text.trim();
    Ok(vec![Value::char_vector(trimmed)])
}

/// `upper(str)` converts characters to uppercase.
pub fn upper_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "upper requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Char(c) => {
            let u_chars: Vec<char> = c
                .as_slice()
                .iter()
                .map(|ch| ch.to_ascii_uppercase())
                .collect();
            let arr = DenseArray::from_vec_unchecked(c.shape().to_vec(), u_chars);
            Ok(vec![Value::Char(arr)])
        }
        Value::String(s) => {
            let u_strs: Vec<String> = s.as_slice().iter().map(|st| st.to_uppercase()).collect();
            let arr = DenseArray::from_vec_unchecked(s.shape().to_vec(), u_strs);
            Ok(vec![Value::String(arr)])
        }
        _ => {
            let text = to_string_lossy(&args[0])?;
            Ok(vec![Value::char_vector(&text.to_uppercase())])
        }
    }
}

/// `lower(str)` converts characters to lowercase.
pub fn lower_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "lower requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Char(c) => {
            let l_chars: Vec<char> = c
                .as_slice()
                .iter()
                .map(|ch| ch.to_ascii_lowercase())
                .collect();
            let arr = DenseArray::from_vec_unchecked(c.shape().to_vec(), l_chars);
            Ok(vec![Value::Char(arr)])
        }
        Value::String(s) => {
            let l_strs: Vec<String> = s.as_slice().iter().map(|st| st.to_lowercase()).collect();
            let arr = DenseArray::from_vec_unchecked(s.shape().to_vec(), l_strs);
            Ok(vec![Value::String(arr)])
        }
        _ => {
            let text = to_string_lossy(&args[0])?;
            Ok(vec![Value::char_vector(&text.to_lowercase())])
        }
    }
}

/// `strcmp(s1, s2)` compares two strings or character vectors for exact equality.
pub fn strcmp_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "strcmp requires 2 inputs."));
    }
    let s1 = to_string_lossy(&args[0])?;
    let s2 = to_string_lossy(&args[1])?;
    Ok(vec![Value::logical_scalar(s1 == s2)])
}

/// `strcmpi(s1, s2)` case-insensitive string comparison.
pub fn strcmpi_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "strcmpi requires 2 inputs."));
    }
    let s1 = to_string_lossy(&args[0])?;
    let s2 = to_string_lossy(&args[1])?;
    Ok(vec![Value::logical_scalar(s1.eq_ignore_ascii_case(&s2))])
}

/// `strncmp(s1, s2, n)` compares the first `n` characters of two strings.
pub fn strncmp_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err("MATLAB:minrhs", "strncmp requires 3 inputs."));
    }
    let s1 = to_string_lossy(&args[0])?;
    let s2 = to_string_lossy(&args[1])?;
    let n = args[2]
        .as_scalar_f64()
        .map_err(|e| user_err("MATLAB:strncmp:badN", &format!("{e}")))? as usize;

    let sub1: String = s1.chars().take(n).collect();
    let sub2: String = s2.chars().take(n).collect();
    Ok(vec![Value::logical_scalar(sub1 == sub2)])
}

/// `contains(str, pattern, ['IgnoreCase', true/false])`
pub fn contains_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "contains requires at least 2 inputs.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let pat = to_string_lossy(&args[1])?;
    let ignore_case = parse_ignore_case(args, 2);

    let res = if ignore_case {
        text.to_lowercase().contains(&pat.to_lowercase())
    } else {
        text.contains(&pat)
    };
    Ok(vec![Value::logical_scalar(res)])
}

/// `startsWith(str, pattern, ['IgnoreCase', true/false])`
pub fn starts_with_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "startsWith requires at least 2 inputs.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let pat = to_string_lossy(&args[1])?;
    let ignore_case = parse_ignore_case(args, 2);

    let res = if ignore_case {
        text.to_lowercase().starts_with(&pat.to_lowercase())
    } else {
        text.starts_with(&pat)
    };
    Ok(vec![Value::logical_scalar(res)])
}

/// `endsWith(str, pattern, ['IgnoreCase', true/false])`
pub fn ends_with_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "endsWith requires at least 2 inputs.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let pat = to_string_lossy(&args[1])?;
    let ignore_case = parse_ignore_case(args, 2);

    let res = if ignore_case {
        text.to_lowercase().ends_with(&pat.to_lowercase())
    } else {
        text.ends_with(&pat)
    };
    Ok(vec![Value::logical_scalar(res)])
}

/// `regexp(str, expression, ...)`
/// Supported flags:
/// - `'match'`: returns cell array of matched substrings
/// - `'tokens'`: returns cell array of captured group substrings
/// - `'names'`: returns struct array with named capture fields
/// - `'once'`: returns only first match
///
/// Default: returns 1-based start indices of matches as a double row vector.
pub fn regexp_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "regexp requires at least 2 inputs.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let pattern_str = to_string_lossy(&args[1])?;

    let re = Regex::new(&pattern_str).map_err(|e| {
        user_err(
            "MATLAB:regexp:invalidPattern",
            &format!("Invalid regex pattern: {e}"),
        )
    })?;

    // Parse options
    let mut mode_match = false;
    let mut mode_tokens = false;
    let mut mode_names = false;
    let mut mode_once = false;

    for arg in &args[2..] {
        if let Ok(opt) = extract_string(arg) {
            match opt.to_lowercase().as_str() {
                "match" => mode_match = true,
                "tokens" => mode_tokens = true,
                "names" => mode_names = true,
                "once" => mode_once = true,
                _ => {}
            }
        }
    }

    if mode_names {
        // Struct array with named capture groups
        let mut field_names: Vec<String> = re
            .capture_names()
            .flatten()
            .map(|s| s.to_string())
            .collect();
        field_names.sort();

        let matches: Vec<_> = if mode_once {
            re.captures(&text).into_iter().collect()
        } else {
            re.captures_iter(&text).collect()
        };

        if matches.is_empty() {
            return Ok(vec![Value::Struct(StructArray::empty())]);
        }

        let mut data = Vec::with_capacity(matches.len());
        for cap in &matches {
            let mut map = HashMap::new();
            for field in &field_names {
                let val_str = cap.name(field).map(|m| m.as_str()).unwrap_or("");
                map.insert(field.clone(), Value::char_vector(val_str));
            }
            data.push(map);
        }

        let st = StructArray::new(vec![1, matches.len()], field_names, data);
        return Ok(vec![Value::Struct(st)]);
    }

    if mode_tokens {
        let mut token_cells = Vec::new();
        let captures: Vec<_> = if mode_once {
            re.captures(&text).into_iter().collect()
        } else {
            re.captures_iter(&text).collect()
        };

        for cap in captures {
            let mut groups = Vec::new();
            for i in 1..cap.len() {
                if let Some(m) = cap.get(i) {
                    groups.push(Value::char_vector(m.as_str()));
                }
            }
            let group_cell = DenseArray::from_vec(vec![1, groups.len()], groups)
                .unwrap_or_else(|_| DenseArray::empty());
            token_cells.push(Value::Cell(group_cell));
        }

        let n = token_cells.len();
        let arr =
            DenseArray::from_vec(vec![1, n], token_cells).unwrap_or_else(|_| DenseArray::empty());
        return Ok(vec![Value::Cell(arr)]);
    }

    if mode_match {
        let matches: Vec<_> = if mode_once {
            re.find(&text).into_iter().collect()
        } else {
            re.find_iter(&text).collect()
        };

        let cell_vals: Vec<Value> = matches
            .into_iter()
            .map(|m| Value::char_vector(m.as_str()))
            .collect();
        let n = cell_vals.len();
        let arr =
            DenseArray::from_vec(vec![1, n], cell_vals).unwrap_or_else(|_| DenseArray::empty());
        return Ok(vec![Value::Cell(arr)]);
    }

    // Default: 1-based start indices
    let indices: Vec<f64> = if mode_once {
        re.find(&text)
            .map(|m| (m.start() + 1) as f64)
            .into_iter()
            .collect()
    } else {
        re.find_iter(&text)
            .map(|m| (m.start() + 1) as f64)
            .collect()
    };

    let n = indices.len();
    let arr = DenseArray::from_vec(vec![1, n], indices).unwrap_or_else(|_| DenseArray::empty());
    Ok(vec![Value::Double(arr)])
}

/// `regexprep(str, expression, replace, ...)`
pub fn regexprep_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "regexprep requires at least 3 inputs.",
        ));
    }
    let text = to_string_lossy(&args[0])?;
    let pattern_str = to_string_lossy(&args[1])?;
    let rep = to_string_lossy(&args[2])?;

    let re = Regex::new(&pattern_str).map_err(|e| {
        user_err(
            "MATLAB:regexprep:invalidPattern",
            &format!("Invalid regex pattern: {e}"),
        )
    })?;

    let mut mode_once = false;
    for arg in &args[3..] {
        if let Ok(opt) = extract_string(arg) {
            if opt.eq_ignore_ascii_case("once") {
                mode_once = true;
            }
        }
    }

    let replaced = if mode_once {
        re.replace(&text, rep.as_str()).to_string()
    } else {
        re.replace_all(&text, rep.as_str()).to_string()
    };

    Ok(vec![Value::char_vector(&replaced)])
}
