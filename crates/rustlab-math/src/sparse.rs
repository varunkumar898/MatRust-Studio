//! Compressed Sparse Column (CSC) matrix representation for rustlab.
//!
//! Follows standard MATLAB sparse semantics:
//! - 0-based internal indexing with 1-based MATLAB interface
//! - Column-major Compressed Sparse Column format
//! - Duplicate triplet accumulation during sparse construction
//! - Vectorized sparse-dense and sparse-sparse linear algebra

use crate::array::DenseArray;
use crate::error::MathError;
use std::fmt;

/// A Compressed Sparse Column (CSC) 2D matrix of element type `T`.
#[derive(Debug, Clone, PartialEq)]
pub struct CscMatrix<T> {
    pub shape: [usize; 2],
    /// Column pointers of length `shape[1] + 1`.
    pub col_ptr: Vec<usize>,
    /// Row indices of length `nnz`.
    pub row_ind: Vec<usize>,
    /// Nonzero values of length `nnz`.
    pub values: Vec<T>,
}

impl<T: Copy + Default + PartialEq> CscMatrix<T> {
    /// Constructs an empty $M \times N$ sparse matrix with zero nonzeros.
    pub fn new(m: usize, n: usize) -> Self {
        Self {
            shape: [m, n],
            col_ptr: vec![0; n + 1],
            row_ind: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Number of rows.
    #[inline]
    pub fn rows(&self) -> usize {
        self.shape[0]
    }

    /// Number of columns.
    #[inline]
    pub fn cols(&self) -> usize {
        self.shape[1]
    }

    /// Number of non-zero elements.
    #[inline]
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the sparse matrix has zero nonzeros or empty shape.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rows() == 0 || self.cols() == 0 || self.values.is_empty()
    }
}

impl CscMatrix<f64> {
    /// Constructs a sparse matrix from coordinate triplets `(rows, cols, values)` with shape `[m, n]`.
    ///
    /// 1-based indexing is expected for input `rows` and `cols`.
    /// Duplicate entries `(i, j)` are accumulated by summation, exactly matching MATLAB's `sparse(i,j,v,m,n)`.
    pub fn from_triplets_1based(
        m: usize,
        n: usize,
        rows: &[usize],
        cols: &[usize],
        vals: &[f64],
    ) -> Result<Self, MathError> {
        let len = rows.len();
        if cols.len() != len || vals.len() != len {
            return Err(MathError::DimensionMismatch {
                identifier: "MATLAB:sparse:tripletLengthMismatch",
                shape_a: vec![rows.len()],
                shape_b: vec![cols.len()],
            });
        }

        // Validate 1-based bounds and filter out zero values
        let mut triplets: Vec<(usize, usize, f64)> = Vec::with_capacity(len);
        for k in 0..len {
            let r = rows[k];
            let c = cols[k];
            let v = vals[k];
            if r == 0 || r > m || c == 0 || c > n {
                return Err(MathError::IndexOutOfBounds {
                    identifier: "MATLAB:sparse:indexOutOfBounds",
                    index: if r == 0 || r > m { r } else { c },
                    max_len: if r == 0 || r > m { m } else { n },
                });
            }
            triplets.push((c - 1, r - 1, v)); // Store as (col, row, val) for column-major sorting
        }

        // Sort by column then row
        triplets.sort_by(|a, b| {
            if a.0 != b.0 {
                a.0.cmp(&b.0)
            } else {
                a.1.cmp(&b.1)
            }
        });

        // Accumulate duplicates and construct CSC arrays
        let mut col_ptr = vec![0; n + 1];
        let mut row_ind = Vec::new();
        let mut values = Vec::new();

        let mut curr_col = 0;
        let mut idx = 0;
        while idx < triplets.len() {
            let (c, r, mut sum_val) = triplets[idx];

            // Advance empty columns
            while curr_col < c {
                curr_col += 1;
                col_ptr[curr_col] = row_ind.len();
            }

            // Sum duplicates
            idx += 1;
            while idx < triplets.len() && triplets[idx].0 == c && triplets[idx].1 == r {
                sum_val += triplets[idx].2;
                idx += 1;
            }

            // Retain only non-zero entries (or explicitly stored entries)
            if sum_val != 0.0 {
                row_ind.push(r);
                values.push(sum_val);
            }
        }

        while curr_col < n {
            curr_col += 1;
            col_ptr[curr_col] = row_ind.len();
        }

        Ok(Self {
            shape: [m, n],
            col_ptr,
            row_ind,
            values,
        })
    }

