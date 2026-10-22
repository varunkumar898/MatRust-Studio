use matlab_runtime::run_source;

#[test]
fn logspace_result() {
    let r = run_source("x = logspace(0, 2, 5)").unwrap();
    assert!(r.outputs.iter().any(|o| o.label.as_deref() == Some("x")));
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    // logspace(0,2,5) = [1, 10^0.5, 10, 10^1.5, 100]
    assert!((x.data[0] - 1.0).abs() < 1e-10);
    assert!((x.data[4] - 100.0).abs() < 1e-10);
}

#[test]
fn magic_squares() {
    // Odd order
    let r = run_source("M = magic(3); s = sum(M)").unwrap();
    let s = r.env.get("s").unwrap().as_matrix().unwrap();
    // All columns should sum to 15 (3*(3^2+1)/2)
    for v in &s.data {
        assert!((v - 15.0).abs() < 1e-10);
    }

    // Doubly-even order
    let r = run_source("M = magic(4); s = sum(M)").unwrap();
    let s = r.env.get("s").unwrap().as_matrix().unwrap();
    // All columns should sum to 34 (4*(4^2+1)/2)
    for v in &s.data {
        assert!((v - 34.0).abs() < 1e-10);
    }

    // n == 2 special case
    let r = run_source("M = magic(2)").unwrap();
    let M = r.env.get("M").unwrap().as_matrix().unwrap();
    assert_eq!(M.data, vec![1.0, 3.0, 4.0, 2.0]);

    // n == 0 empty
    let r = run_source("M = magic(0)").unwrap();
    let M = r.env.get("M").unwrap().as_matrix().unwrap();
    assert_eq!(M.rows, 0);
    assert_eq!(M.cols, 0);
}

#[test]
fn magic_singly_even_errors() {
    // n = 6 (singly even) should error
    assert!(run_source("magic(6)").is_err());
}

#[test]
fn vertcat_horzcat_cat() {
    let r = run_source("A = [1 2; 3 4]; B = [5 6; 7 8]; V = vertcat(A, B); H = horzcat(A, B); C1 = cat(1, A, B); C2 = cat(2, A, B)").unwrap();
    let V = r.env.get("V").unwrap().as_matrix().unwrap();
    let H = r.env.get("H").unwrap().as_matrix().unwrap();
    let C1 = r.env.get("C1").unwrap().as_matrix().unwrap();
    let C2 = r.env.get("C2").unwrap().as_matrix().unwrap();

    assert_eq!(V.rows, 4);
    assert_eq!(V.cols, 2);
    assert_eq!(V.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);

    assert_eq!(H.rows, 2);
    assert_eq!(H.cols, 4);
    // Row-major: row 0 = [1,2,5,6], row 1 = [3,4,7,8]
    assert_eq!(H.data, vec![1.0, 2.0, 5.0, 6.0, 3.0, 4.0, 7.0, 8.0]);

    assert_eq!(C1.rows, 4);
    assert_eq!(C1.cols, 2);
    assert_eq!(C2.rows, 2);
    assert_eq!(C2.cols, 4);
}

#[test]
fn cat_dimension_errors() {
    // cat with dim > 2 should error
    assert!(run_source("cat(3, [1 2], [3 4])").is_err());
    // vertcat with mismatched cols should error
    assert!(run_source("vertcat([1 2], [3 4 5])").is_err());
    // horzcat with mismatched rows should error
    assert!(run_source("horzcat([1; 2], [3; 4; 5])").is_err());
}

#[test]
fn blkdiag_basic() {
    let r = run_source("A = [1 2; 3 4]; B = [5 6; 7 8]; C = blkdiag(A, B)").unwrap();
    let C = r.env.get("C").unwrap().as_matrix().unwrap();
    assert_eq!(C.rows, 4);
    assert_eq!(C.cols, 4);
    assert_eq!(
        C.data,
        vec![1.0, 2.0, 0.0, 0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 5.0, 6.0, 0.0, 0.0, 7.0, 8.0,]
    );
}

