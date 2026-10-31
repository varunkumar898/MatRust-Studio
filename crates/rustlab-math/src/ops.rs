//! Mathematical operations: elementwise broadcasting, linear algebra, and ranges.

use crate::array::DenseArray;
use crate::broadcast::{broadcast_shapes, BroadcastPairIterator};
use crate::error::MathError;

/// Applies a binary elementwise function across two arrays with implicit expansion (broadcasting).
/// Includes specialized fast paths for identical shapes and scalar operands to achieve maximum SIMD/cache throughput.
pub fn elementwise_binary<T, U, R, F>(
    a: &DenseArray<T>,
    b: &DenseArray<U>,
    mut f: F,
) -> Result<DenseArray<R>, MathError>
where
    F: FnMut(&T, &U) -> R,
{
    // Fast Path 1: Exact shape match (no broadcast indexing overhead)
    if a.shape() == b.shape() {
        let a_slice = a.as_slice();
        let b_slice = b.as_slice();
        let out_data: Vec<R> = a_slice
            .iter()
            .zip(b_slice.iter())
            .map(|(va, vb)| f(va, vb))
            .collect();
        return Ok(DenseArray::from_vec_unchecked(a.shape().to_vec(), out_data));
    }

    // Fast Path 2: Scalar broadcast from left
    if a.is_scalar() {
        let a_val = &a.as_slice()[0];
        let b_slice = b.as_slice();
        let mut out_data = Vec::with_capacity(b_slice.len());
        for vb in b_slice.iter() {
            out_data.push(f(a_val, vb));
        }
        return DenseArray::from_vec(b.shape().to_vec(), out_data);
    }

    // Fast Path 3: Scalar broadcast from right
    if b.is_scalar() {
        let b_val = &b.as_slice()[0];
        let a_slice = a.as_slice();
        let mut out_data = Vec::with_capacity(a_slice.len());
        for va in a_slice.iter() {
            out_data.push(f(va, b_val));
        }
        return DenseArray::from_vec(a.shape().to_vec(), out_data);
    }

    // General N-D broadcasting fallback
    let target_shape = broadcast_shapes(a.shape(), b.shape())?;
    let iter = BroadcastPairIterator::new(
        a.shape(),
        a.strides(),
        b.shape(),
        b.strides(),
        &target_shape,
    );

    let a_slice = a.as_slice();
    let b_slice = b.as_slice();
    let mut out_data = Vec::with_capacity(iter.len());

    for (offset_a, offset_b) in iter {
        out_data.push(f(&a_slice[offset_a], &b_slice[offset_b]));
    }

    DenseArray::from_vec(target_shape, out_data)
}

/// Applies a unary elementwise function across an array, preserving shape and strides.
pub fn elementwise_unary<T, R, F>(a: &DenseArray<T>, f: F) -> DenseArray<R>
where
    F: FnMut(&T) -> R,
{
    let out_data = a.as_slice().iter().map(f).collect();
    DenseArray::from_vec_unchecked(a.shape().to_vec(), out_data)
}