    /// Constructs a sparse matrix from a dense 2D array.
    pub fn from_dense(dense: &DenseArray<f64>) -> Result<Self, MathError> {
        let shp = dense.shape();
        if shp.len() != 2 {
            return Err(MathError::DimensionMismatch {
                identifier: "MATLAB:sparse:matrixRequired",
                shape_a: shp.to_vec(),
                shape_b: vec![shp[0], 1],
            });
        }
        let m = shp[0];
        let n = shp[1];
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_ind = Vec::new();
        let mut values = Vec::new();

        col_ptr.push(0);
        for c in 0..n {
            for r in 0..m {
                let val = dense.as_slice()[r + c * m];
                if val != 0.0 {
                    row_ind.push(r);
                    values.push(val);
                }
            }
            col_ptr.push(row_ind.len());
        }

        Ok(Self {
            shape: [m, n],
            col_ptr,
            row_ind,
            values,
        })
    }

    /// Converts the sparse matrix into a full dense 2D array.
    pub fn to_dense(&self) -> DenseArray<f64> {
        let m = self.rows();
        let n = self.cols();
        let mut data = vec![0.0; m * n];

        for c in 0..n {
            let start = self.col_ptr[c];
            let end = self.col_ptr[c + 1];
            for i in start..end {
                let r = self.row_ind[i];
                let v = self.values[i];
                data[r + c * m] = v;
            }
        }
        DenseArray::from_vec_unchecked(vec![m, n], data)
    }

    /// Sets an entry at 1-based `(r, c)`. Inserts, updates, or removes (if val == 0.0).
    pub fn set_1based(&mut self, r: usize, c: usize, val: f64) -> Result<(), MathError> {
        if r == 0 || r > self.rows() || c == 0 || c > self.cols() {
            return Err(MathError::IndexOutOfBounds {
                identifier: "MATLAB:badsubscript",
                index: if r == 0 || r > self.rows() { r } else { c },
                max_len: if r == 0 || r > self.rows() {
                    self.rows()
                } else {
                    self.cols()
                },
            });
        }
        let col = c - 1;
        let row = r - 1;
        let start = self.col_ptr[col];
        let end = self.col_ptr[col + 1];

        for i in start..end {
            if self.row_ind[i] == row {
                if val == 0.0 {
                    self.row_ind.remove(i);
                    self.values.remove(i);
                    for p in (col + 1)..=self.cols() {
                        self.col_ptr[p] -= 1;
                    }
                } else {
                    self.values[i] = val;
                }
                return Ok(());
            }
            if self.row_ind[i] > row {
                if val != 0.0 {
                    self.row_ind.insert(i, row);
                    self.values.insert(i, val);
                    for p in (col + 1)..=self.cols() {
                        self.col_ptr[p] += 1;
                    }
                }
                return Ok(());
            }
        }

        if val != 0.0 {
            self.row_ind.insert(end, row);
            self.values.insert(end, val);
            for p in (col + 1)..=self.cols() {
                self.col_ptr[p] += 1;
            }
        }
        Ok(())
    }

    /// Multiplies sparse matrix by a scalar: `s * S`.
    pub fn scale(&self, scalar: f64) -> Self {
        if scalar == 0.0 {
            return Self::new(self.rows(), self.cols());
        }
        Self {
            shape: self.shape,
            col_ptr: self.col_ptr.clone(),
            row_ind: self.row_ind.clone(),
            values: self.values.iter().map(|&v| v * scalar).collect(),
        }
    }

    /// Sparse identity matrix `speye(m, n)`.
    pub fn speye(m: usize, n: usize) -> Self {
        let k = m.min(n);
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_ind = Vec::with_capacity(k);
        let mut values = Vec::with_capacity(k);

        col_ptr.push(0);
        for c in 0..n {
            if c < k {
                row_ind.push(c);
                values.push(1.0);
            }
            col_ptr.push(row_ind.len());
        }

        Self {
            shape: [m, n],
            col_ptr,
            row_ind,
            values,
        }
    }

