//! Array construction and shape-manipulation kernels (Stage A).
//!
//! Pure functions on `DenseMatrix`: no AST, environment, or runtime knowledge.
//! All matrices are treated as 2-D, column-major *semantically* (MATLAB rule:
//! elements aggregate/flatten in column-major order) while the `DenseMatrix`
//! storage itself is row-major — kernels that care about element order go
//! through `to_col_major`/`from_col_major` rather than touching `data`
//! directly, except for shape-preserving element maps.

use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::DenseMatrix;

pub type MatResult = Result<DenseMatrix, Diagnostic>;

pub fn err(msg: impl Into<String>, span: Option<Span>) -> Diagnostic {
    Diagnostic::error(msg, span)
}

/// Assemble a matrix from column-major data (length must match rows*cols).
pub fn from_col_major(data: Vec<f64>, rows: usize, cols: usize, span: Option<Span>) -> MatResult {
    if data.len() != rows * cols {
        return Err(err(
            format!(
                "Internal error: column-major data length {} does not match {}x{}.",
                data.len(),
                rows,
                cols
            ),
            span,
        ));
    }
    // data column-major: out row-major mapping: row r, col c -> data[c*rows + r]
    let mut out = vec![0.0; rows * cols];
    for c in 0..cols {
        for r in 0..rows {
            out[r * cols + c] = data[c * rows + r];
        }
    }
    Ok(DenseMatrix {
        rows,
        cols,
        data: out,
    })
}

// ---------------------------------------------------------------------------
// Spaced sequences
// ---------------------------------------------------------------------------

/// `linspace(a,b,n)` — n evenly spaced points from a to b, endpoints included.
/// `linspace(a,b,1)` returns `a` (the start point), matching MATLAB.
pub fn linspace(a: f64, b: f64, n: usize, _span: Option<Span>) -> MatResult {
    match n {
        0 => Ok(DenseMatrix {
            rows: 1,
            cols: 0,
            data: Vec::new(),
        }),
        1 => Ok(DenseMatrix::scalar(a)),
        count => {
            let n_f = (count - 1) as f64;
            let data = (0..count)
                .map(|i| {
                    let t = i as f64 / n_f;
                    a + t * (b - a)
                })
                .collect();
            Ok(DenseMatrix {
                rows: 1,
                cols: count,
                data,
            })
        }
    }
}

/// `logspace(a,b,n)` — n points from 10^a to 10^b, log-spaced, endpoints included.
/// Like MATLAB, values that overflow to Inf are returned as Inf (not an error).
pub fn logspace(a: f64, b: f64, n: usize, span: Option<Span>) -> MatResult {
    let lin = linspace(a, b, n, span)?;
    let data = lin.data.iter().map(|&x| 10f64.powf(x)).collect();
    Ok(DenseMatrix {
        rows: lin.rows,
        cols: lin.cols,
        data,
    })
}

/// `magic(n)` — a magic square whose every row, column, and both diagonals sum
/// to `n(n²+1)/2`.
///
/// Layouts implemented and property-tested:
/// - odd `n`: Siamese method (de la Loubère).
/// - `n % 4 == 0`: the complement-pair doubly-even pattern.
/// - `n == 2`: MATLAB's `magic(2)` = `[1 3; 4 2]`.
/// - `n == 0`: empty.
///
/// Singly-even sizes (n = 6, 10, 14, …) are **not implemented yet** and return
/// a clear diagnostic; recorded in the function-coverage ledger.
pub fn magic(n: usize, span: Option<Span>) -> MatResult {
    if n == 0 {
        return Ok(DenseMatrix {
            rows: 0,
            cols: 0,
            data: Vec::new(),
        });
    }
    if n == 2 {
        return DenseMatrix::from_rows(vec![vec![1.0, 3.0], vec![4.0, 2.0]], span);
    }
    if n % 4 == 2 {
        return Err(err(
            format!(
                "magic is not yet implemented for singly-even n = {} (try odd or n divisible by 4).",
                n
            ),
            span,
        ));
    }
    let rows = if n % 2 == 1 {
        siamese_magic(n)
    } else {
        doubly_even_magic(n)
    };
    DenseMatrix::from_rows(rows, span)
}

