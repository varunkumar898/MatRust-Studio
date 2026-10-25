//! Singular Value Decomposition (SVD), Moore-Penrose pseudoinverse (pinv),
//! rank, null space (null), range (orth), condition number (cond), and general eig.

use crate::helpers::{extract_f64_array, user_err};
#[cfg(feature = "faer")]
use num_complex::Complex64;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

/// Computes positive floating point spacing (distance from |x| to next larger float),
/// matching MATLAB's `eps(x)`.
#[inline]
pub fn eps(x: f64) -> f64 {
    let x = x.abs();
    if x.is_nan() || x.is_infinite() {
        f64::NAN
    } else {
        let bits = x.to_bits();
        f64::from_bits(bits + 1) - x
    }
}

/// Computes the Singular Value Decomposition of an M x N matrix.
///
/// Syntax:
/// - `s = svd(A)`: returns min(M, N) x 1 column vector of singular values in descending order.
/// - `[U, S, V] = svd(A)`: full SVD (U is M x M, S is M x N, V is N x N).
/// - `[U, S, V] = svd(A, 'econ')` or `svd(A, 0)`: economy size SVD (U is M x K, S is K x K, V is N x K where K = min(M, N)).
pub fn svd_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "svd requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:svd:matrixRequired",
            "Input must be a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);

    if m == 0 || n == 0 {
        if nargout <= 1 {
            let s_arr = DenseArray::from_vec_unchecked(vec![0, 1], Vec::new());
            return Ok(vec![Value::Double(s_arr)]);
        } else {
            let u_arr = DenseArray::from_vec_unchecked(vec![m, m], Vec::new());
            let s_arr = DenseArray::from_vec_unchecked(vec![m, n], Vec::new());
            let v_arr = DenseArray::from_vec_unchecked(vec![n, n], Vec::new());
            return Ok(vec![
                Value::Double(u_arr),
                Value::Double(s_arr),
                Value::Double(v_arr),
            ]);
        }
    }

    #[cfg(feature = "faer")]
    {
        let k = m.min(n);
        let is_econ = if args.len() >= 2 {
            match &args[1] {
                Value::Char(c) => {
                    let s: String = c.as_slice().iter().collect();
                    s == "econ"
                }
                Value::String(sa) => sa.as_slice().first().map(|s| s == "econ").unwrap_or(false),
                v => v.as_scalar_f64().map(|x| x == 0.0).unwrap_or(false),
            }
        } else {
            false
        };

        let faer_mat = faer::Mat::from_fn(m, n, |r, c| da.as_slice()[r + c * m]);
        let svd = faer_mat
            .svd()
            .map_err(|e| user_err("MATLAB:svd:noConvergence", &format!("{e:?}")))?;

        let s_diag = svd.S();
        let mut s_vals = Vec::with_capacity(k);
        for i in 0..k {
            s_vals.push(s_diag[i]);
        }

        if nargout <= 1 {
            let s_col = DenseArray::from_vec_unchecked(vec![k, 1], s_vals);
            return Ok(vec![Value::Double(s_col)]);
        }

        let u_mat = svd.U();
        let v_mat = svd.V();

        if is_econ {
            let mut u_data = vec![0.0; m * k];
            let mut s_data = vec![0.0; k * k];
            let mut v_data = vec![0.0; n * k];

            for c in 0..k {
                for r in 0..m {
                    u_data[r + c * m] = u_mat[(r, c)];
                }
                for r in 0..n {
                    v_data[r + c * n] = v_mat[(r, c)];
                }
                s_data[c + c * k] = s_vals[c];
            }

            let u_arr = DenseArray::from_vec_unchecked(vec![m, k], u_data);
            let s_arr = DenseArray::from_vec_unchecked(vec![k, k], s_data);
            let v_arr = DenseArray::from_vec_unchecked(vec![n, k], v_data);
            Ok(vec![
                Value::Double(u_arr),
                Value::Double(s_arr),
                Value::Double(v_arr),
            ])
        } else {
            // Full SVD
            // Note: faer thin SVD gives U [M, K] and V [N, K].
            // To provide full SVD, if M == K or N == K we construct full matrices
            let mut u_data = vec![0.0; m * m];
            let mut s_data = vec![0.0; m * n];
            let mut v_data = vec![0.0; n * n];

            let u_cols = u_mat.ncols().min(m);
            for c in 0..u_cols {
                for r in 0..m {
                    u_data[r + c * m] = u_mat[(r, c)];
                }
            }
            // Fill identity on any remaining columns of U
            for c in u_cols..m {
                u_data[c + c * m] = 1.0;
            }

            let v_cols = v_mat.ncols().min(n);
            for c in 0..v_cols {
                for r in 0..n {
                    v_data[r + c * n] = v_mat[(r, c)];
                }
            }
            // Fill identity on any remaining columns of V
            for c in v_cols..n {
                v_data[c + c * n] = 1.0;
            }

            for i in 0..k {
                s_data[i + i * m] = s_vals[i];
            }

            let u_arr = DenseArray::from_vec_unchecked(vec![m, m], u_data);
            let s_arr = DenseArray::from_vec_unchecked(vec![m, n], s_data);
            let v_arr = DenseArray::from_vec_unchecked(vec![n, n], v_data);
            Ok(vec![
                Value::Double(u_arr),
                Value::Double(s_arr),
                Value::Double(v_arr),
            ])
        }
    }

    #[cfg(not(feature = "faer"))]
    {
        let (s_vals, u_arr, v_arr) = jacobi_svd_native(&da)?;
        let k = m.min(n);
        let is_econ = if args.len() >= 2 {
            match &args[1] {
                Value::Char(c) => {
                    let s: String = c.as_slice().iter().collect();
                    s == "econ"
                }
                Value::String(sa) => sa.as_slice().first().map(|s| s == "econ").unwrap_or(false),
                v => v.as_scalar_f64().map(|x| x == 0.0).unwrap_or(false),
            }
        } else {
            false
        };

        if nargout <= 1 {
            let s_col = DenseArray::from_vec_unchecked(vec![k, 1], s_vals);
            return Ok(vec![Value::Double(s_col)]);
        }

        if is_econ {
            let mut s_data = vec![0.0; k * k];
            for i in 0..k {
                s_data[i + i * k] = s_vals[i];
            }
            let s_mat = DenseArray::from_vec_unchecked(vec![k, k], s_data);
            Ok(vec![
                Value::Double(u_arr),
                Value::Double(s_mat),
                Value::Double(v_arr),
            ])
        } else {
            let mut u_full = vec![0.0; m * m];
            let mut s_full = vec![0.0; m * n];
            let mut v_full = vec![0.0; n * n];
            for c in 0..k {
                for r in 0..m {
                    u_full[r + c * m] = u_arr.as_slice()[r + c * m];
                }
                for r in 0..n {
                    v_full[r + c * n] = v_arr.as_slice()[r + c * n];
                }
                s_full[c + c * m] = s_vals[c];
            }
            for c in k..m {
                u_full[c + c * m] = 1.0;
            }
            for c in k..n {
                v_full[c + c * n] = 1.0;
            }
            let u_out = DenseArray::from_vec_unchecked(vec![m, m], u_full);
            let s_out = DenseArray::from_vec_unchecked(vec![m, n], s_full);
            let v_out = DenseArray::from_vec_unchecked(vec![n, n], v_full);
            Ok(vec![
                Value::Double(u_out),
                Value::Double(s_out),
                Value::Double(v_out),
            ])
        }
    }
}

