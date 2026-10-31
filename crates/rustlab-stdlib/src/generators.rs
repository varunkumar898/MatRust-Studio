//! Array generators: zeros, ones, eye, rand, randn, linspace, logspace, magic.

use crate::helpers::{math_err, parse_dims, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};
use std::sync::atomic::{AtomicU64, Ordering};

// Simple thread-safe SplitMix64 PRNG state
static RNG_STATE: AtomicU64 = AtomicU64::new(0x853c49e6748fea9b);

fn next_u64() -> u64 {
    let old = RNG_STATE.fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed);
    let mut z = old.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn next_f64() -> f64 {
    // Generate uniform float in (0, 1)
    (next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
}

fn next_randn() -> f64 {
    // Box-Muller transform for standard normal N(0, 1)
    let u1 = next_f64().max(1e-15);
    let u2 = next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

pub fn zeros(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let arr = DenseArray::from_vec(dims, vec![0.0; count]).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn ones(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let arr = DenseArray::from_vec(dims, vec![1.0; count]).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn eye(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    if dims.len() != 2 {
        return Err(user_err(
            "MATLAB:eye:invalidDims",
            "eye expects 1 or 2 dimensions.",
        ));
    }
    let (m, n) = (dims[0], dims[1]);
    let mut data = vec![0.0; m * n];
    let k = m.min(n);
    for i in 0..k {
        // column-major index: i + i * m
        data[i + i * m] = 1.0;
    }
    let arr = DenseArray::from_vec(vec![m, n], data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn rand(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let mut data = Vec::with_capacity(count);
    for _ in 0..count {
        data.push(next_f64());
    }
    let arr = DenseArray::from_vec(dims, data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn randn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let mut data = Vec::with_capacity(count);
    for _ in 0..count {
        data.push(next_randn());
    }
    let arr = DenseArray::from_vec(dims, data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn linspace(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "Not enough input arguments for linspace.",
        ));
    }
    let x1 = args[0].as_scalar_f64().map_err(math_err)?;
    let x2 = args[1].as_scalar_f64().map_err(math_err)?;
    let n = if args.len() >= 3 {
        args[2].as_scalar_f64().map_err(math_err)? as isize
    } else {
        100
    };

    if n <= 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    if n == 1 {
        return Ok(vec![Value::scalar(x2)]);
    }

    let count = n as usize;
    let step = (x2 - x1) / (count - 1) as f64;
    let mut data = Vec::with_capacity(count);
    for i in 0..count {
        if i == count - 1 {
            data.push(x2);
        } else {
            data.push(x1 + (i as f64) * step);
        }
    }
    let arr = DenseArray::row_vector(data);
    Ok(vec![Value::Double(arr)])
}

pub fn logspace(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "Not enough input arguments for logspace.",
        ));
    }
    let a = args[0].as_scalar_f64().map_err(math_err)?;
    let mut b = args[1].as_scalar_f64().map_err(math_err)?;
    let n = if args.len() >= 3 {
        args[2].as_scalar_f64().map_err(math_err)? as isize
    } else {
        50
    };

    // In MATLAB: if b == pi, the interval ends at pi instead of 10^pi
    let ends_at_pi = (b - std::f64::consts::PI).abs() < 1e-12;
    if ends_at_pi {
        b = std::f64::consts::PI.log10();
    }

    if n <= 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let count = n as usize;
    let lin = linspace(
        &[Value::scalar(a), Value::scalar(b), Value::scalar(n as f64)],
        1,
    )?;
    let lin_arr = lin[0].to_double_array().map_err(math_err)?;
    let mut data = Vec::with_capacity(count);
    for &p in lin_arr.as_slice() {
        data.push(10.0_f64.powf(p));
    }
    if ends_at_pi && count > 0 {
        data[count - 1] = std::f64::consts::PI;
    }
    Ok(vec![Value::Double(DenseArray::row_vector(data))])
}

pub fn magic(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "magic requires 1 input."));
    }
    let n_flt = args[0].as_scalar_f64().map_err(math_err)?;
    let n = n_flt as isize;
    if n <= 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    let n = n as usize;
    if n == 2 {
        // Standard MATLAB magic(2)
        let data = vec![1.0, 4.0, 3.0, 2.0]; // col-major for [1 3; 4 2]
        let arr = DenseArray::from_vec(vec![2, 2], data).map_err(math_err)?;
        return Ok(vec![Value::Double(arr)]);
    }

    let mut grid = vec![vec![0.0; n]; n];

    if n % 2 == 1 {
        // Odd n: Siamese method
        let mut r = 0;
        let mut c = n / 2;
        for num in 1..=(n * n) {
            grid[r][c] = num as f64;
            let next_r = if r == 0 { n - 1 } else { r - 1 };
            let next_c = (c + 1) % n;
            if grid[next_r][next_c] != 0.0 {
                r = (r + 1) % n;
            } else {
                r = next_r;
                c = next_c;
            }
        }
    } else if n.is_multiple_of(4) {
        // Doubly even n: complement pattern
        for r in 0..n {
            for c in 0..n {
                let num = r * n + c + 1;
                let cond1 = (r % 4 == c % 4) || ((r % 4 + c % 4) == 3);
                if cond1 {
                    grid[r][c] = (n * n + 1 - num) as f64;
                } else {
                    grid[r][c] = num as f64;
                }
            }
        }
    } else {
        // Singly even n: Strachey's method (4 quadrants)
        let p = n / 2;
        let p_magic = magic(&[Value::scalar(p as f64)], 1)?;
        let p_arr = p_magic[0].to_double_array().map_err(math_err)?;

        for r in 0..p {
            for c in 0..p {
                let base = *p_arr.get_subscript(&[r + 1, c + 1])?;
                grid[r][c] = base; // A
                grid[r + p][c + p] = base + (p * p) as f64; // B
                grid[r][c + p] = base + (2 * p * p) as f64; // C
                grid[r + p][c] = base + (3 * p * p) as f64; // D
            }
        }

        let k = (n - 2) / 4;
        for r in 0..p {
            for c in 0..k {
                let tmp = grid[r][c];
                grid[r][c] = grid[r + p][c];
                grid[r + p][c] = tmp;
            }
            if k > 1 {
                for c in (n - k + 1)..n {
                    let tmp = grid[r][c];
                    grid[r][c] = grid[r + p][c];
                    grid[r + p][c] = tmp;
                }
            }
        }
        let tmp = grid[k][0];
        grid[k][0] = grid[k + p][0];
        grid[k + p][0] = tmp;

        let tmp = grid[k][k];
        grid[k][k] = grid[k + p][k];
        grid[k + p][k] = tmp;
    }

    // Convert row-major grid to col-major DenseArray
    let mut col_major = Vec::with_capacity(n * n);
    for c in 0..n {
        for r in 0..n {
            col_major.push(grid[r][c]);
        }
    }
    let arr = DenseArray::from_vec(vec![n, n], col_major).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn true_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let arr = DenseArray::from_vec(dims, vec![true; count]).map_err(math_err)?;
    Ok(vec![Value::Logical(arr)])
}

pub fn false_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let dims = parse_dims(args)?;
    let count: usize = dims.iter().product();
    let arr = DenseArray::from_vec(dims, vec![false; count]).map_err(math_err)?;
    Ok(vec![Value::Logical(arr)])
}
