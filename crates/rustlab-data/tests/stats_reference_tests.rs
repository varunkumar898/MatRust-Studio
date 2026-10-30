use rustlab_data::stats::{cummax, movmean, zscore};
use rustlab_math::{DenseArray, Value};

fn to_col(vals: &[f64]) -> Value {
    let da = DenseArray::from_vec(vec![vals.len(), 1], vals.to_vec()).expect("valid dense array");
    Value::Double(da)
}

fn from_val(v: &[Value]) -> Vec<f64> {
    match &v[0] {
        Value::Double(da) => da.as_slice().to_vec(),
        _ => panic!("expected Double array"),
    }
}

// Naive reference implementations (O(n*k) and two-pass high-precision)

fn naive_movmean_window(vals: &[f64], k: usize) -> Vec<f64> {
    let n = vals.len();
    if n == 0 {
        return Vec::new();
    }
    let left = (k - 1) / 2;
    let right = k.saturating_sub(1) - left;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = i.saturating_sub(left);
        let b = (i + right).min(n - 1);
        let window = &vals[a..=b];
        let mut sum = 0.0;
        let mut has_nan = false;
        for &x in window {
            if x.is_nan() {
                has_nan = true;
            }
            sum += x;
        }
        if has_nan {
            out.push(f64::NAN);
        } else {
            out.push(sum / window.len() as f64);
        }
    }
    out
}

fn naive_zscore(vals: &[f64]) -> Vec<f64> {
    let n = vals.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![f64::NAN];
    }
    // High-precision two-pass
    let mean = vals.iter().sum::<f64>() / n as f64;
    let ss: f64 = vals.iter().map(|&x| (x - mean) * (x - mean)).sum();
    let sd = (ss / (n - 1) as f64).sqrt();
    if sd > 0.0 {
        vals.iter().map(|&x| (x - mean) / sd).collect()
    } else {
        vec![f64::NAN; n]
    }
}

fn naive_cummax(vals: &[f64]) -> Vec<f64> {
    let n = vals.len();
    if n == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n);
    let mut poisoned = false;
    let mut current_max = f64::NEG_INFINITY;
    for &x in vals {
        if poisoned || x.is_nan() {
            poisoned = true;
            out.push(f64::NAN);
        } else {
            if x > current_max {
                current_max = x;
            }
            out.push(current_max);
        }
    }
    out
}

fn assert_approx_eq(actual: &[f64], expected: &[f64], tol: f64, desc: &str) {
    assert_eq!(actual.len(), expected.len(), "{desc}: length mismatch");
    for (i, (&act, &exp)) in actual.iter().zip(expected.iter()).enumerate() {
        if exp.is_nan() {
            assert!(act.is_nan(), "{desc} at index {i}: expected NaN, got {act}");
        } else if exp.is_infinite() {
            assert_eq!(
                act, exp,
                "{desc} at index {i}: infinite mismatch act={act}, exp={exp}"
            );
        } else {
            let diff = (act - exp).abs();
            let scale = exp.abs().max(1.0);
            assert!(
                diff <= tol * scale,
                "{desc} at index {i}: diff={diff} > tol*scale ({}), act={act}, exp={exp}",
                tol * scale
            );
        }
    }
}

// Simple seeded LCG PRNG for reproducible test data
fn lcg_data(n: usize, seed: u64, min: f64, max: f64) -> Vec<f64> {
    let mut state = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let u = (state >> 11) as f64 / ((1u64 << 53) as f64);
        out.push(min + u * (max - min));
    }
    out
}

#[test]
fn test_reference_empty_and_len1() {
    // Empty
    let empty: Vec<f64> = vec![];
    let mm = movmean(&[to_col(&empty), Value::scalar(3.0)], 1).expect("mm empty");
    assert_eq!(from_val(&mm).len(), 0);

    let zs = zscore(&[to_col(&empty)], 1).expect("zs empty");
    assert_eq!(from_val(&zs).len(), 0);

    let cm = cummax(&[to_col(&empty)], 1).expect("cm empty");
    assert_eq!(from_val(&cm).len(), 0);

    // Len-1 normal
    let one = vec![42.0];
    let mm = movmean(&[to_col(&one), Value::scalar(3.0)], 1).expect("mm 1");
    assert_approx_eq(
        &from_val(&mm),
        &naive_movmean_window(&one, 3),
        1e-12,
        "mm len1",
    );

    let zs = zscore(&[to_col(&one)], 1).expect("zs 1");
    assert_approx_eq(&from_val(&zs), &naive_zscore(&one), 1e-12, "zs len1");

    let cm = cummax(&[to_col(&one)], 1).expect("cm 1");
    assert_approx_eq(&from_val(&cm), &naive_cummax(&one), 1e-12, "cm len1");

    // Len-1 NaN
    let one_nan = vec![f64::NAN];
    let mm = movmean(&[to_col(&one_nan), Value::scalar(3.0)], 1).expect("mm nan");
    assert!(from_val(&mm)[0].is_nan());

    let zs = zscore(&[to_col(&one_nan)], 1).expect("zs nan");
    assert!(from_val(&zs)[0].is_nan());

    let cm = cummax(&[to_col(&one_nan)], 1).expect("cm nan");
    assert!(from_val(&cm)[0].is_nan());

    // Len-1 Inf
    let one_inf = vec![f64::INFINITY];
    let mm = movmean(&[to_col(&one_inf), Value::scalar(3.0)], 1).expect("mm inf");
    assert_eq!(from_val(&mm)[0], f64::INFINITY);

    let zs = zscore(&[to_col(&one_inf)], 1).expect("zs inf");
    assert!(from_val(&zs)[0].is_nan());

    let cm = cummax(&[to_col(&one_inf)], 1).expect("cm inf");
    assert_eq!(from_val(&cm)[0], f64::INFINITY);
}

