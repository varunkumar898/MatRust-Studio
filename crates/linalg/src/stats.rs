use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::DenseMatrix;

use crate::map_binary_simple;

/// Resolve the effective reduction dimension for matrix `a`.
/// If `dim` is specified, it is validated and returned.
/// If `dim` is None:
/// - If `a` is a row vector (1 x N with N > 1), returns 2.
/// - Otherwise (column vector, scalar, matrix, or empty), returns 1.
pub fn resolve_dim(a: &DenseMatrix, dim: Option<usize>) -> usize {
    match dim {
        Some(d) => d,
        None => {
            if a.rows == 1 && a.cols > 1 {
                2
            } else {
                1
            }
        }
    }
}

/// Generic dimension reduction on 2D `DenseMatrix`.
/// For dim > 2 (singleton dimension in 2D array), each slice along dim has length 1.
pub fn reduce_dim<F>(a: &DenseMatrix, dim: usize, reduce_slice: F) -> DenseMatrix
where
    F: Fn(&[f64]) -> f64,
{
    if a.data.is_empty() {
        return DenseMatrix::scalar(reduce_slice(&[]));
    }
    if dim == 1 {
        let mut data = Vec::with_capacity(a.cols);
        let mut slice = vec![0.0; a.rows];
        for c in 0..a.cols {
            for r in 0..a.rows {
                slice[r] = a.data[r * a.cols + c];
            }
            data.push(reduce_slice(&slice));
        }
        DenseMatrix {
            rows: 1,
            cols: a.cols,
            data,
        }
    } else if dim == 2 {
        let mut data = Vec::with_capacity(a.rows);
        for r in 0..a.rows {
            let slice = &a.data[r * a.cols..(r + 1) * a.cols];
            data.push(reduce_slice(slice));
        }
        DenseMatrix {
            rows: a.rows,
            cols: 1,
            data,
        }
    } else {
        // dim > 2: each slice has length 1
        let data = a.data.iter().map(|&x| reduce_slice(&[x])).collect();
        DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
        }
    }
}

