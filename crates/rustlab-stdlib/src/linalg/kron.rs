//! Kronecker tensor product: kron(A, B).

use crate::helpers::{extract_f64_array, math_err, user_err};
use num_complex::Complex64;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

/// Computes the Kronecker product C = kron(A, B).
pub fn kron_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "kron requires 2 inputs: kron(A, B).",
        ));
    }

    // Check if either argument is complex
    let is_complex =
        matches!(&args[0], Value::ComplexDouble(_)) || matches!(&args[1], Value::ComplexDouble(_));

    if is_complex {
        let a_c = to_complex_dense(&args[0])?;
        let b_c = to_complex_dense(&args[1])?;

        let (ma, na) = (a_c.shape()[0], a_c.shape()[1]);
        let (mb, nb) = (b_c.shape()[0], b_c.shape()[1]);
        let mc = ma * mb;
        let nc = na * nb;

        let mut c_data = vec![Complex64::new(0.0, 0.0); mc * nc];

        for ca in 0..na {
            for cb in 0..nb {
                let out_c = ca * nb + cb;
                for ra in 0..ma {
                    let a_val = *a_c.get_subscript(&[ra + 1, ca + 1])?;
                    for rb in 0..mb {
                        let b_val = *b_c.get_subscript(&[rb + 1, cb + 1])?;
                        let out_r = ra * mb + rb;
                        c_data[out_r + out_c * mc] = a_val * b_val;
                    }
                }
            }
        }

        let arr = DenseArray::from_vec(vec![mc, nc], c_data).map_err(math_err)?;
        Ok(vec![Value::ComplexDouble(arr)])
    } else {
        let a_arr = extract_f64_array(&args[0])?;
        let b_arr = extract_f64_array(&args[1])?;

        let (ma, na) = (a_arr.shape()[0], a_arr.shape()[1]);
        let (mb, nb) = (b_arr.shape()[0], b_arr.shape()[1]);
        let mc = ma * mb;
        let nc = na * nb;

        let mut c_data = vec![0.0; mc * nc];

        for ca in 0..na {
            for cb in 0..nb {
                let out_c = ca * nb + cb;
                for ra in 0..ma {
                    let a_val = *a_arr.get_subscript(&[ra + 1, ca + 1])?;
                    for rb in 0..mb {
                        let b_val = *b_arr.get_subscript(&[rb + 1, cb + 1])?;
                        let out_r = ra * mb + rb;
                        c_data[out_r + out_c * mc] = a_val * b_val;
                    }
                }
            }
        }

        let arr = DenseArray::from_vec(vec![mc, nc], c_data).map_err(math_err)?;
        Ok(vec![Value::Double(arr)])
    }
}

fn to_complex_dense(v: &Value) -> Result<DenseArray<Complex64>, EvalError> {
    match v {
        Value::ComplexDouble(da) => Ok(da.clone()),
        Value::Double(da) => {
            let c: Vec<Complex64> = da
                .as_slice()
                .iter()
                .map(|&x| Complex64::new(x, 0.0))
                .collect();
            DenseArray::from_vec(da.shape().to_vec(), c).map_err(math_err)
        }
        _ => {
            let da = extract_f64_array(v)?;
            let c: Vec<Complex64> = da
                .as_slice()
                .iter()
                .map(|&x| Complex64::new(x, 0.0))
                .collect();
            DenseArray::from_vec(da.shape().to_vec(), c).map_err(math_err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kron_2x2() {
        let a = Value::Double(DenseArray::from_vec_unchecked(
            vec![2, 2],
            vec![1.0, 3.0, 2.0, 4.0],
        )); // [1, 2; 3, 4]
        let b = Value::Double(DenseArray::from_vec_unchecked(
            vec![2, 2],
            vec![1.0, 0.0, 0.0, 1.0],
        )); // eye(2)
        let res = kron_fn(&[a, b], 1).unwrap();
        if let Value::Double(c) = &res[0] {
            assert_eq!(c.shape(), &[4, 4]);
            // Top-left 2x2 should be 1 * eye(2) = eye(2)
            assert_eq!(*c.get_subscript(&[1, 1]).unwrap(), 1.0);
            assert_eq!(*c.get_subscript(&[1, 2]).unwrap(), 0.0);
            assert_eq!(*c.get_subscript(&[2, 1]).unwrap(), 0.0);
            assert_eq!(*c.get_subscript(&[2, 2]).unwrap(), 1.0);
            // Top-right 2x2 should be 2 * eye(2)
            assert_eq!(*c.get_subscript(&[1, 3]).unwrap(), 2.0);
            assert_eq!(*c.get_subscript(&[2, 4]).unwrap(), 2.0);
        } else {
            panic!();
        }
    }
}
