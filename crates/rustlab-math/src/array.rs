//! Foundational N-dimensional column-major dense tensor.

use crate::error::MathError;
use crate::shape::{
    compute_column_major_strides, coords_to_offset, normalize_shape, numel, offset_to_coords,
};
use std::fmt;

/// A column-major (Fortran-order) multidimensional dense array.
#[derive(Debug, Clone, PartialEq)]
pub struct DenseArray<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>,
}

impl<T> DenseArray<T> {
    /// Creates a `DenseArray` from a shape and linear column-major data vector.
    pub fn from_vec(shape: Vec<usize>, data: Vec<T>) -> Result<Self, MathError> {
        let norm_shape = normalize_shape(&shape);
        let expected_numel = numel(&norm_shape);
        if data.len() != expected_numel {
            return Err(MathError::reshape_numel(data.len(), expected_numel));
        }
        let strides = compute_column_major_strides(&norm_shape);
        Ok(Self {
            data,
            shape: norm_shape,
            strides,
        })
    }

    /// Creates a `DenseArray` bypassing numel checks in release mode,
    /// with debug assertion ensuring length matches the normalized shape.
    pub fn from_vec_unchecked(shape: Vec<usize>, data: Vec<T>) -> Self {
        let norm_shape = normalize_shape(&shape);
        debug_assert_eq!(
            data.len(),
            numel(&norm_shape),
            "from_vec_unchecked: data length does not match shape numel"
        );
        let strides = compute_column_major_strides(&norm_shape);
        Self {
            data,
            shape: norm_shape,
            strides,
        }
    }