/// Computes the Moore-Penrose pseudoinverse of matrix A: pinv(A).
pub fn pinv_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "pinv requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:pinv:matrixRequired",
            "Input must be a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);
    if m == 0 || n == 0 {
        let out = DenseArray::from_vec_unchecked(vec![n, m], Vec::new());
        return Ok(vec![Value::Double(out)]);
    }

    #[cfg(feature = "faer")]
    {
        let faer_mat = faer::Mat::from_fn(m, n, |r, c| da.as_slice()[r + c * m]);
        let svd = faer_mat
            .svd()
            .map_err(|e| user_err("MATLAB:pinv:noConvergence", &format!("{e:?}")))?;

        let k = m.min(n);
        let s_diag = svd.S();
        let max_sigma = if k > 0 { s_diag[0] } else { 0.0 };
        let tol = if args.len() >= 2 {
            args[1]
                .as_scalar_f64()
                .unwrap_or((m.max(n) as f64) * eps(max_sigma))
        } else {
            (m.max(n) as f64) * eps(max_sigma)
        };

        let u = svd.U();
        let v = svd.V();

        // pinv = V * S^+ * U^T
        // Shape: [N, M]
        let mut pinv_data = vec![0.0; n * m];

        for c_out in 0..m {
            for r_out in 0..n {
                let mut sum = 0.0;
                for i in 0..k {
                    let sigma = s_diag[i];
                    if sigma > tol {
                        let inv_s = 1.0 / sigma;
                        sum += v[(r_out, i)] * inv_s * u[(c_out, i)];
                    }
                }
                pinv_data[r_out + c_out * n] = sum;
            }
        }

        let out = DenseArray::from_vec_unchecked(vec![n, m], pinv_data);
        Ok(vec![Value::Double(out)])
    }

    #[cfg(not(feature = "faer"))]
    {
        let (s_vals, u_arr, v_arr) = jacobi_svd_native(&da)?;
        let k = m.min(n);
        let max_sigma = if k > 0 { s_vals[0] } else { 0.0 };
        let tol = if args.len() >= 2 {
            args[1]
                .as_scalar_f64()
                .unwrap_or((m.max(n) as f64) * eps(max_sigma))
        } else {
            (m.max(n) as f64) * eps(max_sigma)
        };

        let mut pinv_data = vec![0.0; n * m];
        for c_out in 0..m {
            for r_out in 0..n {
                let mut sum = 0.0;
                for i in 0..k {
                    let sigma = s_vals[i];
                    if sigma > tol {
                        let inv_s = 1.0 / sigma;
                        sum += v_arr.as_slice()[r_out + i * n]
                            * inv_s
                            * u_arr.as_slice()[c_out + i * m];
                    }
                }
                pinv_data[r_out + c_out * n] = sum;
            }
        }

        let out = DenseArray::from_vec_unchecked(vec![n, m], pinv_data);
        Ok(vec![Value::Double(out)])
    }
}