fn siamese_magic(n: usize) -> Vec<Vec<f64>> {
    let mut grid = vec![vec![0usize; n]; n];
    let (mut r, mut c) = (0usize, n / 2);
    for k in 1..=n * n {
        grid[r][c] = k;
        let nr = (r + n - 1) % n;
        let nc = (c + 1) % n;
        if grid[nr][nc] != 0 {
            r = (r + 1) % n;
        } else {
            r = nr;
            c = nc;
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().map(|v| v as f64).collect())
        .collect()
}

fn doubly_even_magic(n: usize) -> Vec<Vec<f64>> {
    let mut grid = Vec::with_capacity(n);
    for i in 0..n {
        let mut row = Vec::with_capacity(n);
        for j in 0..n {
            let cell = (i * n + j) as f64 + 1.0;
            let keep = (i % 4 == j % 4) || ((i % 4) + (j % 4) == 3);
            let value = if keep {
                cell
            } else {
                (n * n) as f64 + 1.0 - cell
            };
            row.push(value);
        }
        grid.push(row);
    }
    grid
}

/// `diag(v)` with a vector builds a matrix; `diag(A)` extracts the main
/// diagonal into a column vector. The optional `k` shifts off the main
/// diagonal (`diag(v,1)` above the main diagonal, `diag(v,-1)` below).
pub fn diag(a: &DenseMatrix, k: isize) -> MatResult {
    if a.rows == 1 || a.cols == 1 {
        // Build mode.
        let v: Vec<f64> = a.to_col_major();
        let count = v.len() as isize;
        // result is (count + |k|) square
        let m = (count + k.unsigned_abs() as isize).unsigned_abs() as usize;
        let mut data = vec![0.0; m * m];
        for (t, &value) in v.iter().enumerate() {
            let t_i = t as isize;
            // Diagonal index k satisfies row - col = k, so a negative k
            // shifts the values *down* the sub-diagonal.
            let r = (t_i + (-k).max(0)) as usize;
            let c = (t_i + k.max(0)) as usize;
            data[r * m + c] = value;
        }
        return Ok(DenseMatrix {
            rows: m,
            cols: m,
            data,
        });
    }
    // Extract mode: a matrix whose rows or cols may differ; a rectangular
    // matrix extract is allowed by MATLAB when k fits.
    let out_of_range =
        k < 0 && (k.unsigned_abs() as usize) >= a.rows || k > 0 && (k as usize) >= a.cols;
    if out_of_range {
        return Ok(DenseMatrix {
            rows: 0,
            cols: 1,
            data: Vec::new(),
        });
    }
    let len = if k >= 0 {
        (a.cols as isize - k).min(a.rows as isize)
    } else {
        let kk = -k;
        (a.rows as isize - kk).min(a.cols as isize)
    };
    if len <= 0 {
        return Ok(DenseMatrix {
            rows: 0,
            cols: 1,
            data: Vec::new(),
        });
    }
    let mut data = Vec::with_capacity(len as usize);
    for t in 0..len as usize {
        let (r, c) = if k >= 0 {
            (t, t + k as usize)
        } else {
            (t + (-k) as usize, t)
        };
        data.push(a.data[r * a.cols + c]);
    }
    Ok(DenseMatrix {
        rows: data.len(),
        cols: 1,
        data,
    })
}

/// `repmat(A,m,n)` — tile `A` m times down and n times across.
pub fn repmat(a: &DenseMatrix, m: usize, n: usize) -> MatResult {
    let out_rows = a.rows * m;
    let out_cols = a.cols * n;
    let mut data = Vec::with_capacity(out_rows * out_cols);
    for r in 0..out_rows {
        for c in 0..out_cols {
            data.push(a.data[(r % a.rows) * a.cols + (c % a.cols)]);
        }
    }
    Ok(DenseMatrix {
        rows: out_rows,
        cols: out_cols,
        data,
    })
}

// ---------------------------------------------------------------------------
// Grids
// ---------------------------------------------------------------------------

/// Internal grid: builds one output of the requested orientation.
fn grid_output(values: &[f64], varies_down: bool, rows: usize, cols: usize) -> DenseMatrix {
    let mut data = vec![0.0; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            data[r * cols + c] = if varies_down {
                values[r.min(values.len() - 1)]
            } else {
                values[c.min(values.len() - 1)]
            };
        }
    }
    DenseMatrix { rows, cols, data }
}