    /// Creates an empty $0 \times 0$ matrix.
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            shape: vec![0, 0],
            strides: vec![1, 0],
        }
    }

    /// Creates a $1 \times 1$ scalar array.
    pub fn scalar(val: T) -> Self {
        Self {
            data: vec![val],
            shape: vec![1, 1],
            strides: vec![1, 1],
        }
    }

    /// Creates a $1 \times N$ row vector.
    pub fn row_vector(data: Vec<T>) -> Self {
        let n = data.len();
        let shape = vec![1, n];
        let strides = vec![1, 1];
        Self {
            data,
            shape,
            strides,
        }
    }

    /// Creates an $N \times 1$ column vector.
    pub fn col_vector(data: Vec<T>) -> Self {
        let n = data.len();
        let shape = vec![n, 1];
        let strides = vec![1, n];
        Self {
            data,
            shape,
            strides,
        }
    }

    /// Returns the shape slice of the array.
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Returns the column-major strides slice.
    pub fn strides(&self) -> &[usize] {
        &self.strides
    }

    /// Returns the total number of elements.
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of dimensions (always $\ge 2$).
    pub fn ndims(&self) -> usize {
        self.shape.len()
    }

    /// Returns true if the array has 0 elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns true if the array is a $1 \times 1$ scalar.
    pub fn is_scalar(&self) -> bool {
        self.shape == [1, 1]
    }

    /// Returns true if the array is a $1 \times N$ or $N \times 1$ vector.
    pub fn is_vector(&self) -> bool {
        self.ndims() == 2 && (self.shape[0] == 1 || self.shape[1] == 1)
    }

    /// Returns true if the array has exactly 2 dimensions.
    pub fn is_matrix(&self) -> bool {
        self.ndims() == 2
    }

    /// Underlying slice of data elements in column-major order.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Underlying mutable slice of data elements in column-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Consumes the array and returns the raw linear vector.
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Transforms elements with a mapping function `f`, preserving array shape and strides.
    pub fn map<U, F>(&self, f: F) -> DenseArray<U>
    where
        U: Clone + Default + PartialEq + fmt::Debug,
        F: FnMut(&T) -> U,
    {
        let new_data: Vec<U> = self.data.iter().map(f).collect();
        DenseArray {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }

    // --- 1-Based Indexing ---

    /// Retrieves an element by 1-based linear index.
    pub fn get_linear(&self, index: usize) -> Result<&T, MathError> {
        if index == 0 || index > self.data.len() {
            Err(MathError::index_out_of_bounds(index, self.data.len()))
        } else {
            Ok(&self.data[index - 1])
        }
    }

    /// Retrieves a mutable reference to an element by 1-based linear index.
    pub fn get_linear_mut(&mut self, index: usize) -> Result<&mut T, MathError> {
        let max_len = self.data.len();
        if index == 0 || index > max_len {
            Err(MathError::index_out_of_bounds(index, max_len))
        } else {
            Ok(&mut self.data[index - 1])
        }
    }

    /// Retrieves an element by 1-based subscript coordinates.
    pub fn get_subscript(&self, subscripts: &[usize]) -> Result<&T, MathError> {
        if subscripts.is_empty() {
            return Err(MathError::invalid_subscript(0));
        }
        if subscripts.len() == 1 {
            return self.get_linear(subscripts[0]);
        }

        let k = subscripts.len();
        if k < self.shape.len() {
            // Trailing subscript spans the product of remaining dimensions
            let last_dim_len: usize = self.shape[k - 1..].iter().product();
            let mut offset = 0;
            for (i, &sub) in subscripts.iter().enumerate() {
                let limit = if i == k - 1 {
                    last_dim_len
                } else {
                    self.shape[i]
                };
                if sub == 0 || sub > limit {
                    return Err(MathError::index_out_of_bounds(sub, limit));
                }
                offset += (sub - 1) * self.strides[i];
            }
            return Ok(&self.data[offset]);
        }

        if k > self.shape.len() {
            for &sub in &subscripts[self.shape.len()..] {
                if sub != 1 {
                    return Err(MathError::index_out_of_bounds(sub, 1));
                }
            }
        }

        // Validate 1-based bounds and convert to 0-based coordinates
        let mut zero_based_coords = Vec::with_capacity(self.shape.len());
        for (i, &sub) in subscripts.iter().take(self.shape.len()).enumerate() {
            if sub == 0 || sub > self.shape[i] {
                return Err(MathError::index_out_of_bounds(sub, self.shape[i]));
            }
            zero_based_coords.push(sub - 1);
        }

        let offset = coords_to_offset(&zero_based_coords, &self.strides);
        Ok(&self.data[offset])
    }

    /// Retrieves a mutable reference to an element by 1-based subscript coordinates.
    pub fn get_subscript_mut(&mut self, subscripts: &[usize]) -> Result<&mut T, MathError> {
        if subscripts.is_empty() {
            return Err(MathError::invalid_subscript(0));
        }
        if subscripts.len() == 1 {
            return self.get_linear_mut(subscripts[0]);
        }

        let k = subscripts.len();
        if k < self.shape.len() {
            let last_dim_len: usize = self.shape[k - 1..].iter().product();
            let mut offset = 0;
            for (i, &sub) in subscripts.iter().enumerate() {
                let limit = if i == k - 1 {
                    last_dim_len
                } else {
                    self.shape[i]
                };
                if sub == 0 || sub > limit {
                    return Err(MathError::index_out_of_bounds(sub, limit));
                }
                offset += (sub - 1) * self.strides[i];
            }
            return Ok(&mut self.data[offset]);
        }

        if k > self.shape.len() {
            for &sub in &subscripts[self.shape.len()..] {
                if sub != 1 {
                    return Err(MathError::index_out_of_bounds(sub, 1));
                }
            }
        }

        let mut zero_based_coords = Vec::with_capacity(self.shape.len());
        for (i, &sub) in subscripts.iter().take(self.shape.len()).enumerate() {
            if sub == 0 || sub > self.shape[i] {
                return Err(MathError::index_out_of_bounds(sub, self.shape[i]));
            }
            zero_based_coords.push(sub - 1);
        }

        let offset = coords_to_offset(&zero_based_coords, &self.strides);
        Ok(&mut self.data[offset])
    }

    /// Reshapes the array to a new shape preserving total element count in column-major order.
    pub fn reshape(&self, new_shape: &[usize]) -> Result<Self, MathError>
    where
        T: Clone,
    {
        let norm_new_shape = normalize_shape(new_shape);
        let req_numel = numel(&norm_new_shape);
        if req_numel != self.data.len() {
            return Err(MathError::reshape_numel(self.data.len(), req_numel));
        }
        let strides = compute_column_major_strides(&norm_new_shape);
        Ok(Self {
            data: self.data.clone(),
            shape: norm_new_shape,
            strides,
        })
    }

    /// Extracts elements matching a boolean mask array into a vector.
    /// If the array is a $1 \times N$ row vector, produces a row vector; otherwise a column vector.
    pub fn get_mask(&self, mask: &DenseArray<bool>) -> Result<Self, MathError>
    where
        T: Clone,
    {
        if mask.numel() != self.numel() {
            return Err(MathError::mask_size_mismatch());
        }
        let mut selected = Vec::new();
        for (idx, &is_true) in mask.as_slice().iter().enumerate() {
            if is_true {
                selected.push(self.data[idx].clone());
            }
        }
        let n = selected.len();
        let out_shape = if self.shape.len() == 2 && self.shape[0] == 1 {
            vec![1, n]
        } else {
            vec![n, 1]
        };
        Self::from_vec(out_shape, selected)
    }

    /// Sets elements matching a boolean mask to either a scalar or matching vector values.
    pub fn set_mask(&mut self, mask: &DenseArray<bool>, values: &[T]) -> Result<(), MathError>
    where
        T: Clone,
    {
        if mask.numel() != self.numel() {
            return Err(MathError::mask_size_mismatch());
        }
        let true_count = mask.as_slice().iter().filter(|&&b| b).count();
        if values.len() == 1 {
            let fill_val = values[0].clone();
            for (idx, &is_true) in mask.as_slice().iter().enumerate() {
                if is_true {
                    self.data[idx] = fill_val.clone();
                }
            }
            Ok(())
        } else if values.len() == true_count {
            let mut val_idx = 0;
            for (idx, &is_true) in mask.as_slice().iter().enumerate() {
                if is_true {
                    self.data[idx] = values[val_idx].clone();
                    val_idx += 1;
                }
            }
            Ok(())
        } else {
            Err(MathError::assignment_count_mismatch())
        }
    }
}

