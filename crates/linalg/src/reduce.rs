//! Reduction, ordering, set-membership and difference kernels (Stage A).
//!
//! Conventions follow MATLAB's documented defaults:
//! - Reduce along the first non-singleton dimension when no dimension is given;
//!   an explicit dimension overrides it.
//! - `sort` places NaN values last (ascending and descending).
//! - `unique` preserves NaNs as distinct elements (NaN != NaN by equality).

use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::DenseMatrix;

use crate::arrays::err;

/// Resolve the effective dimension: explicit, or the first non-singleton
/// dimension (`dim 1` when every dimension is 1 or when the array is empty).
pub fn effective_dim(a: &DenseMatrix, dim: Option<usize>) -> Result<usize, Diagnostic> {
    match dim {
        Some(d @ 1) | Some(d @ 2) => Ok(d),
        Some(_) => Err(err(
            "Dimensions must be 1 or 2 in this runtime (2-D arrays).",
            None,
        )),
        None => {
            if a.rows != 1 {
                Ok(1)
            } else if a.cols != 1 {
                Ok(2)
            } else {
                Ok(1)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sorting
// ---------------------------------------------------------------------------

/// NaN-last comparator: NaN compares greater than every finite value (and
/// equal to other NaNs), matching MATLAB's sort convention in both directions.
fn nan_last_cmp(a: f64, b: f64) -> std::cmp::Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
    }
}

fn sort_slice_desc(values: &mut Vec<f64>) {
    // Stable sort with NaNs pinned to the end even in descending order.
    values.sort_by(|a, b| {
        if a.is_nan() || b.is_nan() {
            nan_last_cmp(*a, *b)
        } else {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
}

fn sort_slice_asc(values: &mut Vec<f64>) {
    values.sort_by(|a, b| nan_last_cmp(*a, *b));
}

/// `sort(A)` — default dimension is the first non-singleton one; the optional
/// dimension argument and the `'descend'` flag mirror MATLAB.
pub fn sort(a: &DenseMatrix, dim: Option<usize>, descend: bool) -> MatResult {
    let dim = effective_dim(a, dim)?;
    if a.data.is_empty() {
        return Ok(a.clone());
    }
    match dim {
        1 => {
            // Sort each column independently, keep positions.
            let mut cols: Vec<Vec<f64>> = (0..a.cols)
                .map(|c| (0..a.rows).map(|r| a.data[r * a.cols + c]).collect())
                .collect();
            for col in cols.iter_mut() {
                if descend {
                    sort_slice_desc(col);
                } else {
                    sort_slice_asc(col);
                }
            }
            let mut data = vec![0.0; a.data.len()];
            for (c, col) in cols.iter().enumerate() {
                for (r, &v) in col.iter().enumerate() {
                    data[r * a.cols + c] = v;
                }
            }
            Ok(DenseMatrix { rows: a.rows, cols: a.cols, data })
        }
        _ => {
            let mut rows_sorted: Vec<Vec<f64>> = (0..a.rows)
                .map(|r| a.data[r * a.cols..(r + 1) * a.cols].to_vec())
                .collect();
            for row in rows_sorted.iter_mut() {
                if descend {
                    sort_slice_desc(row);
                } else {
                    sort_slice_asc(row);
                }
            }
            Ok(DenseMatrix::from_rows(rows_sorted, None)?)
        }
    }
}

/// `sortrows(A, cols)` — stable row ordering by the given column priorities.
/// Positive column indexes sort ascending, negative descending; with an empty
/// `cols` all columns are keys in ascending order.
pub fn sortrows(a: &DenseMatrix, cols: &[isize]) -> MatResult {
    let keys: Vec<(usize, bool)> = if cols.is_empty() {
        (0..a.cols).map(|c| (c, false)).collect()
    } else {
        for &c in cols {
            let idx = c.unsigned_abs() as usize;
            if idx == 0 || idx > a.cols {
                return Err(err(
                    format!(
                        "sortrows: column index {} is out of range for {} columns.",
                        c, a.cols
                    ),
                    None,
                ));
            }
        }
        cols.iter()
            .map(|&c| (c.unsigned_abs() as usize - 1, c < 0))
            .collect()
    };
    let mut sortable: Vec<(Vec<f64>, usize)> = (0..a.rows)
        .map(|r| (a.data[r * a.cols..(r + 1) * a.cols].to_vec(), r))
        .collect();
    sortable.sort_by(|(ra, ia), (rb, ib)| {
        for &(key, descend) in &keys {
            let ord = if descend {
                nan_last_cmp(rb[key], ra[key])
            } else {
                nan_last_cmp(ra[key], rb[key])
            };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        ia.cmp(ib) // stable relative to original row order
    });
    let data: Vec<f64> = sortable.into_iter().flat_map(|(row, _)| row).collect();
    Ok(DenseMatrix { rows: a.rows, cols: a.cols, data })
}

// ---------------------------------------------------------------------------
// Sets and membership
// ---------------------------------------------------------------------------

/// `unique(A)` — sorted unique values as a row vector; NaNs are preserved
/// (each NaN is its own element, matching MATLAB).
pub fn unique(a: &DenseMatrix) -> MatResult {
    let mut values = a.to_col_major();
    if values.is_empty() {
        return Ok(DenseMatrix { rows: 1, cols: 0, data: Vec::new() });
    }
    // Stable ascending sort, NaNs last.
    values.sort_by(|a, b| {
        if a.is_nan() && b.is_nan() {
            // Keep NaN insertion order stable; equality of NaN elements must
            // not collapse them later, so order between them is irrelevant.
            std::cmp::Ordering::Equal
        } else {
            nan_last_cmp(*a, *b)
        }
    });
    let mut out: Vec<f64> = Vec::new();
    for &v in &values {
        let duplicate = out.iter().any(|&u| u == v); // NaN == NaN is false -> kept
        if !duplicate {
            out.push(v);
        }
    }
    Ok(DenseMatrix { rows: 1, cols: out.len(), data: out })
}

/// `find(cond)` — linear indices (column-major, 1-based) of nonzero elements,
/// returned as a column vector.
pub fn find(a: &DenseMatrix) -> MatResult {
    let mut out = Vec::new();
    for c in 0..a.cols {
        for r in 0..a.rows {
            if a.data[r * a.cols + c] != 0.0 {
                out.push((c * a.rows + r + 1) as f64);
            }
        }
    }
    Ok(DenseMatrix { rows: out.len(), cols: 1, data: out })
}

/// `any(A, dim)` / `all(A, dim)` share this reduction kernel.
pub fn any_all(a: &DenseMatrix, all_mode: bool, dim: Option<usize>) -> MatResult {
    let dim = effective_dim(a, dim)?;
    if a.data.is_empty() {
        // MATLAB: any([]) is 0 (false); all([]) is 1 (true) — reduced away.
        let v = if all_mode { 1.0 } else { 0.0 };
        return Ok(DenseMatrix { rows: 1, cols: 1, data: vec![v] });
    }
    let (line_count, line_len) = match dim {
        1 => (a.cols, a.rows),
        _ => (a.rows, a.cols),
    };
    let mut out = Vec::with_capacity(line_count);
    for l in 0..line_count {
        let mut any_z = false;
        let mut all_t = true;
        for idx in 0..line_len {
            let v = match dim {
                1 => a.data[idx * a.cols + l],
                _ => a.data[l * a.cols + idx],
            };
            if v == 0.0 {
                any_z = true;
            } else {
                all_t = false;
            }
        }
        out.push(if all_mode {
            if all_t { 1.0 } else { 0.0 }
        } else if any_z {
            0.0
        } else {
            1.0
        });
    }
    match dim {
        1 => Ok(DenseMatrix { rows: 1, cols: out.len(), data: out }),
        _ => Ok(DenseMatrix { rows: out.len(), cols: 1, data: out }),
    }
}

/// `ismember(a, B)` — element-wise membership of `a`'s values in the flat `B`.
pub fn ismember(a: &DenseMatrix, set: &DenseMatrix) -> MatResult {
    let pool: Vec<f64> = set.data.clone();
    let data = a
        .data
        .iter()
        .map(|&v| if pool.iter().any(|&u| u == v) { 1.0 } else { 0.0 })
        .collect();
    Ok(DenseMatrix { rows: a.rows, cols: a.cols, data })
}

/// `setdiff(a, b)` — sorted unique values of `a` missing from `b`.
pub fn setdiff(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let uniq_a = unique(a)?;
    let mut out = Vec::new();
    for &v in uniq_a.data.iter() {
        if !b.data.iter().any(|&u| u == v) {
            out.push(v);
        }
    }
    out.sort_by(|a, b| nan_last_cmp(*a, *b));
    Ok(DenseMatrix { rows: 1, cols: out.len(), data: out })
}

/// `union(a, b)` — sorted unique values present in either.
pub fn union(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let mut combined = a.to_col_major();
    combined.extend(b.to_col_major());
    let uniq = unique(&DenseMatrix {
        rows: combined.len(),
        cols: 1,
        data: combined,
    })?;
    let mut out = uniq.data;
    out.sort_by(|a, b| nan_last_cmp(*a, *b));
    Ok(DenseMatrix { rows: 1, cols: out.len(), data: out })
}

/// `intersect(a, b)` — sorted unique values present in both.
pub fn intersect(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let uniq_a = unique(a)?;
    let mut out = Vec::new();
    for &v in uniq_a.data.iter() {
        if b.data.iter().any(|&u| u == v) {
            out.push(v);
        }
    }
    out.sort_by(|a, b| nan_last_cmp(*a, *b));
    Ok(DenseMatrix { rows: 1, cols: out.len(), data: out })
}

// ---------------------------------------------------------------------------
// Cumulative and difference
// ---------------------------------------------------------------------------

/// `cumsum(A,dim)` / `cumprod(A,dim)` share this cumulative kernel.
pub fn cumulative(a: &DenseMatrix, sum_mode: bool, dim: Option<usize>) -> MatResult {
    let original = a;
    if a.data.is_empty() {
        return Ok(a.clone());
    }
    let dim = effective_dim(a, dim)?;
    let (line_count, line_len) = match dim {
        1 => (a.cols, a.rows),
        _ => (a.rows, a.cols),
    };
    let mut out = vec![0.0; a.data.len()];
    for l in 0..line_count {
        let mut acc = 1.0; // multiplicative identity
        let mut total = 0.0; // additive identity
        for idx in 0..line_len {
            let v = match dim {
                1 => original.data[idx * a.cols + l],
                _ => original.data[l * a.cols + idx],
            };
            if sum_mode {
                total += v;
                let value = total;
                match dim {
                    1 => out[idx * a.cols + l] = value,
                    _ => out[l * a.cols + idx] = value,
                }
            } else {
                acc *= v;
                let value = acc;
                match dim {
                    1 => out[idx * a.cols + l] = value,
                    _ => out[l * a.cols + idx] = value,
                }
            }
        }
    }
    Ok(DenseMatrix { rows: a.rows, cols: a.cols, data: out })
}

/// `diff(A,n,dim)` — n-th order consecutive differences. Default dimension is
/// the first non-singleton one.
pub fn diff(a: &DenseMatrix, n: usize, dim: Option<usize>) -> MatResult {
    if a.data.is_empty() {
        return Ok(a.clone());
    }
    let dim = effective_dim(a, dim)?;
    let mut m = a.clone();
    for _ in 0..n {
        let (len, other) = match dim {
            1 => (m.rows, m.cols),
            _ => (m.cols, m.rows),
        };
        if len == 0 {
            break;
        }
        let new_len = if len <= 1 { 0 } else { len - 1 };
        if new_len == 0 {
            // Degrades to an empty line along `dim`.
            return Ok(match dim {
                1 => DenseMatrix { rows: 0, cols: other, data: Vec::new() },
                _ => DenseMatrix { rows: other, cols: 0, data: Vec::new() },
            });
        }
        let mut out = vec![0.0; new_len * other];
        for l in 0..other {
            for idx in 0..new_len {
                let cur = if dim == 1 {
                    m.data[idx * m.cols + l]
                } else {
                    m.data[l * m.cols + idx]
                };
                let next = if dim == 1 {
                    m.data[(idx + 1) * m.cols + l]
                } else {
                    m.data[l * m.cols + idx + 1]
                };
                let v = next - cur;
                if dim == 1 {
                    out[idx * m.cols + l] = v;
                } else {
                    out[l * m.cols + idx] = v;
                }
            }
        }
        m = match dim {
            1 => DenseMatrix { rows: new_len, cols: m.cols, data: out },
            _ => DenseMatrix { rows: m.rows, cols: new_len, data: out },
        };
    }
    Ok(m)
}

/// `histcounts(x, edges)` / `histcounts(x, nbins)`.
/// With explicit edges: count values in `[e_i, e_{i+1})`; the last bin also
/// includes its right edge. Values outside the edges are not counted.
/// With a bin count: `nbins` equal bins from `min(x)` to `max(x)`.
pub fn histcounts(
    x: &DenseMatrix,
    edges_or_n: HistcountsArg,
    span: Option<Span>,
) -> MatResult {
    let values: Vec<f64> = x.to_col_major();
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let edges: Vec<f64> = match edges_or_n {
        HistcountsArg::Edges(e) => {
            if e.len() < 2 {
                return Err(err("histcounts: at least two edges are required.", span));
            }
            e
        }
        HistcountsArg::Bins(n) => {
            if n == 0 {
                return Ok(DenseMatrix { rows: 1, cols: 0, data: Vec::new() });
            }
            if finite.is_empty() {
                // No finite range to spread over: zero counts everywhere.
                return Ok(DenseMatrix { rows: 1, cols: n, data: vec![0.0; n] });
            }
            let lo = finite.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = finite.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let step = (hi - lo) / n as f64;
            (0..=n).map(|i| lo + step * i as f64).collect()
        }
    };
    let nbins = edges.len() - 1;
    let mut counts = vec![0.0f64; nbins];
    for &v in &values {
        // Skip NaN/Inf like MATLAB (they never fall inside finite bins).
        if v.is_nan() || v.is_infinite() {
            continue;
        }
        // Values below the first edge are excluded.
        if v < edges[0] || v > edges[edges.len() - 1] {
            continue;
        }
        // Binary search for the bin.
        let mut lo = 0usize;
        let mut hi = nbins;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if v < edges[mid + 1] {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        let bin = lo.min(nbins - 1);
        // The final bin includes its right edge; other bins exclude it.
        if bin == nbins - 1 || v < edges[bin + 1] {
            counts[bin] += 1.0;
        }
    }
    Ok(DenseMatrix { rows: 1, cols: nbins, data: counts })
}

#[derive(Debug, Clone)]
pub enum HistcountsArg {
    Edges(Vec<f64>),
    Bins(usize),
}