/// `meshgrid(x)` or `meshgrid(x,y)` → [X Y]; `meshgrid(x,y,z)` → [X Y Z].
/// X varies along columns (dim 2), Y along rows (dim 1), Z as planes.
pub fn meshgrid(inputs: &[DenseMatrix]) -> Result<Vec<DenseMatrix>, Diagnostic> {
    if inputs.is_empty() || inputs.len() > 3 {
        return Err(err("meshgrid expects 1 to 3 vector inputs.", None));
    }
    let x = inputs[0].to_col_major();
    let vals: Vec<DenseMatrix> = match inputs.len() {
        1 => {
            // meshgrid(x): X and Y both n-by-n
            vec![
                grid_output(&x, false, x.len(), x.len()),
                grid_output(&x, true, x.len(), x.len()),
            ]
        }
        2 => {
            let y = inputs[1].to_col_major();
            vec![
                grid_output(&x, false, y.len(), x.len()),
                grid_output(&y, true, y.len(), x.len()),
            ]
        }
        3 => {
            let y = inputs[1].to_col_major();
            let z = inputs[2].to_col_major();
            vec![
                grid_output(&x, false, y.len(), x.len()),
                grid_output(&y, true, y.len(), x.len()),
                grid_output(&z, true, y.len(), x.len()),
            ]
        }
        _ => unreachable!(),
    };
    Ok(vals)
}

/// `ndgrid(x,y)` → [X Y]; X varies along rows (dim 1), Y along columns (dim 2).
pub fn ndgrid(inputs: &[DenseMatrix]) -> Result<Vec<DenseMatrix>, Diagnostic> {
    if inputs.is_empty() {
        return Err(err("ndgrid expects 1 or more vector inputs.", None));
    }
    if inputs.len() > 2 {
        return Err(err(
            "ndgrid is limited to 2 outputs in this runtime (2-D arrays only).",
            None,
        ));
    }
    let x = inputs[0].to_col_major();
    let y = if inputs.len() == 2 {
        inputs[1].to_col_major()
    } else {
        x.clone()
    };
    Ok(vec![
        grid_output(&x, true, x.len(), y.len()),
        grid_output(&y, false, x.len(), y.len()),
    ])
}

// ---------------------------------------------------------------------------
// Concatenation
// ---------------------------------------------------------------------------

/// `cat(dim, A, B, ...)` — concatenate along dim 1 (rows) or 2 (columns).
/// 2-D only; other dims error while N-D is unavailable.
pub fn cat(dim: usize, inputs: &[DenseMatrix], span: Option<Span>) -> MatResult {
    if inputs.len() < 2 {
        // MATLAB: cat(1, A) returns A; with zero args it errors.
        if inputs.len() == 1 {
            return Ok(inputs[0].clone());
        }
        return Err(err("cat requires at least one array.", span));
    }
    if dim == 1 {
        vertcat(inputs, span)
    } else if dim == 2 {
        horzcat(inputs, span)
    } else {
        Err(err(
            "cat is limited to dimensions 1 and 2 in this runtime (2-D arrays).",
            span,
        ))
    }
}

