//! Linear algebra routines: inv, det, trace, norm, rank, mldivide, mrdivide, lu, qr, chol, eig, svd, pinv, expm, cond, null, orth, kron.

pub mod eig_sym;
pub mod expm;
pub mod kron;
pub mod qr_pivoted;
pub mod svd;

pub use eig_sym::eig_sym;
pub use expm::expm;
pub use kron::kron_fn;
pub use qr_pivoted::qr_pivoted_fn as qr_fn;
pub use svd::{cond_fn, eig_fn, null_fn, orth_fn, pinv_fn, rank_fn, svd_fn};

use crate::helpers::{extract_f64_array, math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

/// Computes the matrix exponential: expm(A).
pub fn expm_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "expm requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let res = expm::expm(&da)?;
    Ok(vec![Value::Double(res)])
}

pub fn trace_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "trace requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:trace:notSquare", "Matrix must be square."));
    }
    let n = shp[0];
    let mut sum = 0.0;
    for i in 1..=n {
        sum += *da.get_subscript(&[i, i])?;
    }
    Ok(vec![Value::scalar(sum)])
}

/// Solves an N x N linear system Ax = b via Gauss-Jordan elimination with partial pivoting.
pub fn mldivide_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "\\ requires 2 inputs: A \\ b."));
    }
    let a_arr = extract_f64_array(&args[0])?;
    let b_arr = extract_f64_array(&args[1])?;

    let a_shp = a_arr.shape();
    let b_shp = b_arr.shape();

    if a_shp.len() != 2 || a_shp[0] != a_shp[1] {
        return Err(user_err(
            "MATLAB:dimagree",
            "Left division currently requires square coefficient matrix A.",
        ));
    }
    let n = a_shp[0];
    if b_shp[0] != n {
        return Err(user_err(
            "MATLAB:dimagree",
            "Matrix dimensions must agree in A \\ b.",
        ));
    }
    let n_rhs = b_shp[1];

    // Build augmented matrix [A | B] of shape [N, N + n_rhs] in row-major for convenient row ops
    let total_cols = n + n_rhs;
    let mut aug = vec![0.0; n * total_cols];

    for r in 0..n {
        for c in 0..n {
            aug[r * total_cols + c] = *a_arr.get_subscript(&[r + 1, c + 1])?;
        }
        for c in 0..n_rhs {
            aug[r * total_cols + n + c] = *b_arr.get_subscript(&[r + 1, c + 1])?;
        }
    }

    // Forward and back elimination
    for col in 0..n {
        // Find pivot
        let mut pivot_row = col;
        let mut max_val = aug[col * total_cols + col].abs();
        for r in (col + 1)..n {
            let val = aug[r * total_cols + col].abs();
            if val > max_val {
                max_val = val;
                pivot_row = r;
            }
        }

        if max_val < 1e-15 {
            return Err(user_err(
                "MATLAB:singularMatrix",
                "Matrix is singular to working precision.",
            ));
        }

        // Swap rows
        if pivot_row != col {
            for c in 0..total_cols {
                aug.swap(col * total_cols + c, pivot_row * total_cols + c);
            }
        }

        // Normalize pivot row
        let pivot = aug[col * total_cols + col];
        for c in col..total_cols {
            aug[col * total_cols + c] /= pivot;
        }

        // Eliminate column entries in other rows
        for r in 0..n {
            if r != col {
                let factor = aug[r * total_cols + col];
                if factor != 0.0 {
                    for c in col..total_cols {
                        let sub = factor * aug[col * total_cols + c];
                        aug[r * total_cols + c] -= sub;
                    }
                }
            }
        }
    }

    // Extract solution X [N, n_rhs] in col-major
    let mut x_data = vec![0.0; n * n_rhs];
    for c in 0..n_rhs {
        for r in 0..n {
            x_data[r + c * n] = aug[r * total_cols + n + c];
        }
    }

    let out = DenseArray::from_vec(vec![n, n_rhs], x_data).map_err(math_err)?;
    Ok(vec![Value::Double(out)])
}

pub fn mrdivide_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "/ requires 2 inputs: b / A."));
    }
    // b / A == (A' \ b')'
    let b_t = args[0].transpose().map_err(math_err)?;
    let a_t = args[1].transpose().map_err(math_err)?;

    let sol = mldivide_fn(&[a_t, b_t], 1)?;
    let sol_t = sol[0].transpose().map_err(math_err)?;
    Ok(vec![sol_t])
}

pub fn inv_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "inv requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:inv:notSquare", "Matrix must be square."));
    }
    let n = shp[0];
    let eye_val = crate::generators::eye(&[Value::scalar(n as f64)], 1)?;
    match mldivide_fn(&[args[0].clone(), eye_val[0].clone()], 1) {
        Ok(v) => Ok(v),
        Err(e) if e.to_string().contains("singularMatrix") => {
            eprintln!("Warning: Matrix is singular to working precision.");
            let inf_matrix = DenseArray::from_vec(vec![n, n], vec![f64::INFINITY; n * n])?;
            Ok(vec![Value::Double(inf_matrix)])
        }
        Err(e) => Err(e),
    }
}