/// Computes the matrix rank: rank(A).
pub fn rank_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "rank requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:rank:matrixRequired",
            "Input must be a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);
    let k = m.min(n);
    if k == 0 {
        return Ok(vec![Value::scalar(0.0)]);
    }

    #[cfg(feature = "faer")]
    {
        let faer_mat = faer::Mat::from_fn(m, n, |r, c| da.as_slice()[r + c * m]);
        let svd = faer_mat
            .svd()
            .map_err(|e| user_err("MATLAB:rank:noConvergence", &format!("{e:?}")))?;

        let s_diag = svd.S();
        let max_sigma = s_diag[0];
        let tol = if args.len() >= 2 {
            args[1]
                .as_scalar_f64()
                .unwrap_or((m.max(n) as f64) * eps(max_sigma))
        } else {
            (m.max(n) as f64) * eps(max_sigma)
        };

        let mut r = 0;
        for i in 0..k {
            if s_diag[i] > tol {
                r += 1;
            }
        }

        Ok(vec![Value::scalar(r as f64)])
    }

    #[cfg(not(feature = "faer"))]
    {
        let (s_vals, _, _) = jacobi_svd_native(&da)?;
        let k = m.min(n);
        let max_sigma = if k > 0 { s_vals[0] } else { 0.0 };
        let tol = if args.len() >= 2 {
            args[1]
                .as_scalar_f64()
                .unwrap_or((m.max(n) as f64) * eps(max_sigma))
        } else {
            (m.max(n) as f64) * eps(max_sigma)
        };

        let mut r = 0;
        for i in 0..k {
            if s_vals[i] > tol {
                r += 1;
            }
        }
        Ok(vec![Value::scalar(r as f64)])
    }
}