/// `vertcat(A, B, ...)` — stack inputs on top of one another.
pub fn vertcat(inputs: &[DenseMatrix], span: Option<Span>) -> MatResult {
    if inputs.is_empty() {
        return Err(err("vertcat requires at least one array.", span));
    }
    let cols = inputs[0].cols;
    for (idx, m) in inputs.iter().enumerate().skip(1) {
        if m.cols != cols {
            return Err(err(
                format!(
                    "Dimensions of arrays being concatenated are not consistent: input {} has {} columns, first input has {}.",
                    idx + 1,
                    m.cols,
                    cols
                ),
                span,
            ));
        }
    }
    let mut data = Vec::new();
    for m in inputs {
        data.extend_from_slice(&m.data);
    }
    Ok(DenseMatrix {
        rows: inputs.iter().map(|m| m.rows).sum(),
        cols,
        data,
    })
}

/// `horzcat(A, B, ...)` — place inputs side by side.
pub fn horzcat(inputs: &[DenseMatrix], span: Option<Span>) -> MatResult {
    if inputs.is_empty() {
        return Err(err("horzcat requires at least one array.", span));
    }
    let rows = inputs[0].rows;
    for (idx, m) in inputs.iter().enumerate().skip(1) {
        if m.rows != rows {
            return Err(err(
                format!(
                    "Dimensions of arrays being concatenated are not consistent: input {} has {} rows, first input has {}.",
                    idx + 1,
                    m.rows,
                    rows
                ),
                span,
            ));
        }
    }
    let mut data = Vec::new();
    for r in 0..rows {
        for m in inputs {
            data.extend_from_slice(&m.data[r * m.cols..(r + 1) * m.cols]);
        }
    }
    Ok(DenseMatrix {
        rows,
        cols: inputs.iter().map(|m| m.cols).sum(),
        data,
    })
}

/// `blkdiag(A, B, ...)` — block-diagonal assembly with zero fill.
pub fn blkdiag(inputs: &[DenseMatrix], span: Option<Span>) -> MatResult {
    if inputs.is_empty() {
        return Err(err("blkdiag requires at least one array.", span));
    }
    let rows: usize = inputs.iter().map(|m| m.rows).sum();
    let cols: usize = inputs.iter().map(|m| m.cols).sum();
    let mut data = vec![0.0; rows * cols];
    let mut row_off = 0;
    let mut col_off = 0;
    for m in inputs {
        for r in 0..m.rows {
            for c in 0..m.cols {
                data[(row_off + r) * cols + (col_off + c)] = m.data[r * m.cols + c];
            }
        }
        row_off += m.rows;
        col_off += m.cols;
    }
    Ok(DenseMatrix { rows, cols, data })
}

// ---------------------------------------------------------------------------
// Shape operations
// ---------------------------------------------------------------------------

/// `reshape(A, m, n)` — reorder into a new shape keeping column-major order.
/// One of m or n may be `None` (MATLAB `reshape(A,[],n)`), auto-inferred.
pub fn reshape(a: &DenseMatrix, m: Option<usize>, n: Option<usize>) -> MatResult {
    let total = a.rows * a.cols;
    let (rows, cols) = match (m, n) {
        (Some(r), Some(c)) => (r, c),
        (None, Some(c)) if total % c == 0 || total == 0 => (total / c, c),
        (Some(r), None) if total % r == 0 || total == 0 => (r, total / r),
        _ => {
            return Err(err(
                "reshape: one dimension may be [] only when the other divides numel.",
                None,
            ))
        }
    };
    if rows * cols != total {
        return Err(err(
            format!(
                "To reshape, the number of elements must not change: {}x{} has {} elements, requested {}x{} = {}.",
                a.rows,
                a.cols,
                total,
                rows,
                cols,
                rows * cols
            ),
            None,
        ));
    }
    let cm = a.to_col_major();
    from_col_major(cm, rows, cols, None)
}

/// `squeeze(A)` — 2-D model: no change (N-D is not available yet, and MATLAB
/// also leaves 2-D arrays untouched), except a 0-width/0-height input stays.
pub fn squeeze(a: &DenseMatrix) -> DenseMatrix {
    a.clone()
}

