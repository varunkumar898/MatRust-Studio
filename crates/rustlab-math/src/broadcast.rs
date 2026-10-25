//! Broadcasting (implicit expansion) algorithms and coordinate iterators.

use crate::error::MathError;
use crate::shape::normalize_shape;

/// Computes the resulting shape from broadcasting two shapes according to MATLAB rules.
/// Returns `Err(MathError::DimensionMismatch)` if the shapes are incompatible.
pub fn broadcast_shapes(shape_a: &[usize], shape_b: &[usize]) -> Result<Vec<usize>, MathError> {
    let max_ndims = shape_a.len().max(shape_b.len());
    let mut out_shape = Vec::with_capacity(max_ndims);

    for i in 0..max_ndims {
        let dim_a = shape_a.get(i).copied().unwrap_or(1);
        let dim_b = shape_b.get(i).copied().unwrap_or(1);

        if dim_a == dim_b {
            out_shape.push(dim_a);
        } else if dim_a == 1 {
            out_shape.push(dim_b);
        } else if dim_b == 1 {
            out_shape.push(dim_a);
        } else {
            return Err(MathError::dim_agree(shape_a, shape_b));
        }
    }

    Ok(normalize_shape(&out_shape))
}

/// Computes the broadcasted strides for an array mapped into a larger target shape.
/// Dimensions that expand from size 1 to $N > 1$ are assigned a stride of **0**.
pub fn compute_broadcast_strides(
    orig_shape: &[usize],
    target_shape: &[usize],
    orig_strides: &[usize],
) -> Vec<usize> {
    let mut broadcast_strides = Vec::with_capacity(target_shape.len());

    for (i, &target_dim) in target_shape.iter().enumerate() {
        let orig_dim = orig_shape.get(i).copied().unwrap_or(1);
        if orig_dim == 1 && target_dim > 1 {
            broadcast_strides.push(0);
        } else if let Some(&s) = orig_strides.get(i) {
            broadcast_strides.push(s);
        } else {
            broadcast_strides.push(0);
        }
    }

    broadcast_strides
}

/// Zero-allocation coordinate iterator that yields corresponding `(offset_a, offset_b)` pairs
/// for two broadcasted arrays in column-major order.
pub struct BroadcastPairIterator<'a> {
    target_shape: &'a [usize],
    strides_a: Vec<usize>,
    strides_b: Vec<usize>,
    coords: Vec<usize>,
    total_elements: usize,
    current_index: usize,
}

impl<'a> BroadcastPairIterator<'a> {
    /// Creates a new broadcast iterator for two operand shapes targeting `target_shape`.
    pub fn new(
        shape_a: &[usize],
        strides_a: &[usize],
        shape_b: &[usize],
        strides_b: &[usize],
        target_shape: &'a [usize],
    ) -> Self {
        let b_strides_a = compute_broadcast_strides(shape_a, target_shape, strides_a);
        let b_strides_b = compute_broadcast_strides(shape_b, target_shape, strides_b);
        let total = if target_shape.contains(&0) {
            0
        } else {
            target_shape.iter().product()
        };

        Self {
            target_shape,
            strides_a: b_strides_a,
            strides_b: b_strides_b,
            coords: vec![0; target_shape.len()],
            total_elements: total,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for BroadcastPairIterator<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.total_elements {
            return None;
        }

        // Compute current offsets from coordinates and broadcast strides
        let mut offset_a = 0;
        let mut offset_b = 0;
        for (i, &c) in self.coords.iter().enumerate() {
            offset_a += c * self.strides_a[i];
            offset_b += c * self.strides_b[i];
        }

        // Advance coordinates in column-major order: dimension 0 increments first!
        for dim_idx in 0..self.target_shape.len() {
            self.coords[dim_idx] += 1;
            if self.coords[dim_idx] < self.target_shape[dim_idx] {
                break;
            }
            self.coords[dim_idx] = 0;
        }

        self.current_index += 1;
        Some((offset_a, offset_b))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.total_elements.saturating_sub(self.current_index);
        (rem, Some(rem))
    }
}

impl<'a> ExactSizeIterator for BroadcastPairIterator<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::compute_column_major_strides;

    #[test]
    fn test_broadcast_shapes() {
        // Scalar [1, 1] with [2, 3] -> [2, 3]
        assert_eq!(broadcast_shapes(&[1, 1], &[2, 3]).unwrap(), vec![2, 3]);

        // Column vector [3, 1] with Row vector [1, 2] -> [3, 2] (outer product)
        assert_eq!(broadcast_shapes(&[3, 1], &[1, 2]).unwrap(), vec![3, 2]);

        // Matrix [2, 3] with Row vector [1, 3] -> [2, 3]
        assert_eq!(broadcast_shapes(&[2, 3], &[1, 3]).unwrap(), vec![2, 3]);

        // Incompatible: [2, 3] with [2, 4] -> Error
        assert!(broadcast_shapes(&[2, 3], &[2, 4]).is_err());
    }

    #[test]
    fn test_broadcast_iterator_outer_sum() {
        let shape_a = [3, 1];
        let strides_a = compute_column_major_strides(&shape_a);
        let shape_b = [1, 2];
        let strides_b = compute_column_major_strides(&shape_b);
        let target_shape = [3, 2];

        let iter =
            BroadcastPairIterator::new(&shape_a, &strides_a, &shape_b, &strides_b, &target_shape);
        let pairs: Vec<_> = iter.collect();

        // 3x2 matrix in col-major:
        // (0,0): A(0,0), B(0,0) -> offset_a = 0, offset_b = 0
        // (1,0): A(1,0), B(0,0) -> offset_a = 1, offset_b = 0
        // (2,0): A(2,0), B(0,0) -> offset_a = 2, offset_b = 0
        // (0,1): A(0,0), B(0,1) -> offset_a = 0, offset_b = 1
        // (1,1): A(1,0), B(0,1) -> offset_a = 1, offset_b = 1
        // (2,1): A(2,0), B(0,1) -> offset_a = 2, offset_b = 1
        assert_eq!(pairs, vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)]);
    }
}