impl<T: Default + Clone> DenseArray<T> {
    /// Allocates an array of the given shape initialized with `T::default()`.
    pub fn zeros(shape: &[usize]) -> Self {
        let norm_shape = normalize_shape(shape);
        let total = numel(&norm_shape);
        let data = vec![T::default(); total];
        let strides = compute_column_major_strides(&norm_shape);
        Self {
            data,
            shape: norm_shape,
            strides,
        }
    }

    /// Assigns a value at 1-based linear index, automatically growing the array if out-of-bounds!
    pub fn set_linear_grow(&mut self, index: usize, val: T) -> Result<(), MathError> {
        if index == 0 {
            return Err(MathError::invalid_subscript(0));
        }

        if index > self.data.len() {
            // Grow data buffer with default padding
            self.data.resize(index, T::default());
            // Update shape
            if self.is_empty() || (self.shape[0] <= 1 && self.shape[1] <= 1) {
                // Was scalar or empty -> becomes 1xN row vector
                self.shape = vec![1, index];
                self.strides = vec![1, 1];
            } else if self.shape[0] == 1 {
                // Row vector grows along column dimension
                self.shape[1] = index;
            } else if self.shape[1] == 1 {
                // Column vector grows along row dimension
                self.shape[0] = index;
                self.strides[1] = index;
            } else {
                // 2D matrix or higher: treated as linear array of shape [1, index]
                self.shape = vec![1, index];
                self.strides = vec![1, 1];
            }
        }

        self.data[index - 1] = val;
        Ok(())
    }

    /// Assigns a value at 1-based subscript coordinates, automatically growing the dimensions if out-of-bounds!
    pub fn set_subscript_grow(&mut self, subscripts: &[usize], val: T) -> Result<(), MathError> {
        if subscripts.contains(&0) {
            return Err(MathError::invalid_subscript(0));
        }

        if subscripts.len() == 1 {
            return self.set_linear_grow(subscripts[0], val);
        }

        let mut needs_growth = subscripts.len() > self.shape.len();
        let mut new_shape = self.shape.clone();
        if subscripts.len() > new_shape.len() {
            new_shape.resize(subscripts.len(), 1);
        }

        for (i, &sub) in subscripts.iter().enumerate() {
            if sub > new_shape[i] {
                new_shape[i] = sub;
                needs_growth = true;
            }
        }

        if needs_growth {
            new_shape = normalize_shape(&new_shape);
            let new_total = numel(&new_shape);
            let new_strides = compute_column_major_strides(&new_shape);
            let mut new_data = vec![T::default(); new_total];

            // Re-map existing elements from old coordinates to new coordinates
            for old_offset in 0..self.data.len() {
                let coords = offset_to_coords(old_offset, &self.shape);
                let new_offset = coords_to_offset(&coords, &new_strides);
                new_data[new_offset] = self.data[old_offset].clone();
            }

            self.data = new_data;
            self.shape = new_shape;
            self.strides = new_strides;
        }

        let mut zero_based_coords = Vec::with_capacity(subscripts.len());
        for &sub in subscripts.iter() {
            zero_based_coords.push(sub - 1);
        }

        let offset = coords_to_offset(&zero_based_coords, &self.strides);
        self.data[offset] = val;
        Ok(())
    }
}

