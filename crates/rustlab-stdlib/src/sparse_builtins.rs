//! Compressed Sparse Column (CSC) matrix operations: sparse, full, speye, nnz, nonzeros, spones.

use crate::helpers::{math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{CscMatrix, DenseArray, Value};

/// `sparse(...)` constructs or converts to a CSC sparse matrix.
/// - `sparse(A)`: converts dense matrix to sparse.
/// - `sparse(m, n)`: constructs empty $m \times n$ sparse matrix.
/// - `sparse(i, j, v [, m, n])`: constructs from 1-based coordinates with duplicate accumulation.
pub fn sparse(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Ok(vec![Value::Sparse(CscMatrix::new(0, 0))]);
    }

    if args.len() == 1 {
        // sparse(A)
        match &args[0] {
            Value::Sparse(s) => Ok(vec![Value::Sparse(s.clone())]),
            other => {
                let da = other.to_double_array().map_err(math_err)?;
                let sp = CscMatrix::from_dense(&da).map_err(math_err)?;
                Ok(vec![Value::Sparse(sp)])
            }
        }
    } else if args.len() == 2 {
        // sparse(m, n) -> all-zeros sparse matrix
        let m = args[0].as_scalar_f64().map_err(math_err)? as usize;
        let n = args[1].as_scalar_f64().map_err(math_err)? as usize;
        let sp = CscMatrix::from_triplets_1based(m, n, &[], &[], &[]).map_err(math_err)?;
        Ok(vec![Value::Sparse(sp)])
    } else if args.len() >= 3 {
        // sparse(i, j, v [, m, n])
        let i_arr = args[0].to_double_array().map_err(math_err)?;
        let j_arr = args[1].to_double_array().map_err(math_err)?;
        let v_arr = args[2].to_double_array().map_err(math_err)?;

        let num_triplets = i_arr.numel();
        if j_arr.numel() != num_triplets || v_arr.numel() != num_triplets {
            return Err(user_err(
                "MATLAB:sparse:sizeMismatch",
                "Indices i, j and values v must have the same number of elements.",
            ));
        }

        let i_vec: Vec<usize> = i_arr.as_slice().iter().map(|&x| x as usize).collect();
        let j_vec: Vec<usize> = j_arr.as_slice().iter().map(|&x| x as usize).collect();
        let v_vec: &[f64] = v_arr.as_slice();

        let (m, n) = if args.len() >= 5 {
            let m_val = args[3].as_scalar_f64().map_err(math_err)? as usize;
            let n_val = args[4].as_scalar_f64().map_err(math_err)? as usize;
            (m_val, n_val)
        } else {
            let m_max = i_vec.iter().copied().max().unwrap_or(0);
            let n_max = j_vec.iter().copied().max().unwrap_or(0);
            (m_max, n_max)
        };

        let sp = CscMatrix::from_triplets_1based(m, n, &i_vec, &j_vec, v_vec).map_err(math_err)?;
        Ok(vec![Value::Sparse(sp)])
    } else {
        Err(user_err(
            "MATLAB:sparse:invalidNumArgs",
            "sparse requires 1, 2, 3, or 5 arguments.",
        ))
    }
}

/// `full(S)` converts a sparse matrix into a dense 2D double matrix.
pub fn full(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "full requires 1 input."));
    }
    match &args[0] {
        Value::Sparse(s) => Ok(vec![Value::Double(s.to_dense())]),
        other => Ok(vec![other.clone()]),
    }
}

/// `speye(m [, n])` constructs a sparse identity matrix.
pub fn speye(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Ok(vec![Value::Sparse(CscMatrix::speye(1, 1))]);
    }
    let m = args[0].as_scalar_f64().map_err(math_err)? as usize;
    let n = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        m
    };
    Ok(vec![Value::Sparse(CscMatrix::speye(m, n))])
}

/// `nnz(S)` returns the number of non-zero elements.
pub fn nnz(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "nnz requires 1 input."));
    }
    match &args[0] {
        Value::Sparse(s) => Ok(vec![Value::scalar(s.nnz() as f64)]),
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            let count = da.as_slice().iter().filter(|&&x| x != 0.0).count();
            Ok(vec![Value::scalar(count as f64)])
        }
    }
}

/// `nonzeros(S)` returns a column vector of all non-zero elements.
pub fn nonzeros(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "nonzeros requires 1 input."));
    }
    match &args[0] {
        Value::Sparse(s) => {
            let vals = s.values.clone();
            let arr = DenseArray::col_vector(vals);
            Ok(vec![Value::Double(arr)])
        }
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            let vals: Vec<f64> = da
                .as_slice()
                .iter()
                .copied()
                .filter(|&x| x != 0.0)
                .collect();
            let arr = DenseArray::col_vector(vals);
            Ok(vec![Value::Double(arr)])
        }
    }
}

/// `spones(S)` returns a sparse matrix with 1 in non-zero locations, preserving sparsity.
pub fn spones(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "spones requires 1 input."));
    }
    match &args[0] {
        Value::Sparse(s) => Ok(vec![Value::Sparse(s.spones())]),
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            let sp = CscMatrix::from_dense(&da).map_err(math_err)?;
            Ok(vec![Value::Sparse(sp.spones())])
        }
    }
}