/// Performs 2D matrix multiplication $C = A \times B$ in column-major layout.
/// Inner dimensions must match: $[M, K] \times [K, N] \to [M, N]$.
pub fn matmul_2d(a: &DenseArray<f64>, b: &DenseArray<f64>) -> Result<DenseArray<f64>, MathError> {
    if !a.is_matrix() || !b.is_matrix() {
        return Err(MathError::UnsupportedOperation {
            op: "matrix multiplication",
            type_name: "non-2D array",
        });
    }

    let m = a.shape()[0];
    let k_a = a.shape()[1];
    let k_b = b.shape()[0];
    let n = b.shape()[1];

    if k_a != k_b {
        return Err(MathError::inner_dim(k_a, k_b));
    }

    // Dimension / overflow checks
    let total_c = match m.checked_mul(n) {
        Some(sz) if sz <= 128 * 1024 * 1024 => sz,
        _ => {
            return Err(MathError::DimensionMismatch {
                identifier: "MATLAB:pmaxsize",
                shape_a: vec![m, n],
                shape_b: Vec::new(),
            });
        }
    };

    // If any dimension is zero, return empty or zero matrix safely without calling external kernels
    if m == 0 || n == 0 {
        return DenseArray::from_vec(vec![m, n], Vec::new());
    }
    if k_a == 0 {
        return DenseArray::from_vec(vec![m, n], vec![0.0; total_c]);
    }

    // Verify stride conversions to isize do not overflow
    let m_isize = isize::try_from(m).map_err(|_| MathError::DimensionMismatch {
        identifier: "MATLAB:pmaxsize",
        shape_a: vec![m],
        shape_b: Vec::new(),
    })?;
    let k_isize = isize::try_from(k_a).map_err(|_| MathError::DimensionMismatch {
        identifier: "MATLAB:pmaxsize",
        shape_a: vec![k_a],
        shape_b: Vec::new(),
    })?;

    // Verify input buffer lengths match dimensions
    let total_a = m
        .checked_mul(k_a)
        .ok_or_else(|| MathError::DimensionMismatch {
            identifier: "MATLAB:pmaxsize",
            shape_a: vec![m, k_a],
            shape_b: Vec::new(),
        })?;
    let total_b = k_a
        .checked_mul(n)
        .ok_or_else(|| MathError::DimensionMismatch {
            identifier: "MATLAB:pmaxsize",
            shape_a: vec![k_a, n],
            shape_b: Vec::new(),
        })?;

    if a.as_slice().len() != total_a || b.as_slice().len() != total_b {
        return Err(MathError::dim_agree(
            &[a.as_slice().len(), b.as_slice().len()],
            &[total_a, total_b],
        ));
    }

    #[cfg(feature = "matrixmultiply")]
    {
        let mut c_data = vec![0.0; total_c];
        let a_data = a.as_slice();
        let b_data = b.as_slice();
        // matrixmultiply::dgemm uses column-major layout when rsa=1, csa=m, rsb=1, csb=k, rsc=1, csc=m
        unsafe {
            matrixmultiply::dgemm(
                m,
                k_a,
                n,
                1.0,
                a_data.as_ptr(),
                1,
                m_isize,
                b_data.as_ptr(),
                1,
                k_isize,
                0.0,
                c_data.as_mut_ptr(),
                1,
                m_isize,
            );
        }
        Ok(DenseArray::from_vec_unchecked(vec![m, n], c_data))
    }

    #[cfg(not(feature = "matrixmultiply"))]
    {
        let mut c_data = vec![0.0; m * n];
        let a_data = a.as_slice();
        let b_data = b.as_slice();

        // Cache-friendly column-major accumulation with hoisted slice bounds:
        for col in 0..n {
            let c_col = &mut c_data[col * m..(col + 1) * m];
            for k in 0..k_a {
                let b_val = b_data[col * k_a + k];
                if b_val != 0.0 {
                    let a_col = &a_data[k * m..(k + 1) * m];
                    for (c_elem, a_elem) in c_col.iter_mut().zip(a_col.iter()) {
                        *c_elem += *a_elem * b_val;
                    }
                }
            }
        }

        DenseArray::from_vec(vec![m, n], c_data)
    }
}

/// Generates a linearly spaced row vector according to MATLAB colon syntax `start:step:end`.
pub fn colon_range(start: f64, step: f64, end: f64) -> DenseArray<f64> {
    if step == 0.0 || (step > 0.0 && start > end) || (step < 0.0 && start < end) {
        return DenseArray::empty();
    }

    let count = ((end - start) / step).floor() as usize + 1;
    let mut data = Vec::with_capacity(count);

    let mut current = start;
    for _ in 0..count {
        data.push(current);
        current += step;
    }

    DenseArray::row_vector(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elementwise_binary_with_broadcasting() {
        // [10; 20] + [1, 2] -> 2x2 matrix
        let a = DenseArray::col_vector(vec![10.0, 20.0]);
        let b = DenseArray::row_vector(vec![1.0, 2.0]);

        let c = elementwise_binary(&a, &b, |x, y| x + y).unwrap();
        assert_eq!(c.shape(), &[2, 2]);
        // Col 1: 11, 21
        // Col 2: 12, 22
        assert_eq!(c.as_slice(), &[11.0, 21.0, 12.0, 22.0]);
    }

    #[test]
    fn test_matmul_2d() {
        // A = [1 2; 3 4] (2x2)
        // Col 1: 1, 3
        // Col 2: 2, 4
        let a = DenseArray::from_vec(vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]).unwrap();

        // B = [5 6; 7 8] (2x2)
        // Col 1: 5, 7
        // Col 2: 6, 8
        let b = DenseArray::from_vec(vec![2, 2], vec![5.0, 7.0, 6.0, 8.0]).unwrap();

        // C = A * B
        // Row 1: 1*5 + 2*7 = 19, 1*6 + 2*8 = 22
        // Row 2: 3*5 + 4*7 = 43, 3*6 + 4*8 = 50
        // Col-major C: 19, 43, 22, 50
        let c = matmul_2d(&a, &b).unwrap();
        assert_eq!(c.shape(), &[2, 2]);
        assert_eq!(c.as_slice(), &[19.0, 43.0, 22.0, 50.0]);
    }

    #[test]
    fn test_colon_range() {
        let r1 = colon_range(1.0, 1.0, 5.0);
        assert_eq!(r1.shape(), &[1, 5]);
        assert_eq!(r1.as_slice(), &[1.0, 2.0, 3.0, 4.0, 5.0]);

        let r2 = colon_range(1.0, 2.0, 6.0);
        assert_eq!(r2.as_slice(), &[1.0, 3.0, 5.0]);

        let r3 = colon_range(5.0, -1.0, 2.0);
        assert_eq!(r3.as_slice(), &[5.0, 4.0, 3.0, 2.0]);
    }
}