/// Generic dimension reduction with 1-based indices (used by min and max).
pub fn reduce_dim_with_idx<F>(
    a: &DenseMatrix,
    dim: usize,
    reduce_slice: F,
) -> (DenseMatrix, DenseMatrix)
where
    F: Fn(&[f64]) -> (f64, usize),
{
    if a.data.is_empty() {
        return (DenseMatrix::empty(), DenseMatrix::empty());
    }
    if dim == 1 {
        let mut vals = Vec::with_capacity(a.cols);
        let mut idxs = Vec::with_capacity(a.cols);
        let mut slice = vec![0.0; a.rows];
        for c in 0..a.cols {
            for r in 0..a.rows {
                slice[r] = a.data[r * a.cols + c];
            }
            let (v, idx0) = reduce_slice(&slice);
            vals.push(v);
            idxs.push((idx0 + 1) as f64);
        }
        (
            DenseMatrix {
                rows: 1,
                cols: a.cols,
                data: vals,
            },
            DenseMatrix {
                rows: 1,
                cols: a.cols,
                data: idxs,
            },
        )
    } else if dim == 2 {
        let mut vals = Vec::with_capacity(a.rows);
        let mut idxs = Vec::with_capacity(a.rows);
        for r in 0..a.rows {
            let slice = &a.data[r * a.cols..(r + 1) * a.cols];
            let (v, idx0) = reduce_slice(slice);
            vals.push(v);
            idxs.push((idx0 + 1) as f64);
        }
        (
            DenseMatrix {
                rows: a.rows,
                cols: 1,
                data: vals,
            },
            DenseMatrix {
                rows: a.rows,
                cols: 1,
                data: idxs,
            },
        )
    } else {
        let mut vals = Vec::with_capacity(a.data.len());
        let mut idxs = Vec::with_capacity(a.data.len());
        for &x in &a.data {
            let (v, _) = reduce_slice(&[x]);
            vals.push(v);
            idxs.push(1.0);
        }
        (
            DenseMatrix {
                rows: a.rows,
                cols: a.cols,
                data: vals,
            },
            DenseMatrix {
                rows: a.rows,
                cols: a.cols,
                data: idxs,
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Slice Reduction Helpers
// ---------------------------------------------------------------------------

pub fn sum_slice(slice: &[f64]) -> f64 {
    slice.iter().sum()
}

pub fn prod_slice(slice: &[f64]) -> f64 {
    slice.iter().fold(1.0, |acc, &x| acc * x)
}

pub fn mean_slice(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    let mut sum = 0.0;
    for &x in slice {
        if x.is_nan() {
            return f64::NAN;
        }
        sum += x;
    }
    sum / (slice.len() as f64)
}

pub fn median_slice(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if slice.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    let mut sorted = slice.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n % 2 != 0 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

pub fn mode_slice(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if slice.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    let mut sorted = slice.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut max_count = 0;
    let mut mode_val = sorted[0];
    let mut cur_val = sorted[0];
    let mut cur_count = 0;
    for &x in &sorted {
        if x == cur_val {
            cur_count += 1;
        } else {
            if cur_count > max_count {
                max_count = cur_count;
                mode_val = cur_val;
            }
            cur_val = x;
            cur_count = 1;
        }
    }
    if cur_count > max_count {
        mode_val = cur_val;
    }
    mode_val
}

pub fn var_slice(slice: &[f64], w: usize) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if slice.len() == 1 {
        return 0.0;
    }
    let mean = mean_slice(slice);
    if mean.is_nan() {
        return f64::NAN;
    }
    let mut ss = 0.0;
    for &x in slice {
        let diff = x - mean;
        ss += diff * diff;
    }
    let denom = if w == 0 {
        (slice.len() - 1) as f64
    } else {
        slice.len() as f64
    };
    ss / denom
}

pub fn std_slice(slice: &[f64], w: usize) -> f64 {
    var_slice(slice, w).sqrt()
}

pub fn range_slice(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if slice.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    let min = slice.iter().copied().fold(f64::INFINITY, f64::min);
    let max = slice.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    max - min
}

fn quantile(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let r = 0.5 + (n as f64) * p;
    if r <= 1.0 {
        sorted[0]
    } else if r >= (n as f64) {
        sorted[n - 1]
    } else {
        let k = r.floor() as usize;
        let f = r - (k as f64);
        (1.0 - f) * sorted[k - 1] + f * sorted[k]
    }
}

pub fn iqr_slice(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if slice.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    if slice.len() == 1 {
        return 0.0;
    }
    let mut sorted = slice.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q25 = quantile(&sorted, 0.25);
    let q75 = quantile(&sorted, 0.75);
    q75 - q25
}

pub fn min_slice(slice: &[f64]) -> (f64, usize) {
    if slice.is_empty() {
        return (f64::NAN, 0);
    }
    let mut best_val = f64::NAN;
    let mut best_idx = 0;
    for (i, &v) in slice.iter().enumerate() {
        if v.is_nan() {
            continue;
        }
        if best_val.is_nan() || v < best_val {
            best_val = v;
            best_idx = i;
        }
    }
    (best_val, best_idx)
}

pub fn max_slice(slice: &[f64]) -> (f64, usize) {
    if slice.is_empty() {
        return (f64::NAN, 0);
    }
    let mut best_val = f64::NAN;
    let mut best_idx = 0;
    for (i, &v) in slice.iter().enumerate() {
        if v.is_nan() {
            continue;
        }
        if best_val.is_nan() || v > best_val {
            best_val = v;
            best_idx = i;
        }
    }
    (best_val, best_idx)
}

// ---------------------------------------------------------------------------
// Matrix-Level Operations
// ---------------------------------------------------------------------------

pub fn mean(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, mean_slice)
}

pub fn median(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, median_slice)
}

pub fn mode(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, mode_slice)
}

pub fn var(a: &DenseMatrix, w: usize, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, |s| var_slice(s, w))
}

pub fn std(a: &DenseMatrix, w: usize, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, |s| std_slice(s, w))
}

pub fn range(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, range_slice)
}

pub fn iqr(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, iqr_slice)
}

pub fn prod(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, prod_slice)
}

pub fn sum(a: &DenseMatrix, dim: Option<usize>) -> DenseMatrix {
    let d = resolve_dim(a, dim);
    reduce_dim(a, d, sum_slice)
}

pub fn min_reduction(a: &DenseMatrix, dim: Option<usize>) -> (DenseMatrix, DenseMatrix) {
    let d = resolve_dim(a, dim);
    reduce_dim_with_idx(a, d, min_slice)
}

pub fn max_reduction(a: &DenseMatrix, dim: Option<usize>) -> (DenseMatrix, DenseMatrix) {
    let d = resolve_dim(a, dim);
    reduce_dim_with_idx(a, d, max_slice)
}

pub fn min_elementwise(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    map_binary_simple(a, b, span, |x, y| {
        if x.is_nan() {
            y
        } else if y.is_nan() {
            x
        } else {
            x.min(y)
        }
    })
}

pub fn max_elementwise(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    map_binary_simple(a, b, span, |x, y| {
        if x.is_nan() {
            y
        } else if y.is_nan() {
            x
        } else {
            x.max(y)
        }
    })
}
