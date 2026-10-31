use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{StructArray, Value};
use std::collections::HashMap;

/// `S = load(filename)` loads a .mat file and returns a struct containing the variables.
pub fn load(args: &[Value], _nouts: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "load requires at least 1 input."));
    }
    let filename = match &args[0] {
        Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
        Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
        _ => {
            return Err(user_err(
                "MATLAB:load:invalidInput",
                "load: first argument must be a string or char vector",
            ))
        }
    };

    let vars = rustlab_matio::load_mat(&filename)
        .map_err(|e| user_err("MATLAB:load:ioError", &e.to_string()))?;

    let mut fields = Vec::new();
    let mut map = HashMap::new();

    for (k, v) in vars {
        fields.push(k.clone());
        map.insert(k, v);
    }

    let struct_arr = StructArray {
        shape: vec![1, 1],
        fields,
        data: vec![map],
    };

    Ok(vec![Value::Struct(struct_arr)])
}

/// `save(filename, S)` saves the fields in struct `S` into a .mat file.
pub fn save(args: &[Value], _nouts: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "save requires at least 2 inputs.",
        ));
    }
    let filename = match &args[0] {
        Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
        Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
        _ => {
            return Err(user_err(
                "MATLAB:save:invalidInput",
                "save: first argument must be a string or char vector",
            ))
        }
    };

    let mut vars = HashMap::new();
    match &args[1] {
        Value::Struct(s) => {
            if let Some(first_map) = s.data.first() {
                for (k, v) in first_map {
                    vars.insert(k.clone(), v.clone());
                }
            }
        }
        _ => {
            return Err(user_err(
                "MATLAB:save:invalidInput",
                "save: second argument must be a struct containing variables to save",
            ))
        }
    }

    rustlab_matio::save_mat(&filename, &vars, true)
        .map_err(|e| user_err("MATLAB:save:ioError", &e.to_string()))?;

    Ok(vec![Value::empty()])
}
