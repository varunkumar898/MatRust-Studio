//! Heterogeneous containers and string utilities: cell, struct, isfield, fieldnames, string, char, strlength.

use crate::helpers::{parse_dims, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;

pub fn cell_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let arr = DenseArray::from_vec_unchecked(dims, vec![Value::empty(); count]);
    Ok(vec![Value::Cell(arr)])
}

pub fn struct_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if !args.len().is_multiple_of(2) {
        return Err(user_err(
            "MATLAB:struct:unevenArgs",
            "struct arguments must occur in name-value pairs.",
        ));
    }

    let mut fields = Vec::with_capacity(args.len() / 2);
    let mut map = HashMap::new();

    let mut i = 0;
    while i < args.len() {
        let name = match &args[i] {
            Value::String(s) if s.is_scalar() => s.as_slice()[0].clone(),
            Value::Char(c) => c.as_slice().iter().collect(),
            _ => {
                return Err(user_err(
                    "MATLAB:struct:badName",
                    "Field name must be a text scalar or character vector.",
                ))
            }
        };
        let val = args[i + 1].clone();
        fields.push(name.clone());
        map.insert(name, val);
        i += 2;
    }

    let s = StructArray {
        shape: vec![1, 1],
        fields,
        data: vec![map],
    };
    Ok(vec![Value::Struct(s)])
}

pub fn isfield_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "isfield requires 2 inputs."));
    }
    let field = match &args[1] {
        Value::String(s) if s.is_scalar() => s.as_slice()[0].clone(),
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => return Ok(vec![Value::logical_scalar(false)]),
    };

    match &args[0] {
        Value::Struct(s) => {
            let has = s.get_field(&field).is_some();
            Ok(vec![Value::logical_scalar(has)])
        }
        _ => Ok(vec![Value::logical_scalar(false)]),
    }
}

pub fn fieldnames_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "fieldnames requires 1 input."));
    }
    match &args[0] {
        Value::Struct(s) => {
            let names: Vec<Value> = s.fields.iter().map(|f| Value::char_vector(f)).collect();
            let n = names.len();
            let arr = DenseArray::from_vec_unchecked(vec![n, 1], names);
            Ok(vec![Value::Cell(arr)])
        }
        _ => Err(user_err(
            "MATLAB:fieldnames:notStruct",
            "Input must be a structure array.",
        )),
    }
}

pub fn string_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Ok(vec![Value::string_scalar("")]);
    }
    match &args[0] {
        Value::String(_) => Ok(vec![args[0].clone()]),
        Value::Char(c) => {
            let text: String = c.as_slice().iter().collect();
            Ok(vec![Value::string_scalar(&text)])
        }
        Value::Double(d) => {
            let strs: Vec<String> = d.as_slice().iter().map(|x| format!("{}", x)).collect();
            let arr = DenseArray::from_vec_unchecked(d.shape().to_vec(), strs);
            Ok(vec![Value::String(arr)])
        }
        v => Ok(vec![Value::string_scalar(&format!("{}", v))]),
    }
}

pub fn char_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Ok(vec![Value::char_vector("")]);
    }
    match &args[0] {
        Value::Char(_) => Ok(vec![args[0].clone()]),
        Value::String(s) => {
            let text = s.as_slice().first().cloned().unwrap_or_default();
            Ok(vec![Value::char_vector(&text)])
        }
        Value::Double(d) => {
            let chars: Vec<char> = d
                .as_slice()
                .iter()
                .map(|&x| char::from_u32(x as u32).unwrap_or(' '))
                .collect();
            let arr = DenseArray::from_vec_unchecked(d.shape().to_vec(), chars);
            Ok(vec![Value::Char(arr)])
        }
        _ => Ok(vec![Value::char_vector("")]),
    }
}

pub fn strlength_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "strlength requires 1 input."));
    }
    match &args[0] {
        Value::String(s) => {
            let lengths: Vec<f64> = s.as_slice().iter().map(|str| str.len() as f64).collect();
            let arr = DenseArray::from_vec_unchecked(s.shape().to_vec(), lengths);
            Ok(vec![Value::Double(arr)])
        }
        Value::Char(c) => Ok(vec![Value::scalar(c.numel() as f64)]),
        _ => Err(user_err(
            "MATLAB:strlength:invalidInput",
            "Input must be string or char array.",
        )),
    }
}