impl<T: Clone> DenseArray<T> {
    /// Non-conjugate 2D array transpose (`A.'`): swaps rows and columns.
    pub fn transpose_2d(&self) -> Result<Self, MathError> {
        if !self.is_matrix() {
            return Err(MathError::UnsupportedOperation {
                op: "transpose_2d",
                type_name: "multidimensional array",
            });
        }
        let rows = self.shape[0];
        let cols = self.shape[1];
        let mut new_data = Vec::with_capacity(self.data.len());

        // In col-major, output row c and col r has element input (r, c)
        for r in 0..rows {
            for c in 0..cols {
                let in_offset = c * self.strides[1] + r;
                new_data.push(self.data[in_offset].clone());
            }
        }

        let new_shape = vec![cols, rows];
        let new_strides = compute_column_major_strides(&new_shape);
        Ok(Self {
            data: new_data,
            shape: new_shape,
            strides: new_strides,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_array_creation_and_indexing() {
        // Create 2x3 matrix:
        // [ 10 30 50 ]
        // [ 20 40 60 ]
        let data = vec![10, 20, 30, 40, 50, 60];
        let arr = DenseArray::from_vec(vec![2, 3], data).unwrap();

        assert_eq!(arr.shape(), &[2, 3]);
        assert_eq!(arr.numel(), 6);

        // 1-based linear indexing
        assert_eq!(*arr.get_linear(1).unwrap(), 10);
        assert_eq!(*arr.get_linear(2).unwrap(), 20);
        assert_eq!(*arr.get_linear(3).unwrap(), 30);
        assert_eq!(*arr.get_linear(6).unwrap(), 60);
        assert!(arr.get_linear(0).is_err());
        assert!(arr.get_linear(7).is_err());

        // 1-based subscript indexing
        assert_eq!(*arr.get_subscript(&[1, 1]).unwrap(), 10);
        assert_eq!(*arr.get_subscript(&[2, 1]).unwrap(), 20);
        assert_eq!(*arr.get_subscript(&[1, 2]).unwrap(), 30);
        assert_eq!(*arr.get_subscript(&[2, 3]).unwrap(), 60);
    }

    #[test]
    fn test_dynamic_array_growth() {
        let mut arr = DenseArray::row_vector(vec![1, 2]);
        arr.set_linear_grow(5, 10).unwrap();

        assert_eq!(arr.shape(), &[1, 5]);
        assert_eq!(arr.as_slice(), &[1, 2, 0, 0, 10]);

        // Grow via 2D subscript
        let mut m = DenseArray::from_vec(vec![2, 2], vec![1, 2, 3, 4]).unwrap();
        // Set element at (3, 3) = 99
        m.set_subscript_grow(&[3, 3], 99).unwrap();
        assert_eq!(m.shape(), &[3, 3]);
        assert_eq!(*m.get_subscript(&[3, 3]).unwrap(), 99);
        assert_eq!(*m.get_subscript(&[1, 1]).unwrap(), 1);
        assert_eq!(*m.get_subscript(&[3, 1]).unwrap(), 0); // Padded with 0
    }

    #[test]
    fn test_transpose() {
        let m = DenseArray::from_vec(vec![2, 3], vec![1, 2, 3, 4, 5, 6]).unwrap();
        let mt = m.transpose_2d().unwrap();
        assert_eq!(mt.shape(), &[3, 2]);
        // Original:
        // [1 3 5]
        // [2 4 6]
        // Transposed:
        // [1 2]
        // [3 4]
        // [5 6]
        // Col-major storage of transposed:
        // Col 1: 1, 3, 5
        // Col 2: 2, 4, 6
        assert_eq!(mt.as_slice(), &[1, 3, 5, 2, 4, 6]);
    }
}
