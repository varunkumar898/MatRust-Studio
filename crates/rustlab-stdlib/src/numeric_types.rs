//! Numeric type constructors, predicates, limits, integer division, and bitwise operations.

use crate::helpers::{math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

/// `single(X)` converts array to single-precision float.
pub fn single(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "single requires 1 input."));
    }
    let val = args[0].cast_to("single").map_err(math_err)?;
    Ok(vec![val])
}

/// `double(X)` converts array to double-precision float.
pub fn double(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "double requires 1 input."));
    }
    let val = args[0].cast_to("double").map_err(math_err)?;
    Ok(vec![val])
}

/// `int8(X)` converts array to signed 8-bit integer with saturating rounding.
pub fn int8(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "int8 requires 1 input."));
    }
    let val = args[0].to_int8().map_err(math_err)?;
    Ok(vec![val])
}

/// `int16(X)` converts array to signed 16-bit integer with saturating rounding.
pub fn int16(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "int16 requires 1 input."));
    }
    let val = args[0].to_int16().map_err(math_err)?;
    Ok(vec![val])
}

/// `int32(X)` converts array to signed 32-bit integer with saturating rounding.
pub fn int32(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "int32 requires 1 input."));
    }
    let val = args[0].to_int32().map_err(math_err)?;
    Ok(vec![val])
}

/// `int64(X)` converts array to signed 64-bit integer with saturating rounding.
pub fn int64(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "int64 requires 1 input."));
    }
    let val = args[0].to_int64().map_err(math_err)?;
    Ok(vec![val])
}

/// `uint8(X)` converts array to unsigned 8-bit integer with saturating rounding.
pub fn uint8(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "uint8 requires 1 input."));
    }
    let val = args[0].to_uint8().map_err(math_err)?;
    Ok(vec![val])
}

/// `uint16(X)` converts array to unsigned 16-bit integer with saturating rounding.
pub fn uint16(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "uint16 requires 1 input."));
    }
    let val = args[0].to_uint16().map_err(math_err)?;
    Ok(vec![val])
}

/// `uint32(X)` converts array to unsigned 32-bit integer with saturating rounding.
pub fn uint32(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "uint32 requires 1 input."));
    }
    let val = args[0].to_uint32().map_err(math_err)?;
    Ok(vec![val])
}

/// `uint64(X)` converts array to unsigned 64-bit integer with saturating rounding.
pub fn uint64(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "uint64 requires 1 input."));
    }
    let val = args[0].to_uint64().map_err(math_err)?;
    Ok(vec![val])
}

/// `logical(X)` converts `X` to logical array.
pub fn logical(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "logical requires 1 input."));
    }
    let val = args[0].cast_to("logical").map_err(math_err)?;
    Ok(vec![val])
}

/// `cast(X, newclass)` casts `X` to the named MATLAB class.
pub fn cast(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "cast requires 2 inputs."));
    }
    let target = extract_string(&args[1])?;
    let val = args[0].cast_to(&target).map_err(math_err)?;
    Ok(vec![val])
}

/// `class(X)` returns the class name of `X`.
pub fn class(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "class requires 1 input."));
    }
    let c = args[0].type_name();
    Ok(vec![Value::char_vector(c)])
}

/// `isa(X, class_name)` checks whether `X` is an instance of `class_name`.
pub fn isa(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "isa requires 2 inputs."));
    }
    let target = extract_string(&args[1])?;
    let is_match = match target.as_str() {
        "integer" => args[0].is_integer(),
        "float" => args[0].is_float(),
        "numeric" => args[0].is_numeric(),
        "sparse" => args[0].is_sparse(),
        other => args[0].type_name() == other,
    };
    Ok(vec![Value::logical_scalar(is_match)])
}

/// `isinteger(X)` returns true if `X` is an integer class.
pub fn isinteger(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "isinteger requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].is_integer())])
}

/// `isfloat(X)` returns true if `X` is a float class (double or single).
pub fn isfloat(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "isfloat requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].is_float())])
}

/// `isnumeric(X)` returns true if `X` is a numeric class.
pub fn isnumeric(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "isnumeric requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].is_numeric())])
}

/// `issparse(X)` returns true if `X` is a sparse matrix.
pub fn issparse(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "issparse requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].is_sparse())])
}

/// `intmax([type])` returns the largest value of integer class.
pub fn intmax(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let type_name = if args.is_empty() {
        "int32".to_string()
    } else {
        extract_string(&args[0])?
    };

    let res = match type_name.as_str() {
        "int8" => Value::Int8(DenseArray::scalar(i8::MAX)),
        "int16" => Value::Int16(DenseArray::scalar(i16::MAX)),
        "int32" => Value::Int32(DenseArray::scalar(i32::MAX)),
        "int64" => Value::Int64(DenseArray::scalar(i64::MAX)),
        "uint8" => Value::UInt8(DenseArray::scalar(u8::MAX)),
        "uint16" => Value::UInt16(DenseArray::scalar(u16::MAX)),
        "uint32" => Value::UInt32(DenseArray::scalar(u32::MAX)),
        "uint64" => Value::UInt64(DenseArray::scalar(u64::MAX)),
        _ => {
            return Err(user_err(
                "MATLAB:intmax:invalidType",
                "Invalid integer type.",
            ))
        }
    };
    Ok(vec![res])
}