/// `permute(A, order)` — rearrange dimensions; the 2-D model supports
/// `[1 2]` (identity) and `[2 1]` (transpose) only.
pub fn permute(a: &DenseMatrix, order: &[usize], span: Option<Span>) -> MatResult {
    if order.len() != 2 {
        return Err(err(
            "permute supports exactly two dimensions in this runtime (2-D arrays).",
            span,
        ));
    }
    match (order[0], order[1]) {
        (1, 2) => Ok(a.clone()),
        (2, 1) => Ok(transpose(a)),
        _ => Err(err(
            format!(
                "PERMUTE: order must contain the dimensions [1 2] or [2 1] for 2-D arrays; got [{:?}].",
                order
            ),
            span,
        )),
    }
}

/// `transpose(A)` — non-conjugate transpose.
pub fn transpose(a: &DenseMatrix) -> DenseMatrix {
    let mut data = vec![0.0; a.rows * a.cols];
    for r in 0..a.rows {
        for c in 0..a.cols {
            data[c * a.rows + r] = a.data[r * a.cols + c];
        }
    }
    DenseMatrix {
        rows: a.cols,
        cols: a.rows,
        data,
    }
}

/// `flip(A)` / `flipud` / `fliplr` with an explicit dimension.
pub fn flip(a: &DenseMatrix, dim: usize, span: Option<Span>) -> MatResult {
    match dim {
        1 => flipud(a),
        2 => fliplr(a),
        _ => Err(err(
            "flip supports dimensions 1 and 2 in this runtime (2-D arrays).",
            span,
        )),
    }
}

