use matlab_diagnostics::{Diagnostic, Span};
use std::fmt;

/// A 2D dense matrix of double-precision floating point numbers.
/// In MATLAB semantics, standard arrays are 2D (or N-D) and indexed starting at 1.
#[derive(Debug, Clone, PartialEq)]
pub struct DenseMatrix {
    pub rows: usize,
    pub cols: usize,
    /// Stored in row-major layout internally for cache-friendly element iteration and display.
    pub data: Vec<f64>,
}

impl DenseMatrix {
    pub fn empty() -> Self {
        Self {
            rows: 0,
            cols: 0,
            data: Vec::new(),
        }
    }

    pub fn scalar(val: f64) -> Self {
        Self {
            rows: 1,
            cols: 1,
            data: vec![val],
        }
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn ones(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![1.0; rows * cols],
        }
    }

    pub fn eye(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Self {
            rows: n,
            cols: n,
            data,
        }
    }

    pub fn from_rows(rows_data: Vec<Vec<f64>>, span: Option<Span>) -> Result<Self, Diagnostic> {
        if rows_data.is_empty() {
            return Ok(Self::empty());
        }
        let cols = rows_data[0].len();
        let rows = rows_data.len();
        let mut data = Vec::with_capacity(rows * cols);

        for (r_idx, row) in rows_data.into_iter().enumerate() {
            if row.len() != cols {
                return Err(Diagnostic::error(
                    format!(
                        "Dimensions of arrays being concatenated are not consistent: row 1 has {} elements, but row {} has {}",
                        cols, r_idx + 1, row.len()
                    ),
                    span,
                ));
            }
            data.extend(row);
        }

        Ok(Self { rows, cols, data })
    }

    #[inline]
    pub fn is_scalar(&self) -> bool {
        self.rows == 1 && self.cols == 1
    }

    #[inline]
    pub fn as_scalar(&self) -> Option<f64> {
        if self.is_scalar() {
            self.data.first().copied()
        } else {
            None
        }
    }

    /// 1-based 2D indexing: get element at (row, col)
    pub fn get_2d(&self, r1: usize, c1: usize, span: Option<Span>) -> Result<f64, Diagnostic> {
        if r1 == 0 || r1 > self.rows || c1 == 0 || c1 > self.cols {
            return Err(Diagnostic::error(
                format!(
                    "Index exceeds matrix dimensions. Index ({}, {}) out of bounds for {}x{} matrix.",
                    r1, c1, self.rows, self.cols
                ),
                span,
            ));
        }
        let r0 = r1 - 1;
        let c0 = c1 - 1;
        Ok(self.data[r0 * self.cols + c0])
    }

    /// 1-based 2D indexing: set element at (row, col)
    pub fn set_2d(
        &mut self,
        r1: usize,
        c1: usize,
        val: f64,
        span: Option<Span>,
    ) -> Result<(), Diagnostic> {
        if r1 == 0 || r1 > self.rows || c1 == 0 || c1 > self.cols {
            return Err(Diagnostic::error(
                format!(
                    "Index exceeds matrix dimensions. Cannot assign to ({}, {}) in a {}x{} matrix.",
                    r1, c1, self.rows, self.cols
                ),
                span,
            ));
        }
        let r0 = r1 - 1;
        let c0 = c1 - 1;
        self.data[r0 * self.cols + c0] = val;
        Ok(())
    }

    /// 1-based linear indexing: A(k)
    pub fn get_linear(&self, k1: usize, span: Option<Span>) -> Result<f64, Diagnostic> {
        let total = self.rows * self.cols;
        if k1 == 0 || k1 > total {
            return Err(Diagnostic::error(
                format!(
                    "Index in position 1 exceeds array bounds (index {} exceeds {}).",
                    k1, total
                ),
                span,
            ));
        }
        // In MATLAB, linear indexing is column-major:
        // k0 = col0 * rows + row0  => row0 = k0 % rows, col0 = k0 / rows
        let k0 = k1 - 1;
        let r0 = k0 % self.rows;
        let c0 = k0 / self.rows;
        Ok(self.data[r0 * self.cols + c0])
    }

    /// Format matrix to string in MATLAB disp format
    pub fn format_matlab(&self) -> String {
        if self.rows == 0 || self.cols == 0 {
            return "     []".to_string();
        }

        let mut lines = Vec::new();
        // Compute column widths
        let mut col_widths = vec![0usize; self.cols];
        let mut formatted_cells: Vec<Vec<String>> = Vec::with_capacity(self.rows);

        for r in 0..self.rows {
            let mut row_strings = Vec::with_capacity(self.cols);
            for c in 0..self.cols {
                let v = self.data[r * self.cols + c];
                let s = if v.fract() == 0.0 && v.abs() < 1e14 {
                    format!("{:.0}", v)
                } else {
                    format!("{:.4}", v)
                };
                if s.len() > col_widths[c] {
                    col_widths[c] = s.len();
                }
                row_strings.push(s);
            }
            formatted_cells.push(row_strings);
        }

        for row in formatted_cells {
            let mut row_line = String::new();
            for (c, cell) in row.into_iter().enumerate() {
                let width = col_widths[c].max(4);
                row_line.push_str(&format!("{:>width$}  ", cell, width = width));
            }
            lines.push(row_line.trim_end().to_string());
        }

        lines.join("\n")
    }
}

impl fmt::Display for DenseMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_matlab())
    }
}