/// Computes an orthonormal basis for the range of A: orth(A).
pub fn orth_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "orth requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let (m, n) = (da.shape()[0], da.shape()[1]);
    let k = m.min(n);
    if k == 0 {
        let out = DenseArray::from_vec_unchecked(vec![m, 0], Vec::new());
        return Ok(vec![Value::Double(out)]);
    }

    #[cfg(feature = "faer")]
    {
        let faer_mat = faer::Mat::from_fn(m, n, |r, c| da.as_slice()[r + c * m]);
        let svd = faer_mat
            .svd()
            .map_err(|e| user_err("MATLAB:orth:noConvergence", &format!("{e:?}")))?;
        let s_diag = svd.S();
        let max_sigma = s_diag[0];
        let tol = (m.max(n) as f64) * eps(max_sigma);

        let mut rank = 0;
        for i in 0..k {
            if s_diag[i] > tol {
                rank += 1;
            }
        }

        if rank == 0 {
            let out = DenseArray::from_vec_unchecked(vec![m, 0], Vec::new());
            return Ok(vec![Value::Double(out)]);
        }

        let u = svd.U();
        let mut orth_data = vec![0.0; m * rank];
        for c in 0..rank {
            for r in 0..m {
                orth_data[r + c * m] = u[(r, c)];
            }
        }

        let out = DenseArray::from_vec_unchecked(vec![m, rank], orth_data);
        Ok(vec![Value::Double(out)])
    }

    #[cfg(not(feature = "faer"))]
    {
        let (s_vals, u_arr, _) = jacobi_svd_native(&da)?;
        let k = m.min(n);
        let max_sigma = if k > 0 { s_vals[0] } else { 0.0 };
        let tol = (m.max(n) as f64) * eps(max_sigma);

        let mut rank = 0;
        for i in 0..k {
            if s_vals[i] > tol {
                rank += 1;
            }
        }

        if rank == 0 {
            let out = DenseArray::from_vec_unchecked(vec![m, 0], Vec::new());
            return Ok(vec![Value::Double(out)]);
        }

        let mut orth_data = vec![0.0; m * rank];
        for c in 0..rank {
            for r in 0..m {
                orth_data[r + c * m] = u_arr.as_slice()[r + c * m];
            }
        }

        let out = DenseArray::from_vec_unchecked(vec![m, rank], orth_data);
        Ok(vec![Value::Double(out)])
    }
}

/// Computes an orthonormal basis for the null space of A: null(A).
pub fn null_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "null requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:null:matrixRequired",
            "Input must be a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);
    if m == 0 || n == 0 {
        let out = DenseArray::from_vec_unchecked(vec![n, 0], Vec::new());
        return Ok(vec![Value::Double(out)]);
    }

    #[cfg(feature = "faer")]
    {
        let k = m.min(n);
        let faer_mat = faer::Mat::from_fn(m, n, |r, c| da.as_slice()[r + c * m]);
        let svd = faer_mat
            .svd()
            .map_err(|e| user_err("MATLAB:null:noConvergence", &format!("{e:?}")))?;
        let s_diag = svd.S();
        let max_sigma = if k > 0 { s_diag[0] } else { 0.0 };
        let tol = (m.max(n) as f64) * eps(max_sigma);

        let mut rank = 0;
        for i in 0..k {
            if s_diag[i] > tol {
                rank += 1;
            }
        }

        let null_dim = n - rank;
        if null_dim == 0 {
            let out = DenseArray::from_vec_unchecked(vec![n, 0], Vec::new());
            return Ok(vec![Value::Double(out)]);
        }

        let v = svd.V();
        let mut null_data = vec![0.0; n * null_dim];
        for c in 0..null_dim {
            let src_col = rank + c;
            for r in 0..n {
                null_data[r + c * n] = v[(r, src_col)];
            }
        }

        let out = DenseArray::from_vec_unchecked(vec![n, null_dim], null_data);
        Ok(vec![Value::Double(out)])
    }

    #[cfg(not(feature = "faer"))]
    {
        let (s_vals, _, v_arr) = jacobi_svd_native(&da)?;
        let k = m.min(n);
        let max_sigma = if k > 0 { s_vals[0] } else { 0.0 };
        let tol = (m.max(n) as f64) * eps(max_sigma);

        let mut rank = 0;
        for i in 0..k {
            if s_vals[i] > tol {
                rank += 1;
            }
        }

        let null_dim = n - rank;
        if null_dim == 0 {
            let out = DenseArray::from_vec_unchecked(vec![n, 0], Vec::new());
            return Ok(vec![Value::Double(out)]);
        }

        let mut null_data = vec![0.0; n * null_dim];
        for c in 0..null_dim {
            let src_col = rank + c;
            for r in 0..n {
                null_data[r + c * n] = v_arr.as_slice()[r + src_col * n];
            }
        }

        let out = DenseArray::from_vec_unchecked(vec![n, null_dim], null_data);
        Ok(vec![Value::Double(out)])
    }
}