#[test]
fn blkdiag_single() {
    let r = run_source("A = [1 2; 3 4]; C = blkdiag(A)").unwrap();
    let C = r.env.get("C").unwrap().as_matrix().unwrap();
    assert_eq!(C.rows, 2);
    assert_eq!(C.cols, 2);
    assert_eq!(C.data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn rand_deterministic() {
    // Fresh session should produce deterministic output from default seed
    let r1 = run_source("rng(42); x = rand(2,3)").unwrap();
    let r2 = run_source("rng(42); x = rand(2,3)").unwrap();
    let x1 = r1.env.get("x").unwrap().as_matrix().unwrap();
    let x2 = r2.env.get("x").unwrap().as_matrix().unwrap();
    assert_eq!(x1.data, x2.data);
}

#[test]
fn rand_output_range() {
    let r = run_source("rng(123); x = rand(100, 1)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    for &v in &x.data {
        assert!(v > 0.0 && v < 1.0, "rand value {v} not in (0,1)");
    }
}

#[test]
fn randn_deterministic() {
    let r1 = run_source("rng(42); x = randn(2,3)").unwrap();
    let r2 = run_source("rng(42); x = randn(2,3)").unwrap();
    let x1 = r1.env.get("x").unwrap().as_matrix().unwrap();
    let x2 = r2.env.get("x").unwrap().as_matrix().unwrap();
    assert_eq!(x1.data, x2.data);
}

#[test]
fn randn_stats() {
    let r = run_source("rng(123); x = randn(10000, 1)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    let mean: f64 = x.data.iter().sum::<f64>() / x.data.len() as f64;
    // Mean should be close to 0
    assert!(mean.abs() < 0.05, "mean {mean} too far from 0");
}

#[test]
fn randi_basic() {
    let r = run_source("rng(123); x = randi(10, 100, 1)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    for &v in &x.data {
        assert!(
            (1.0..=10.0).contains(&v),
            "randi value {v} out of range [1,10]"
        );
    }
}

#[test]
fn randi_range() {
    let r = run_source("rng(123); x = randi([5 10], 100, 1)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    for &v in &x.data {
        assert!(
            (5.0..=10.0).contains(&v),
            "randi value {v} out of range [5,10]"
        );
    }
}

#[test]
fn randi_degenerate_range() {
    let r = run_source("rng(123); x = randi([7 7], 10, 1)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    for &v in &x.data {
        assert_eq!(
            v, 7.0,
            "degenerate range should always return the single value"
        );
    }
}

#[test]
fn randperm_full() {
    let r = run_source("rng(123); p = randperm(10)").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.cols, 10);
    // Should be a permutation of 1..10
    let mut sorted = p.data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(sorted, (1..=10).map(|i| i as f64).collect::<Vec<_>>());
}

#[test]
fn randperm_partial() {
    let r = run_source("rng(123); p = randperm(10, 5)").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.cols, 5);
    // All elements should be unique and in 1..10
    let mut sorted = p.data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for (i, &v) in sorted.iter().enumerate() {
        assert!((1.0..=10.0).contains(&v));
    }
    // Check uniqueness
    for i in 1..sorted.len() {
        assert_ne!(sorted[i], sorted[i - 1]);
    }
}

#[test]
fn randperm_error_k_exceeds_n() {
    assert!(run_source("randperm(5, 10)").is_err());
}

#[test]
fn rand_size_syntaxes() {
    // rand()
    let r = run_source("rng(1); x = rand()").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    assert_eq!(x.rows, 1);
    assert_eq!(x.cols, 1);

    // rand(n)
    let r = run_source("rng(1); x = rand(3)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    assert_eq!(x.rows, 3);
    assert_eq!(x.cols, 3);

    // rand(m,n)
    let r = run_source("rng(1); x = rand(2,4)").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 4);

    // rand([m n])
    let r = run_source("rng(1); x = rand([2 4])").unwrap();
    let x = r.env.get("x").unwrap().as_matrix().unwrap();
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 4);
}