/// Helper to compute LU decomposition with partial pivoting: PA = LU.
/// Returns (L, U, P, num_swaps).
fn compute_lu(
    da: &DenseArray<f64>,
) -> Result<(DenseArray<f64>, DenseArray<f64>, DenseArray<f64>, usize), EvalError> {
    let shp = da.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:lu:notSquare", "Matrix must be square."));
    }
    let n = shp[0];

    let mut a = vec![0.0; n * n];
    let da_slice = da.as_slice();
    for c in 0..n {
        let da_col = c * n;
        for r in 0..n {
            a[r * n + c] = da_slice[da_col + r];
        }
    }

    let mut l = vec![0.0; n * n];
    let mut u = vec![0.0; n * n];
    let mut perm: Vec<usize> = (0..n).collect();
    let mut swaps = 0;

    for i in 0..n {
        // Find pivot
        let mut max_val = a[i * n + i].abs();
        let mut pivot = i;
        for k in (i + 1)..n {
            let val = a[k * n + i].abs();
            if val > max_val {
                max_val = val;
                pivot = k;
            }
        }

        if pivot != i {
            let (row_i, row_p) = if i < pivot {
                let (first, second) = a.split_at_mut(pivot * n);
                (&mut first[i * n..i * n + n], &mut second[..n])
            } else {
                let (first, second) = a.split_at_mut(i * n);
                (&mut second[..n], &mut first[pivot * n..pivot * n + n])
            };
            row_i.swap_with_slice(row_p);
            perm.swap(i, pivot);

            for k in 0..i {
                l.swap(i * n + k, pivot * n + k);
            }
            swaps += 1;
        }

        l[i * n + i] = 1.0;
        let l_i_offset = i * n;
        let u_i_offset = i * n;
        for j in i..n {
            let mut sum = 0.0;
            for k in 0..i {
                sum += l[l_i_offset + k] * u[k * n + j];
            }
            u[u_i_offset + j] = a[i * n + j] - sum;
        }

        let u_ii = u[i * n + i];
        for j in (i + 1)..n {
            let mut sum = 0.0;
            let j_n = j * n;
            for k in 0..i {
                sum += l[j_n + k] * u[k * n + i];
            }
            if u_ii.abs() > 1e-15 {
                l[j_n + i] = (a[j_n + i] - sum) / u_ii;
            } else {
                l[j_n + i] = 0.0;
            }
        }
    }

    // Convert to col-major DenseArrays
    let mut l_data = vec![0.0; n * n];
    let mut u_data = vec![0.0; n * n];
    let mut p_data = vec![0.0; n * n];

    for c in 0..n {
        for r in 0..n {
            l_data[r + c * n] = l[r * n + c];
            u_data[r + c * n] = u[r * n + c];
            if perm[r] == c {
                p_data[r + c * n] = 1.0;
            }
        }
    }

    let l_arr = DenseArray::from_vec_unchecked(vec![n, n], l_data);
    let u_arr = DenseArray::from_vec_unchecked(vec![n, n], u_data);
    let p_arr = DenseArray::from_vec_unchecked(vec![n, n], p_data);

    Ok((l_arr, u_arr, p_arr, swaps))
}

pub fn lu_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "lu requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let (l, u, p, _) = compute_lu(&da)?;

    if nargout == 3 {
        Ok(vec![Value::Double(l), Value::Double(u), Value::Double(p)])
    } else {
        // [L, U] where L is P'*L
        Ok(vec![Value::Double(l), Value::Double(u)])
    }
}

pub fn det_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "det requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:det:notSquare", "Matrix must be square."));
    }
    let n = shp[0];
    let (_, u, _, swaps) = compute_lu(&da)?;

    let mut det = if swaps % 2 == 1 { -1.0 } else { 1.0 };
    for i in 1..=n {
        det *= *u.get_subscript(&[i, i])?;
    }

    Ok(vec![Value::scalar(det)])
}

pub fn norm_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "norm requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let s = da.shape();
    let is_vec = s[0] == 1 || s[1] == 1;

    let p_order = if args.len() >= 2 {
        args[1].as_scalar_f64().unwrap_or(2.0)
    } else {
        2.0
    };

    if is_vec {
        let slice = da.as_slice();
        if p_order == 1.0 {
            let n: f64 = slice.iter().map(|x| x.abs()).sum();
            Ok(vec![Value::scalar(n)])
        } else if p_order == 2.0 {
            let sum_sq: f64 = slice.iter().map(|x| x * x).sum();
            Ok(vec![Value::scalar(sum_sq.sqrt())])
        } else if p_order.is_infinite() {
            let max_val = slice.iter().map(|x| x.abs()).fold(0.0_f64, |a, b| a.max(b));
            Ok(vec![Value::scalar(max_val)])
        } else {
            let sum_p: f64 = slice.iter().map(|x| x.abs().powf(p_order)).sum();
            Ok(vec![Value::scalar(sum_p.powf(1.0 / p_order))])
        }
    } else {
        // Frobenius norm by default for matrix if p_order == 2 or 'fro'
        let sum_sq: f64 = da.as_slice().iter().map(|x| x * x).sum();
        Ok(vec![Value::scalar(sum_sq.sqrt())])
    }
}

pub fn chol_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "chol requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:chol:notSquare", "Matrix must be square."));
    }
    let n = shp[0];
    let mut r = vec![0.0; n * n];

    for j in 0..n {
        let mut sum = 0.0;
        for k in 0..j {
            let r_kj = r[k + j * n];
            sum += r_kj * r_kj;
        }
        let diag_val = *da.get_subscript(&[j + 1, j + 1])? - sum;
        if diag_val <= 0.0 {
            return Err(user_err(
                "MATLAB:posdef",
                "Matrix must be positive definite.",
            ));
        }
        let r_jj = diag_val.sqrt();
        r[j + j * n] = r_jj;

        for i in (j + 1)..n {
            let mut s = 0.0;
            for k in 0..j {
                s += r[k + j * n] * r[k + i * n];
            }
            let a_ji = *da.get_subscript(&[j + 1, i + 1])?;
            r[j + i * n] = (a_ji - s) / r_jj;
        }
    }

    let r_arr = DenseArray::from_vec_unchecked(vec![n, n], r);
    Ok(vec![Value::Double(r_arr)])
}