/// `flipud(A)` — reverse the rows.
pub fn flipud(a: &DenseMatrix) -> MatResult {
    let mut data = Vec::with_capacity(a.data.len());
    for r in (0..a.rows).rev() {
        data.extend_from_slice(&a.data[r * a.cols..(r + 1) * a.cols]);
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// `fliplr(A)` — reverse the columns.
pub fn fliplr(a: &DenseMatrix) -> MatResult {
    let mut data = Vec::with_capacity(a.data.len());
    for r in 0..a.rows {
        for c in (0..a.cols).rev() {
            data.push(a.data[r * a.cols + c]);
        }
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// `rot90(A)` — counter-clockwise rotation by k*90 degrees.
pub fn rot90(a: &DenseMatrix, k: isize) -> MatResult {
    let mut m = a.clone();
    let turns = ((k % 4) + 4) % 4;
    for _ in 0..turns {
        let t = transpose(&m);
        m = flipud(&t)?; // rotate 90 degrees counter-clockwise
    }
    Ok(m)
}

/// `circshift(A, k)` — circular shift. A scalar `k` shifts along the first
/// non-singleton dimension. `k=[k1 k2]` shifts each dimension separately.
pub fn circshift(a: &DenseMatrix, k: &[f64], span: Option<Span>) -> MatResult {
    if k.is_empty() || k.len() > 2 {
        return Err(err(
            "circshift expects a scalar or a 1/2-element shift.",
            span,
        ));
    }
    if k.iter().any(|v| v.fract() != 0.0) {
        return Err(err("circshift: shift amounts must be integers.", span));
    }
    // Decide which dimensions get which shift.
    let (shifts, dims): (Vec<isize>, Vec<usize>) = if k.len() == 1 {
        let dim = if a.rows != 1 {
            1
        } else if a.cols != 1 {
            2
        } else {
            1
        };
        (vec![k[0] as isize], vec![dim])
    } else {
        (vec![k[0] as isize, k[1] as isize], vec![1, 2])
    };
    let mut m = a.clone();
    for (&shift, &dim) in shifts.iter().zip(dims.iter()) {
        m = shift_dim_circular(&m, dim, shift)?;
    }
    Ok(m)
}

/// Circularly shift `a` along one dimension by `shift` positions.
fn shift_dim_circular(a: &DenseMatrix, dim: usize, shift: isize) -> MatResult {
    let size = match dim {
        1 => a.rows,
        2 => a.cols,
        _ => return Err(err("circshift: dimension must be 1 or 2.", None)),
    };
    if size == 0 || shift % size as isize == 0 {
        return Ok(a.clone());
    }
    let s = shift.rem_euclid(size as isize) as usize;
    let mut out = vec![0.0; a.data.len()];
    match dim {
        1 => {
            for r in 0..a.rows {
                for c in 0..a.cols {
                    out[((r + s) % a.rows) * a.cols + c] = a.data[r * a.cols + c];
                }
            }
        }
        _ => {
            for r in 0..a.rows {
                for c in 0..a.cols {
                    out[r * a.cols + (c + s) % a.cols] = a.data[r * a.cols + c];
                }
            }
        }
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data: out,
    })
}

// Sorting
// ---------------------------------------------------------------------------

pub fn sort(a: &DenseMatrix, dim: usize, descending: bool) -> MatResult {
    let mut out = a.clone();
    let cmp = if descending {
        |x: &f64, y: &f64| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal)
    } else {
        |x: &f64, y: &f64| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
    };
    if dim == 1 {
        for c in 0..a.cols {
            let mut col: Vec<f64> = (0..a.rows).map(|r| a.data[r * a.cols + c]).collect();
            col.sort_by(cmp);
            for r in 0..a.rows {
                out.data[r * a.cols + c] = col[r];
            }
        }
    } else {
        for r in 0..a.rows {
            let mut row: Vec<f64> = (0..a.cols).map(|c| a.data[r * a.cols + c]).collect();
            row.sort_by(cmp);
            for c in 0..a.cols {
                out.data[r * a.cols + c] = row[c];
            }
        }
    }
    Ok(out)
}

pub fn sortrows(a: &DenseMatrix, cols: &[usize]) -> MatResult {
    let sort_cols: Vec<usize> = if cols.is_empty() || (cols.len() == 1 && cols[0] == 0) {
        vec![0] // default: first column
    } else {
        cols.iter()
            .map(|&c| if c == 0 { 0 } else { c - 1 })
            .collect()
    };

    let mut rows: Vec<Vec<f64>> = (0..a.rows)
        .map(|r| (0..a.cols).map(|c| a.data[r * a.cols + c]).collect())
        .collect();

    rows.sort_by(|a, b| {
        for &sc in &sort_cols {
            let va = a.get(sc).copied().unwrap_or(0.0);
            let vb = b.get(sc).copied().unwrap_or(0.0);
            let cmp = va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }
        }
        std::cmp::Ordering::Equal
    });

    let mut data = Vec::with_capacity(a.rows * a.cols);
    for row in &rows {
        data.extend_from_slice(row);
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

// ---------------------------------------------------------------------------
// Unique and set operations
// ---------------------------------------------------------------------------

pub fn unique(a: &DenseMatrix) -> MatResult {
    let mut vals: Vec<f64> = a.to_col_major();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    vals.dedup_by(|x, y| *x == *y);
    let n = vals.len();
    Ok(DenseMatrix {
        rows: n,
        cols: 1,
        data: vals,
    })
}

pub fn ismember(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let b_vals: Vec<f64> = b.to_col_major();
    let data: Vec<f64> = a
        .to_col_major()
        .iter()
        .map(|&x| {
            if b_vals.iter().any(|&y| x == y) {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

pub fn setdiff(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let b_vals: Vec<f64> = b.to_col_major();
    let mut vals: Vec<f64> = a
        .to_col_major()
        .into_iter()
        .filter(|x| !b_vals.iter().any(|y| *x == *y))
        .collect();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    vals.dedup_by(|x, y| *x == *y);
    let n = vals.len();
    Ok(DenseMatrix {
        rows: n,
        cols: 1,
        data: vals,
    })
}

pub fn union(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let mut vals: Vec<f64> = a.to_col_major();
    vals.extend(b.to_col_major());
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    vals.dedup_by(|x, y| *x == *y);
    let n = vals.len();
    Ok(DenseMatrix {
        rows: n,
        cols: 1,
        data: vals,
    })
}

pub fn intersect(a: &DenseMatrix, b: &DenseMatrix) -> MatResult {
    let a_vals = a.to_col_major();
    let b_vals = b.to_col_major();
    let mut vals: Vec<f64> = a_vals
        .iter()
        .filter(|x| b_vals.iter().any(|y| **x == *y))
        .copied()
        .collect();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    vals.dedup_by(|x, y| *x == *y);
    let n = vals.len();
    Ok(DenseMatrix {
        rows: n,
        cols: 1,
        data: vals,
    })
}

// ---------------------------------------------------------------------------
// Searching and reductions
// ---------------------------------------------------------------------------

pub fn find(a: &DenseMatrix) -> MatResult {
    let mut indices = Vec::new();
    for (idx, &val) in a.to_col_major().iter().enumerate() {
        if val != 0.0 {
            indices.push((idx + 1) as f64);
        }
    }
    let n = indices.len();
    Ok(DenseMatrix {
        rows: n,
        cols: 1,
        data: indices,
    })
}

pub fn find_multi(a: &DenseMatrix) -> (DenseMatrix, DenseMatrix, DenseMatrix) {
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut vals = Vec::new();
    for r in 0..a.rows {
        for c in 0..a.cols {
            let val = a.data[r * a.cols + c];
            if val != 0.0 {
                rows.push((r + 1) as f64);
                cols.push((c + 1) as f64);
                vals.push(val);
            }
        }
    }
    let n = rows.len();
    (
        DenseMatrix {
            rows: n,
            cols: 1,
            data: rows,
        },
        DenseMatrix {
            rows: n,
            cols: 1,
            data: cols,
        },
        DenseMatrix {
            rows: n,
            cols: 1,
            data: vals,
        },
    )
}

pub fn any(a: &DenseMatrix) -> MatResult {
    let mut data = Vec::with_capacity(a.cols);
    for c in 0..a.cols {
        let mut found = false;
        for r in 0..a.rows {
            if a.data[r * a.cols + c] != 0.0 {
                found = true;
                break;
            }
        }
        data.push(if found { 1.0 } else { 0.0 });
    }
    Ok(DenseMatrix {
        rows: 1,
        cols: a.cols,
        data,
    })
}

pub fn all(a: &DenseMatrix) -> MatResult {
    let mut data = Vec::with_capacity(a.cols);
    for c in 0..a.cols {
        let mut all_nz = true;
        for r in 0..a.rows {
            if a.data[r * a.cols + c] == 0.0 {
                all_nz = false;
                break;
            }
        }
        data.push(if all_nz { 1.0 } else { 0.0 });
    }
    Ok(DenseMatrix {
        rows: 1,
        cols: a.cols,
        data,
    })
}

// ---------------------------------------------------------------------------
// Cumulative operations
// ---------------------------------------------------------------------------

pub fn cumsum(a: &DenseMatrix) -> MatResult {
    let mut data = vec![0.0; a.rows * a.cols];
    for c in 0..a.cols {
        let mut acc = 0.0;
        for r in 0..a.rows {
            acc += a.data[r * a.cols + c];
            data[r * a.cols + c] = acc;
        }
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

pub fn cumprod(a: &DenseMatrix) -> MatResult {
    let mut data = vec![0.0; a.rows * a.cols];
    for c in 0..a.cols {
        let mut acc = 1.0;
        for r in 0..a.rows {
            acc *= a.data[r * a.cols + c];
            data[r * a.cols + c] = acc;
        }
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// `diff(A)` / `diff(A, n)` / `diff(A, n, dim)` — differences.
pub fn diff(a: &DenseMatrix, n: usize, dim: usize) -> MatResult {
    if n == 0 {
        return Ok(a.clone());
    }
    if dim == 1 {
        // Along rows (dim 1)
        if a.rows <= n {
            return Ok(DenseMatrix {
                rows: 0,
                cols: a.cols,
                data: Vec::new(),
            });
        }
        let _out_rows = a.rows - n;
        // Apply n times
        let mut current = a.clone();
        for _ in 0..n {
            if current.rows <= 1 {
                return Ok(DenseMatrix {
                    rows: 0,
                    cols: a.cols,
                    data: Vec::new(),
                });
            }
            let new_rows = current.rows - 1;
            let mut data = vec![0.0; new_rows * current.cols];
            for c in 0..current.cols {
                for r in 0..new_rows {
                    data[r * current.cols + c] = current.data[(r + 1) * current.cols + c]
                        - current.data[r * current.cols + c];
                }
            }
            current = DenseMatrix {
                rows: new_rows,
                cols: current.cols,
                data,
            };
        }
        Ok(current)
    } else {
        // Along cols (dim 2)
        if a.cols <= n {
            return Ok(DenseMatrix {
                rows: a.rows,
                cols: 0,
                data: Vec::new(),
            });
        }
        let mut current = a.clone();
        for _ in 0..n {
            if current.cols <= 1 {
                return Ok(DenseMatrix {
                    rows: a.rows,
                    cols: 0,
                    data: Vec::new(),
                });
            }
            let new_cols = current.cols - 1;
            let mut data = vec![0.0; current.rows * new_cols];
            for r in 0..current.rows {
                for c in 0..new_cols {
                    data[r * new_cols + c] = current.data[r * current.cols + (c + 1)]
                        - current.data[r * current.cols + c];
                }
            }
            current = DenseMatrix {
                rows: current.rows,
                cols: new_cols,
                data,
            };
        }
        Ok(current)
    }
}

pub fn triangular(a: &DenseMatrix, upper: bool, k: isize, _span: Option<Span>) -> MatResult {
    let mut data = vec![0.0; a.data.len()];
    for r in 0..a.rows {
        for c in 0..a.cols {
            let d = (c as isize - r as isize) - k;
            let keep = if upper { d >= 0 } else { d <= 0 };
            if keep {
                data[r * a.cols + c] = a.data[r * a.cols + c];
            }
        }
    }
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// `any(A, dim)` — 1 if any element is nonzero along dimension `dim`.
pub fn any_dim(a: &DenseMatrix, dim: usize) -> MatResult {
    if dim == 1 {
        // Reduce along rows -> 1 x cols
        let mut data = Vec::with_capacity(a.cols);
        for c in 0..a.cols {
            let mut found = false;
            for r in 0..a.rows {
                if a.data[r * a.cols + c] != 0.0 {
                    found = true;
                    break;
                }
            }
            data.push(if found { 1.0 } else { 0.0 });
        }
        Ok(DenseMatrix {
            rows: 1,
            cols: a.cols,
            data,
        })
    } else {
        // Reduce along cols -> rows x 1
        let mut data = Vec::with_capacity(a.rows);
        for r in 0..a.rows {
            let mut found = false;
            for c in 0..a.cols {
                if a.data[r * a.cols + c] != 0.0 {
                    found = true;
                    break;
                }
            }
            data.push(if found { 1.0 } else { 0.0 });
        }
        Ok(DenseMatrix {
            rows: a.rows,
            cols: 1,
            data,
        })
    }
}

/// `all(A, dim)` — 1 if all elements are nonzero along dimension `dim`.
pub fn all_dim(a: &DenseMatrix, dim: usize) -> MatResult {
    if dim == 1 {
        let mut data = Vec::with_capacity(a.cols);
        for c in 0..a.cols {
            let mut all_nz = true;
            for r in 0..a.rows {
                if a.data[r * a.cols + c] == 0.0 {
                    all_nz = false;
                    break;
                }
            }
            data.push(if all_nz { 1.0 } else { 0.0 });
        }
        Ok(DenseMatrix {
            rows: 1,
            cols: a.cols,
            data,
        })
    } else {
        let mut data = Vec::with_capacity(a.rows);
        for r in 0..a.rows {
            let mut all_nz = true;
            for c in 0..a.cols {
                if a.data[r * a.cols + c] == 0.0 {
                    all_nz = false;
                    break;
                }
            }
            data.push(if all_nz { 1.0 } else { 0.0 });
        }
        Ok(DenseMatrix {
            rows: a.rows,
            cols: 1,
            data,
        })
    }
}