    /// Retrieves an entry at 1-based `(r, c)`. Returns `0.0` if not present.
    pub fn get_1based(&self, r: usize, c: usize) -> Result<f64, MathError> {
        if r == 0 || r > self.rows() || c == 0 || c > self.cols() {
            return Err(MathError::IndexOutOfBounds {
                identifier: "MATLAB:badsubscript",
                index: if r == 0 || r > self.rows() { r } else { c },
                max_len: if r == 0 || r > self.rows() {
                    self.rows()
                } else {
                    self.cols()
                },
            });
        }
        let col = c - 1;
        let row = r - 1;
        let start = self.col_ptr[col];
        let end = self.col_ptr[col + 1];

        for i in start..end {
            if self.row_ind[i] == row {
                return Ok(self.values[i]);
            }
            if self.row_ind[i] > row {
                break;
            }
        }
        Ok(0.0)
    }

    /// Transpose of the sparse matrix: `A'`.
    pub fn transpose(&self) -> Self {
        let m = self.rows();
        let n = self.cols();
        let nnz = self.nnz();

        let mut row_counts = vec![0; m];
        for &r in &self.row_ind {
            row_counts[r] += 1;
        }

        let mut col_ptr = Vec::with_capacity(m + 1);
        col_ptr.push(0);
        for count in &row_counts {
            col_ptr.push(col_ptr.last().copied().unwrap_or(0) + count);
        }

        let mut row_ind = vec![0; nnz];
        let mut values = vec![0.0; nnz];
        let mut current_offset = col_ptr[..m].to_vec();

        for c in 0..n {
            let start = self.col_ptr[c];
            let end = self.col_ptr[c + 1];
            for i in start..end {
                let r = self.row_ind[i];
                let v = self.values[i];
                let dest = current_offset[r];
                row_ind[dest] = c;
                values[dest] = v;
                current_offset[r] += 1;
            }
        }

        Self {
            shape: [n, m],
            col_ptr,
            row_ind,
            values,
        }
    }

    /// Sparse-dense matrix multiplication: `C = self * dense`.
    pub fn matmul_dense(&self, dense: &DenseArray<f64>) -> Result<DenseArray<f64>, MathError> {
        let d_shp = dense.shape();
        if d_shp.len() != 2 || d_shp[0] != self.cols() {
            return Err(MathError::InnerDimensionMismatch {
                identifier: "MATLAB:innerdim",
                cols_a: self.cols(),
                rows_b: d_shp[0],
            });
        }
        let m = self.rows();
        let p = d_shp[1];
        let mut out = vec![0.0; m * p];

        for k in 0..p {
            for c in 0..self.cols() {
                let b_val = dense.as_slice()[c + k * self.cols()];
                if b_val != 0.0 {
                    let start = self.col_ptr[c];
                    let end = self.col_ptr[c + 1];
                    for i in start..end {
                        let r = self.row_ind[i];
                        let a_val = self.values[i];
                        out[r + k * m] += a_val * b_val;
                    }
                }
            }
        }
        DenseArray::from_vec(vec![m, p], out)
    }

    /// Sparse-sparse matrix multiplication: `C = self * other`.
    pub fn matmul_sparse(&self, other: &Self) -> Result<Self, MathError> {
        if self.cols() != other.rows() {
            return Err(MathError::InnerDimensionMismatch {
                identifier: "MATLAB:innerdim",
                cols_a: self.cols(),
                rows_b: other.rows(),
            });
        }
        let m = self.rows();
        let p = other.cols();

        let mut col_ptr = Vec::with_capacity(p + 1);
        let mut row_ind = Vec::new();
        let mut values = Vec::new();
        col_ptr.push(0);

        let mut accumulator = vec![0.0; m];
        let mut marker = vec![false; m];
        let mut active_rows = Vec::new();

        for j in 0..p {
            active_rows.clear();
            let b_start = other.col_ptr[j];
            let b_end = other.col_ptr[j + 1];

            for b_idx in b_start..b_end {
                let k = other.row_ind[b_idx];
                let b_val = other.values[b_idx];

                let a_start = self.col_ptr[k];
                let a_end = self.col_ptr[k + 1];
                for a_idx in a_start..a_end {
                    let r = self.row_ind[a_idx];
                    let a_val = self.values[a_idx];

                    if !marker[r] {
                        marker[r] = true;
                        accumulator[r] = a_val * b_val;
                        active_rows.push(r);
                    } else {
                        accumulator[r] += a_val * b_val;
                    }
                }
            }

            active_rows.sort_unstable();
            for &r in &active_rows {
                let val = accumulator[r];
                if val != 0.0 {
                    row_ind.push(r);
                    values.push(val);
                }
                marker[r] = false;
                accumulator[r] = 0.0;
            }
            col_ptr.push(row_ind.len());
        }

        Ok(Self {
            shape: [m, p],
            col_ptr,
            row_ind,
            values,
        })
    }