#[test]
fn test_reference_constant_data() {
    let const_data = vec![7.25; 500];

    // k=1, k=3, k=50, k=len
    for &k in &[1, 3, 50, 500] {
        let mm = movmean(&[to_col(&const_data), Value::scalar(k as f64)], 1).expect("mm const");
        assert_approx_eq(
            &from_val(&mm),
            &naive_movmean_window(&const_data, k),
            1e-12,
            &format!("mm const k={k}"),
        );
    }

    let zs = zscore(&[to_col(&const_data)], 1).expect("zs const");
    assert_approx_eq(
        &from_val(&zs),
        &naive_zscore(&const_data),
        1e-12,
        "zs const",
    );

    let cm = cummax(&[to_col(&const_data)], 1).expect("cm const");
    assert_approx_eq(
        &from_val(&cm),
        &naive_cummax(&const_data),
        1e-12,
        "cm const",
    );
}

#[test]
fn test_reference_random_data() {
    let rand_data = lcg_data(1000, 1234567, -100.0, 100.0);

    for &k in &[1, 3, 7, 31, 1000] {
        let mm = movmean(&[to_col(&rand_data), Value::scalar(k as f64)], 1).expect("mm rand");
        assert_approx_eq(
            &from_val(&mm),
            &naive_movmean_window(&rand_data, k),
            1e-11,
            &format!("mm rand k={k}"),
        );
    }

    let zs = zscore(&[to_col(&rand_data)], 1).expect("zs rand");
    assert_approx_eq(&from_val(&zs), &naive_zscore(&rand_data), 1e-11, "zs rand");

    let cm = cummax(&[to_col(&rand_data)], 1).expect("cm rand");
    assert_approx_eq(&from_val(&cm), &naive_cummax(&rand_data), 1e-12, "cm rand");
}

#[test]
fn test_reference_offset_1e8_plus_noise() {
    // Tests catastrophic cancellation and running sum drift on large DC bias
    let offset = 1e8;
    let noise = lcg_data(2000, 987654321, -1.0, 1.0);
    let data: Vec<f64> = noise.iter().map(|&x| offset + x).collect();

    for &k in &[3, 11, 101] {
        let mm = movmean(&[to_col(&data), Value::scalar(k as f64)], 1).expect("mm 1e8");
        // Relative tolerance 1e-10 of 1e8 is 0.01
        assert_approx_eq(
            &from_val(&mm),
            &naive_movmean_window(&data, k),
            1e-10,
            &format!("mm 1e8 k={k}"),
        );
    }

    let zs = zscore(&[to_col(&data)], 1).expect("zs 1e8");
    assert_approx_eq(&from_val(&zs), &naive_zscore(&data), 1e-6, "zs 1e8");

    let cm = cummax(&[to_col(&data)], 1).expect("cm 1e8");
    assert_approx_eq(&from_val(&cm), &naive_cummax(&data), 1e-12, "cm 1e8");
}

#[test]
fn test_reference_nan_and_inf_windows() {
    let mut data = lcg_data(200, 424242, 10.0, 50.0);
    // Inject NaNs and Infs
    data[0] = f64::NAN; // edge
    data[5] = f64::NAN; // isolated
    data[6] = f64::NAN; // consecutive
    data[50] = f64::INFINITY; // Inf
    data[100] = f64::NEG_INFINITY; // -Inf
    data[199] = f64::NAN; // last edge

    for &k in &[1, 3, 5, 11] {
        let mm = movmean(&[to_col(&data), Value::scalar(k as f64)], 1).expect("mm nan/inf");
        assert_approx_eq(
            &from_val(&mm),
            &naive_movmean_window(&data, k),
            1e-12,
            &format!("mm nan/inf k={k}"),
        );
    }

    let cm = cummax(&[to_col(&data), Value::scalar(1.0)], 1).expect("cm nan/inf");
    assert_approx_eq(&from_val(&cm), &naive_cummax(&data), 1e-12, "cm nan/inf");
}

#[test]
fn test_reference_window_edges_and_k_equals_len() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let k = data.len();

    let mm = movmean(&[to_col(&data), Value::scalar(k as f64)], 1).expect("mm k=len");
    assert_approx_eq(
        &from_val(&mm),
        &naive_movmean_window(&data, k),
        1e-12,
        "mm k=len",
    );

    let k1 = 1;
    let mm1 = movmean(&[to_col(&data), Value::scalar(k1 as f64)], 1).expect("mm k=1");
    assert_approx_eq(
        &from_val(&mm1),
        &naive_movmean_window(&data, k1),
        1e-12,
        "mm k=1",
    );
}

#[test]
fn test_reference_drift_over_100k_elements() {
    // 100,000 elements to check drift over long runs
    let data = lcg_data(100_000, 777111, 1.0, 10.0);
    let k: usize = 15;

    let mm = movmean(&[to_col(&data), Value::scalar(k as f64)], 1).expect("mm 100k");
    // Verify against naive window at sample checkpoints (to keep test fast)
    let actual = from_val(&mm);
    let left = (k - 1) / 2;
    let right = k.saturating_sub(1) - left;

    // Check every 1000th element and edges
    let indices = (0usize..50)
        .chain((50..100_000).step_by(1000))
        .chain(99_950..100_000);
    for idx in indices {
        let a = idx.saturating_sub(left);
        let b = (idx + right).min(data.len() - 1);
        let expected: f64 = data[a..=b].iter().sum::<f64>() / (b - a + 1) as f64;
        let diff = (actual[idx] - expected).abs();
        assert!(
            diff < 1e-10,
            "drift at index {idx}: actual={}, expected={}, diff={diff}",
            actual[idx],
            expected
        );
    }
}
