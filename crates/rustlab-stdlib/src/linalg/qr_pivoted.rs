//! Pivoted and unpivoted QR decomposition: [Q, R, P] = qr(A).
//!
//! Clean-room native implementation using Householder reflections with Businger-Golub
//! column pivoting, matching standard MATLAB behavior.

use crate::helpers::{extract_f64_array, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

/// Computes the QR decomposition of matrix A.
///
/// Syntax:
/// - `R = qr(A)`
/// - `[Q, R] = qr(A)`
/// - `[Q, R, P] = qr(A)`: returns permutation matrix P such that A * P = Q * R.
/// - `[Q, R, p] = qr(A, 'vector')`: returns permutation vector p such that A(:, p) = Q * R.
/// - `[Q, R] = qr(A, 0)`: economy size QR (Q is M x N, R is N x N for M >= N).
pub fn qr_pivoted_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "qr requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:qr:matrixRequired",
            "Input must be a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);

    if m == 0 || n == 0 {
        if nargout <= 1 {
            let r_arr = DenseArray::from_vec_unchecked(vec![m, n], Vec::new());
            return Ok(vec![Value::Double(r_arr)]);
        } else if nargout == 2 {
            let q_arr = DenseArray::from_vec_unchecked(vec![m, m], Vec::new());
            let r_arr = DenseArray::from_vec_unchecked(vec![m, n], Vec::new());
            return Ok(vec![Value::Double(q_arr), Value::Double(r_arr)]);
        } else {
            let q_arr = DenseArray::from_vec_unchecked(vec![m, m], Vec::new());
            let r_arr = DenseArray::from_vec_unchecked(vec![m, n], Vec::new());
            let p_arr = DenseArray::from_vec_unchecked(vec![n, n], Vec::new());
            return Ok(vec![
                Value::Double(q_arr),
                Value::Double(r_arr),
                Value::Double(p_arr),
            ]);
        }
    }

    let is_econ = if args.len() >= 2 {
        args[1].as_scalar_f64().map(|x| x == 0.0).unwrap_or(false)
    } else {
        false
    };

    // If 3 outputs requested, use column pivoted QR: A * P = Q * R
    if nargout >= 3 {
        let is_vector_perm = if args.len() >= 2 {
            match &args[1] {
                Value::Char(c) => {
                    let s: String = c.as_slice().iter().collect();
                    s == "vector"
                }
                Value::String(sa) => sa
                    .as_slice()
                    .first()
                    .map(|s| s == "vector")
                    .unwrap_or(false),
                _ => false,
            }
        } else {
            false
        };

        // Native column-pivoted Householder QR (Businger-Golub)
        let k = m.min(n);
        let mut cols: Vec<Vec<f64>> = (0..n)
            .map(|j| (0..m).map(|i| da.as_slice()[i + j * m]).collect())
            .collect();
        let mut d: Vec<f64> = cols
            .iter()
            .map(|col| col.iter().map(|&x| x * x).sum())
            .collect();
        let mut p: Vec<usize> = (0..n).collect();

        // Q accumulator (M x M identity)
        let mut q_mat = vec![0.0; m * m];
        for i in 0..m {
            q_mat[i + i * m] = 1.0;
        }

        for i in 0..k {
            // Find column with largest norm in remaining submatrix
            let mut best_j = i;
            let mut max_d = d[i];
            for j in (i + 1)..n {
                if d[j] > max_d {
                    max_d = d[j];
                    best_j = j;
                }
            }

            // Pivot columns
            cols.swap(i, best_j);
            d.swap(i, best_j);
            p.swap(i, best_j);

            if max_d < 1e-28 {
                // Remainder is rank deficient
                break;
            }

            // Compute Householder reflection on column i from row i..m
            let len = m - i;
            let norm_x: f64 = cols[i][i..m].iter().map(|&x| x * x).sum::<f64>().sqrt();
            if norm_x > 1e-15 {
                let x0 = cols[i][i];
                let sign = if x0 >= 0.0 { 1.0 } else { -1.0 };
                let alpha = -sign * norm_x;
                let mut v = cols[i][i..m].to_vec();
                v[0] -= alpha;
                let norm_v: f64 = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
                if norm_v > 1e-15 {
                    for x in &mut v {
                        *x /= norm_v;
                    }
                    cols[i][i] = alpha;
                    for r in (i + 1)..m {
                        cols[i][r] = 0.0;
                    }

                    // Apply to remaining columns j in (i+1)..n
                    for j in (i + 1)..n {
                        let dot: f64 = (0..len).map(|idx| v[idx] * cols[j][i + idx]).sum();
                        for idx in 0..len {
                            cols[j][i + idx] -= 2.0 * v[idx] * dot;
                        }
                        // Update remaining norm
                        d[j] = cols[j][(i + 1)..m].iter().map(|&x| x * x).sum();
                    }

                    // Apply reflection to Q matrix
                    for col_q in 0..m {
                        let dot_q: f64 = (0..len)
                            .map(|idx| v[idx] * q_mat[col_q + (i + idx) * m])
                            .sum();
                        for idx in 0..len {
                            q_mat[col_q + (i + idx) * m] -= 2.0 * v[idx] * dot_q;
                        }
                    }
                }
            }
        }

        // Extract R (K x N)
        let mut r_data = vec![0.0; k * n];
        for j in 0..n {
            for r in 0..k.min(j + 1) {
                r_data[r + j * k] = cols[j][r];
            }
        }

        // Q is M x K (economy)
        let mut q_data = vec![0.0; m * k];
        for j in 0..k {
            for i in 0..m {
                q_data[i + j * m] = q_mat[i + j * m];
            }
        }

        let q_arr = DenseArray::from_vec_unchecked(vec![m, k], q_data);
        let r_arr = DenseArray::from_vec_unchecked(vec![k, n], r_data);

        if is_vector_perm {
            let p_vec: Vec<f64> = p.iter().map(|&idx| (idx + 1) as f64).collect();
            let p_arr = DenseArray::from_vec_unchecked(vec![1, n], p_vec);
            Ok(vec![
                Value::Double(q_arr),
                Value::Double(r_arr),
                Value::Double(p_arr),
            ])
        } else {
            let mut p_data = vec![0.0; n * n];
            for (j, &orig_col) in p.iter().enumerate() {
                p_data[orig_col + j * n] = 1.0;
            }
            let p_arr = DenseArray::from_vec_unchecked(vec![n, n], p_data);
            Ok(vec![
                Value::Double(q_arr),
                Value::Double(r_arr),
                Value::Double(p_arr),
            ])
        }
    } else {
        // Unpivoted QR (Gram-Schmidt or Householder)
        let k = if is_econ { m.min(n) } else { n };
        let mut q = vec![0.0; m * k];
        let mut r = vec![0.0; k * n];

        for j in 0..k {
            let mut v = vec![0.0; m];
            for i in 0..m {
                v[i] = *da.get_subscript(&[i + 1, j + 1])?;
            }

            for i in 0..j {
                let mut dot = 0.0;
                for row in 0..m {
                    dot += q[row + i * m] * v[row];
                }
                r[i + j * k] = dot;
                for row in 0..m {
                    v[row] -= dot * q[row + i * m];
                }
            }

            let norm_v: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            r[j + j * k] = norm_v;
            if norm_v > 1e-15 {
                for row in 0..m {
                    q[row + j * m] = v[row] / norm_v;
                }
            }
        }

        // Fill remaining columns of R for any j >= k
        for j in k..n {
            for i in 0..k {
                let mut dot = 0.0;
                for row in 0..m {
                    dot += q[row + i * m] * *da.get_subscript(&[row + 1, j + 1])?;
                }
                r[i + j * k] = dot;
            }
        }

        let q_arr = DenseArray::from_vec_unchecked(vec![m, k], q);
        let r_arr = DenseArray::from_vec_unchecked(vec![k, n], r);

        if nargout > 1 {
            Ok(vec![Value::Double(q_arr), Value::Double(r_arr)])
        } else {
            Ok(vec![Value::Double(r_arr)])
        }
    }
}