    /// Sparse-sparse addition: `C = self + other`.
    pub fn add_sparse(&self, other: &Self) -> Result<Self, MathError> {
        if self.shape != other.shape {
            return Err(MathError::DimensionMismatch {
                identifier: "MATLAB:dimagree",
                shape_a: self.shape.to_vec(),
                shape_b: other.shape.to_vec(),
            });
        }
        let m = self.rows();
        let n = self.cols();

        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_ind = Vec::new();
        let mut values = Vec::new();
        col_ptr.push(0);

        for c in 0..n {
            let mut i_a = self.col_ptr[c];
            let end_a = self.col_ptr[c + 1];
            let mut i_b = other.col_ptr[c];
            let end_b = other.col_ptr[c + 1];

            while i_a < end_a && i_b < end_b {
                let r_a = self.row_ind[i_a];
                let r_b = other.row_ind[i_b];
                if r_a < r_b {
                    row_ind.push(r_a);
                    values.push(self.values[i_a]);
                    i_a += 1;
                } else if r_b < r_a {
                    row_ind.push(r_b);
                    values.push(other.values[i_b]);
                    i_b += 1;
                } else {
                    let sum = self.values[i_a] + other.values[i_b];
                    if sum != 0.0 {
                        row_ind.push(r_a);
                        values.push(sum);
                    }
                    i_a += 1;
                    i_b += 1;
                }
            }

            while i_a < end_a {
                row_ind.push(self.row_ind[i_a]);
                values.push(self.values[i_a]);
                i_a += 1;
            }

            while i_b < end_b {
                row_ind.push(other.row_ind[i_b]);
                values.push(other.values[i_b]);
                i_b += 1;
            }

            col_ptr.push(row_ind.len());
        }

        Ok(Self {
            shape: [m, n],
            col_ptr,
            row_ind,
            values,
        })
    }

    /// Replaces nonzeros with 1.0: `spones(S)`.
    pub fn spones(&self) -> Self {
        Self {
            shape: self.shape,
            col_ptr: self.col_ptr.clone(),
            row_ind: self.row_ind.clone(),
            values: vec![1.0; self.values.len()],
        }
    }

    /// Returns non-zero triplets as `(rows_1based, cols_1based, values)`.
    pub fn find_triplets(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let nnz = self.nnz();
        let mut rows = Vec::with_capacity(nnz);
        let mut cols = Vec::with_capacity(nnz);
        let mut vals = Vec::with_capacity(nnz);

        for c in 0..self.cols() {
            let start = self.col_ptr[c];
            let end = self.col_ptr[c + 1];
            for i in start..end {
                rows.push((self.row_ind[i] + 1) as f64);
                cols.push((c + 1) as f64);
                vals.push(self.values[i]);
            }
        }
        (rows, cols, vals)
    }

    /// Formats the sparse matrix according to MATLAB's standard sparse output:
    /// `  (row, col)   value` or `   All zero sparse: m-by-n`
    pub fn format_matlab_display(&self) -> String {
        if self.nnz() == 0 {
            return format!("   All zero sparse: {}-by-{}\n", self.rows(), self.cols());
        }
        let mut out = String::new();
        for c in 0..self.cols() {
            let start = self.col_ptr[c];
            let end = self.col_ptr[c + 1];
            for i in start..end {
                let r = self.row_ind[i] + 1;
                let col = c + 1;
                let v = self.values[i];
                if v.fract() == 0.0 {
                    out.push_str(&format!("  ({r},{col})   {v:.0}\n"));
                } else {
                    out.push_str(&format!("  ({r},{col})   {v:.4}\n"));
                }
            }
        }
        out
    }
}

impl fmt::Display for CscMatrix<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_matlab_display())
    }
}
