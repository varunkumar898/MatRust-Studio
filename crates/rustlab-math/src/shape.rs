//! Multidimensional shape and column-major stride utilities.

/// Normalizes a shape slice according to MATLAB rules:
/// 1. Minimum 2 dimensions: `[]` becomes `[0, 0]`, `[n]` becomes `[1, n]`.
/// 2. Trailing singleton dimensions (> 2) are removed: `[2, 3, 1]` becomes `[2, 3]`.
pub fn normalize_shape(raw: &[usize]) -> Vec<usize> {
    if raw.is_empty() {
        return vec![0, 0];
    }
    if raw.len() == 1 {
        return vec![1, raw[0]];
    }

    let mut shape = raw.to_vec();
    // Strip trailing dimensions of size 1 if ndims > 2
    while shape.len() > 2 && shape.last() == Some(&1) {
        shape.pop();
    }
    shape
}

/// Computes column-major (Fortran-order) strides for a given shape.
/// For shape `[D0, D1, D2]`, strides are `[1, D0, D0 * D1]`.
pub fn compute_column_major_strides(shape: &[usize]) -> Vec<usize> {
    if shape.is_empty() {
        return Vec::new();
    }
    let mut strides = Vec::with_capacity(shape.len());
    let mut current_stride: usize = 1;
    for &dim in shape {
        strides.push(current_stride);
        current_stride = current_stride.saturating_mul(dim.max(1));
    }
    strides
}

/// Computes the total number of elements in an array of the given shape.
pub fn numel(shape: &[usize]) -> usize {
    if shape.is_empty() || shape.contains(&0) {
        0
    } else {
        shape.iter().product()
    }
}

/// Converts an $N$-dimensional 0-based coordinate to a linear 0-based offset.
pub fn coords_to_offset(coords: &[usize], strides: &[usize]) -> usize {
    coords
        .iter()
        .zip(strides.iter())
        .map(|(&c, &s)| c * s)
        .sum()
}

/// Converts a linear 0-based offset to $N$-dimensional 0-based coordinates in column-major order.
pub fn offset_to_coords(mut offset: usize, shape: &[usize]) -> Vec<usize> {
    let mut coords = vec![0; shape.len()];
    for (i, &dim) in shape.iter().enumerate() {
        if dim == 0 {
            coords[i] = 0;
        } else {
            coords[i] = offset % dim;
            offset /= dim;
        }
    }
    coords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_shape() {
        assert_eq!(normalize_shape(&[]), vec![0, 0]);
        assert_eq!(normalize_shape(&[5]), vec![1, 5]);
        assert_eq!(normalize_shape(&[2, 3]), vec![2, 3]);
        assert_eq!(normalize_shape(&[2, 3, 1]), vec![2, 3]);
        assert_eq!(normalize_shape(&[2, 3, 1, 1]), vec![2, 3]);
        assert_eq!(normalize_shape(&[2, 3, 1, 4]), vec![2, 3, 1, 4]);
    }

    #[test]
    fn test_column_major_strides() {
        let shape = [3, 4]; // 3 rows, 4 cols
        let strides = compute_column_major_strides(&shape);
        assert_eq!(strides, vec![1, 3]);

        // (row 1, col 2) -> 0-based: row=1, col=2 -> 1*1 + 2*3 = 7
        assert_eq!(coords_to_offset(&[1, 2], &strides), 7);
        assert_eq!(offset_to_coords(7, &shape), vec![1, 2]);
    }
}