/// Computes matrix condition number: cond(A).
pub fn cond_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "cond requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let (m, n) = (da.shape()[0], da.shape()[1]);
    let k = m.min(n);
    if k == 0 {
        return Ok(vec![Value::scalar(0.0)]);
    }

    #[cfg(feature = "faer")]
    {
        let faer_mat = faer::Mat::from_fn(m, n, |r, c| da.as_slice()[r + c * m]);
        let svd = faer_mat
            .svd()
            .map_err(|e| user_err("MATLAB:cond:noConvergence", &format!("{e:?}")))?;
        let s_diag = svd.S();
        let s_max = s_diag[0];
        let s_min = s_diag[k - 1];

        if s_min == 0.0 {
            return Ok(vec![Value::scalar(f64::INFINITY)]);
        }

        let c = s_max / s_min;
        Ok(vec![Value::scalar(c)])
    }

    #[cfg(not(feature = "faer"))]
    {
        let (s_vals, _, _) = jacobi_svd_native(&da)?;
        let s_max = s_vals[0];
        let s_min = s_vals[k - 1];
        if s_min == 0.0 {
            return Ok(vec![Value::scalar(f64::INFINITY)]);
        }
        Ok(vec![Value::scalar(s_max / s_min)])
    }
}

/// Eigenvalue decomposition: eig(A) or [V, D] = eig(A).
/// Automatically uses native Householder + implicit QL for symmetric matrices,
/// and delegates general non-symmetric matrices to faer.
pub fn eig_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "eig requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:eig:notSquare", "Matrix must be square."));
    }
    let n = shp[0];

    if n == 0 {
        if nargout <= 1 {
            let d = DenseArray::from_vec_unchecked(vec![0, 1], Vec::new());
            return Ok(vec![Value::Double(d)]);
        } else {
            let v = DenseArray::from_vec_unchecked(vec![0, 0], Vec::new());
            let d = DenseArray::from_vec_unchecked(vec![0, 0], Vec::new());
            return Ok(vec![Value::Double(v), Value::Double(d)]);
        }
    }

    // Check if matrix is symmetric within tolerance
    let mut is_sym = true;
    for i in 1..=n {
        for j in (i + 1)..=n {
            let a_ij = da.as_slice()[(i - 1) + (j - 1) * n];
            let a_ji = da.as_slice()[(j - 1) + (i - 1) * n];
            if (a_ij - a_ji).abs() > 1e-12 * (a_ij.abs().max(a_ji.abs()) + 1.0) {
                is_sym = false;
                break;
            }
        }
        if !is_sym {
            break;
        }
    }

    if is_sym {
        // Single route used by both the builtin and the routing test:
        // faer when enabled, native Householder+QL otherwise. Both backends
        // apply ascending ordering, sign normalization, and (faer) the
        // degenerate-eigenspace tie-break after decomposition.
        let (eigenvalues, v_arr) = crate::linalg::eig_sym::eig_sym_auto(&da)?;
        if nargout <= 1 {
            let d_col = DenseArray::from_vec_unchecked(vec![n, 1], eigenvalues);
            return Ok(vec![Value::Double(d_col)]);
        } else {
            let mut d_mat = vec![0.0; n * n];
            for i in 0..n {
                d_mat[i + i * n] = eigenvalues[i];
            }
            let d_arr = DenseArray::from_vec_unchecked(vec![n, n], d_mat);
            return Ok(vec![Value::Double(v_arr), Value::Double(d_arr)]);
        }
    }

    // General non-symmetric eigenvalue decomposition via faer
    #[cfg(feature = "faer")]
    {
        let faer_mat = faer::Mat::from_fn(n, n, |r, c| da.as_slice()[r + c * n]);

        if nargout <= 1 {
            let evals = faer_mat
                .eigenvalues()
                .map_err(|e| user_err("MATLAB:eig:noConvergence", &format!("{e:?}")))?;
            let has_imag = evals.iter().any(|c| c.im.abs() > 1e-14);
            if has_imag {
                let c_slice: Vec<Complex64> =
                    evals.iter().map(|c| Complex64::new(c.re, c.im)).collect();
                let arr = DenseArray::from_vec_unchecked(vec![n, 1], c_slice);
                Ok(vec![Value::ComplexDouble(arr)])
            } else {
                let r_slice: Vec<f64> = evals.iter().map(|c| c.re).collect();
                let arr = DenseArray::from_vec_unchecked(vec![n, 1], r_slice);
                Ok(vec![Value::Double(arr)])
            }
        } else {
            let evd = faer_mat
                .eigen()
                .map_err(|e| user_err("MATLAB:eig:noConvergence", &format!("{e:?}")))?;
            let u_mat = evd.U();
            let s_diag = evd.S();

            let has_imag_d = (0..n).any(|i| s_diag[i].im.abs() > 1e-14);
            let has_imag_v = (0..n).any(|c| (0..n).any(|r| u_mat[(r, c)].im.abs() > 1e-14));

            if has_imag_d || has_imag_v {
                let mut v_data = vec![Complex64::new(0.0, 0.0); n * n];
                let mut d_data = vec![Complex64::new(0.0, 0.0); n * n];
                for c in 0..n {
                    for r in 0..n {
                        let val = u_mat[(r, c)];
                        v_data[r + c * n] = Complex64::new(val.re, val.im);
                    }
                    let val = s_diag[c];
                    d_data[c + c * n] = Complex64::new(val.re, val.im);
                }
                let v_arr = DenseArray::from_vec_unchecked(vec![n, n], v_data);
                let d_arr = DenseArray::from_vec_unchecked(vec![n, n], d_data);
                Ok(vec![
                    Value::ComplexDouble(v_arr),
                    Value::ComplexDouble(d_arr),
                ])
            } else {
                let mut v_data = vec![0.0; n * n];
                let mut d_data = vec![0.0; n * n];
                for c in 0..n {
                    for r in 0..n {
                        v_data[r + c * n] = u_mat[(r, c)].re;
                    }
                    d_data[c + c * n] = s_diag[c].re;
                }
                let v_arr = DenseArray::from_vec_unchecked(vec![n, n], v_data);
                let d_arr = DenseArray::from_vec_unchecked(vec![n, n], d_data);
                Ok(vec![Value::Double(v_arr), Value::Double(d_arr)])
            }
        }
    }

    #[cfg(not(feature = "faer"))]
    {
        Err(user_err(
            "MATLAB:eig:featureRequired",
            "Non-symmetric eigenvalue decomposition requires the 'faer' feature flag.",
        ))
    }
}