/// `intmin([type])` returns the smallest value of integer class.
pub fn intmin(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let type_name = if args.is_empty() {
        "int32".to_string()
    } else {
        extract_string(&args[0])?
    };

    let res = match type_name.as_str() {
        "int8" => Value::Int8(DenseArray::scalar(i8::MIN)),
        "int16" => Value::Int16(DenseArray::scalar(i16::MIN)),
        "int32" => Value::Int32(DenseArray::scalar(i32::MIN)),
        "int64" => Value::Int64(DenseArray::scalar(i64::MIN)),
        "uint8" => Value::UInt8(DenseArray::scalar(u8::MIN)),
        "uint16" => Value::UInt16(DenseArray::scalar(u16::MIN)),
        "uint32" => Value::UInt32(DenseArray::scalar(u32::MIN)),
        "uint64" => Value::UInt64(DenseArray::scalar(u64::MIN)),
        _ => {
            return Err(user_err(
                "MATLAB:intmin:invalidType",
                "Invalid integer type.",
            ))
        }
    };
    Ok(vec![res])
}

/// `realmax([type])` returns largest finite floating point number.
pub fn realmax(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let type_name = if args.is_empty() {
        "double".to_string()
    } else {
        extract_string(&args[0])?
    };

    match type_name.as_str() {
        "single" => Ok(vec![Value::Single(DenseArray::scalar(f32::MAX))]),
        "double" => Ok(vec![Value::scalar(f64::MAX)]),
        _ => Err(user_err(
            "MATLAB:realmax:invalidType",
            "Type must be 'double' or 'single'.",
        )),
    }
}

/// `realmin([type])` returns smallest positive normalized floating point number.
pub fn realmin(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let type_name = if args.is_empty() {
        "double".to_string()
    } else {
        extract_string(&args[0])?
    };

    match type_name.as_str() {
        "single" => Ok(vec![Value::Single(DenseArray::scalar(f32::MIN_POSITIVE))]),
        "double" => Ok(vec![Value::scalar(f64::MIN_POSITIVE)]),
        _ => Err(user_err(
            "MATLAB:realmin:invalidType",
            "Type must be 'double' or 'single'.",
        )),
    }
}

/// `flintmax([type])` returns largest consecutive integer in float format ($2^{53}$ or $2^{24}$).
pub fn flintmax(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let type_name = if args.is_empty() {
        "double".to_string()
    } else {
        extract_string(&args[0])?
    };

    match type_name.as_str() {
        "single" => Ok(vec![Value::Single(DenseArray::scalar(16777216.0f32))]), // 2^24
        "double" => Ok(vec![Value::scalar(9007199254740992.0f64)]),             // 2^53
        _ => Err(user_err(
            "MATLAB:flintmax:invalidType",
            "Type must be 'double' or 'single'.",
        )),
    }
}

/// `eps([X])` returns floating point relative accuracy.
pub fn eps(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Ok(vec![Value::scalar(f64::EPSILON)]);
    }

    if let Ok(s) = extract_string(&args[0]) {
        match s.as_str() {
            "single" => return Ok(vec![Value::Single(DenseArray::scalar(f32::EPSILON))]),
            "double" => return Ok(vec![Value::scalar(f64::EPSILON)]),
            _ => {}
        }
    }

    let da = args[0].to_double_array().map_err(math_err)?;
    let out = da.map(|&x| {
        let abs_x = x.abs();
        if abs_x == 0.0 {
            // Distance from 0 to next subnormal double
            f64::from_bits(1)
        } else {
            let exp = abs_x.log2().floor();
            2.0f64.powf(exp - 52.0)
        }
    });
    Ok(vec![Value::Double(out)])
}

/// `idivide(A, B [, opt])` integer division with 'fix', 'floor', 'ceil', or 'round'.
pub fn idivide(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "idivide requires at least 2 inputs.",
        ));
    }
    let mode = if args.len() >= 3 {
        extract_string(&args[2])?
    } else {
        "fix".to_string()
    };

    let res = args[0].idivide(&args[1], &mode).map_err(math_err)?;
    Ok(vec![res])
}

/// `bitand(A, B)` bitwise AND.
pub fn bitand(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "bitand requires 2 inputs."));
    }
    let res = args[0].bitand(&args[1]).map_err(math_err)?;
    Ok(vec![res])
}

/// `bitor(A, B)` bitwise OR.
pub fn bitor(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "bitor requires 2 inputs."));
    }
    let res = args[0].bitor(&args[1]).map_err(math_err)?;
    Ok(vec![res])
}

/// `bitxor(A, B)` bitwise XOR.
pub fn bitxor(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "bitxor requires 2 inputs."));
    }
    let res = args[0].bitxor(&args[1]).map_err(math_err)?;
    Ok(vec![res])
}

/// `bitshift(A, K)` bitwise shift.
pub fn bitshift(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "bitshift requires 2 inputs."));
    }
    let res = args[0].bitshift(&args[1]).map_err(math_err)?;
    Ok(vec![res])
}

fn extract_string(val: &Value) -> Result<String, EvalError> {
    match val {
        Value::Char(ca) => Ok(ca.as_slice().iter().collect()),
        Value::String(sa) => {
            if !sa.is_empty() {
                Ok(sa.as_slice()[0].clone())
            } else {
                Ok(String::new())
            }
        }
        _ => Err(user_err(
            "MATLAB:invalidType",
            "Expected a string or character vector.",
        )),
    }
}