/// Native one-sided Hestenes Jacobi SVD algorithm (pure Rust, zero dependencies).
/// Computes A = U * S * V^T, returning (singular_values, U, V).
/// U is M x K, V is N x K where K = min(M, N) (economy SVD).
pub fn jacobi_svd_native(
    da: &DenseArray<f64>,
) -> Result<(Vec<f64>, DenseArray<f64>, DenseArray<f64>), EvalError> {
    let shp = da.shape();
    let (m, n) = (shp[0], shp[1]);
    let k = m.min(n);
    if k == 0 {
        let u = DenseArray::from_vec_unchecked(vec![m, 0], Vec::new());
        let v = DenseArray::from_vec_unchecked(vec![n, 0], Vec::new());
        return Ok((Vec::new(), u, v));
    }

    if m >= n {
        let mut w = da.as_slice().to_vec();
        let mut v = vec![0.0; n * n];
        for i in 0..n {
            v[i + i * n] = 1.0;
        }

        for _sweep in 0..30 {
            let mut max_off_diag = 0.0f64;
            for i in 0..n {
                for j in (i + 1)..n {
                    let mut alpha = 0.0f64;
                    let mut beta = 0.0f64;
                    let mut gamma = 0.0f64;
                    for r in 0..m {
                        let wi = w[r + i * m];
                        let wj = w[r + j * m];
                        alpha += wi * wi;
                        beta += wj * wj;
                        gamma += wi * wj;
                    }
                    let denom = (alpha * beta).sqrt();
                    if denom > 1e-15 {
                        max_off_diag = max_off_diag.max(gamma.abs() / denom);
                    }
                    if gamma.abs() <= 1e-15 * denom || denom == 0.0 {
                        continue;
                    }
                    let zeta = (beta - alpha) / (2.0 * gamma);
                    let t = if zeta >= 0.0 {
                        1.0 / (zeta + (1.0 + zeta * zeta).sqrt())
                    } else {
                        -1.0 / (-zeta + (1.0 + zeta * zeta).sqrt())
                    };
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;
                    for r in 0..m {
                        let wi = w[r + i * m];
                        let wj = w[r + j * m];
                        w[r + i * m] = c * wi - s * wj;
                        w[r + j * m] = s * wi + c * wj;
                    }
                    for r in 0..n {
                        let vi = v[r + i * n];
                        let vj = v[r + j * n];
                        v[r + i * n] = c * vi - s * vj;
                        v[r + j * n] = s * vi + c * vj;
                    }
                }
            }
            if max_off_diag < 1e-14 {
                break;
            }
        }

        let mut sigmas = vec![0.0; n];
        let mut u_data = vec![0.0; m * n];
        for j in 0..n {
            let mut norm_sq = 0.0;
            for r in 0..m {
                let val = w[r + j * m];
                norm_sq += val * val;
            }
            let sigma = norm_sq.sqrt();
            sigmas[j] = sigma;
            if sigma > 1e-15 {
                let inv = 1.0 / sigma;
                for r in 0..m {
                    u_data[r + j * m] = w[r + j * m] * inv;
                }
            }
        }

        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            sigmas[b]
                .partial_cmp(&sigmas[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut sorted_s = vec![0.0; n];
        let mut sorted_u = vec![0.0; m * n];
        let mut sorted_v = vec![0.0; n * n];
        for (new_idx, &old_idx) in order.iter().enumerate() {
            sorted_s[new_idx] = sigmas[old_idx];
            for r in 0..m {
                sorted_u[r + new_idx * m] = u_data[r + old_idx * m];
            }
            for r in 0..n {
                sorted_v[r + new_idx * n] = v[r + old_idx * n];
            }
        }

        let u_arr = DenseArray::from_vec_unchecked(vec![m, n], sorted_u);
        let v_arr = DenseArray::from_vec_unchecked(vec![n, n], sorted_v);
        Ok((sorted_s, u_arr, v_arr))
    } else {
        let mut a_t = vec![0.0; n * m];
        for r in 0..m {
            for c in 0..n {
                a_t[c + r * n] = da.as_slice()[r + c * m];
            }
        }
        let da_t = DenseArray::from_vec_unchecked(vec![n, m], a_t);
        let (s, u_t, v_t) = jacobi_svd_native(&da_t)?;
        Ok((s, v_t, u_t))
    }
}
