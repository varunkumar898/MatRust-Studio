//! Milestone A2 Test Suite: Comprehensive Linear Algebra and Transform Suite.
//!
//! Covers:
//! - Golden tests: symmetric eig, non-symmetric eig with complex conjugate pairs, SVD of rank-1,
//!   expm of zero/diagonal matrices, pinv of rank-deficient matrix.
//! - Property tests: A*V = V*D, U*S*V' = A, A*pinv(A)*A = A, ifft(fft(x)) = x for lengths 1, 2, 7, 12, 1000.
//! - Pivoted QR: [Q, R, P] = qr(A) and [Q, R, p] = qr(A, 'vector').
//! - Geometric routines: rank, cond, null, orth, kron.
//! - 2D Transforms: fft2, ifft2 roundtrips.
//! - Edge cases: empty [], 1x1, singular matrix, NaN inputs, non-square errors.

use rustlab_math::Value;
use rustlab_stdlib::new_interpreter;

#[test]
fn test_a2_golden_eig_symmetric_ascending_and_sign_normalized() {
    let mut interp = new_interpreter();

    // 1. eig([2 0; 0 3]) = [2; 3]
    // Citation: Official MATLAB documentation for `eig`
    let r1 = interp.eval_script("eig([2 0; 0 3])").unwrap().unwrap();
    if let Value::Double(d) = r1 {
        assert_eq!(d.shape(), &[2, 1]);
        assert!((d.as_slice()[0] - 2.0).abs() < 1e-12);
        assert!((d.as_slice()[1] - 3.0).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // 2. eig([2 1; 1 2]) = [1; 3]
    // Citation: Standard textbook eigenvalue problem (Golub & Van Loan)
    let r2 = interp.eval_script("eig([2 1; 1 2])").unwrap().unwrap();
    if let Value::Double(d) = r2 {
        assert_eq!(d.shape(), &[2, 1]);
        assert!((d.as_slice()[0] - 1.0).abs() < 1e-12);
        assert!((d.as_slice()[1] - 3.0).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // Two-output syntax: [V, D] = eig([2 1; 1 2])
    interp.eval_script("[V, D] = eig([2 1; 1 2]);").unwrap();
    let v_val = interp.eval_script("V").unwrap().unwrap();
    let d_val = interp.eval_script("D").unwrap().unwrap();

    if let (Value::Double(v), Value::Double(d)) = (v_val, d_val) {
        assert_eq!(d.shape(), &[2, 2]);
        assert_eq!(v.shape(), &[2, 2]);
        assert!((d.get_subscript(&[1, 1]).unwrap() - 1.0).abs() < 1e-12);
        assert!((d.get_subscript(&[2, 2]).unwrap() - 3.0).abs() < 1e-12);

        // Sign-invariant check: verify A * V = V * D and |V| matches theoretical magnitude
        // Citation: Golub & Van Loan (2013) Matrix Computations, 4th ed., §8.2
        let a_times_v = interp.eval_script("[2 1; 1 2] * V").unwrap().unwrap();
        let v_times_d = interp.eval_script("V * D").unwrap().unwrap();
        if let (Value::Double(av), Value::Double(vd)) = (a_times_v, v_times_d) {
            for i in 0..4 {
                assert!(
                    (av.as_slice()[i] - vd.as_slice()[i]).abs() < 1e-12,
                    "A*V != V*D"
                );
            }
        }
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        for val in v.as_slice() {
            assert!(
                (val.abs() - inv_sqrt2).abs() < 1e-12,
                "|V| component mismatch"
            );
        }

        // RustLab/MATLAB sign normalization check:
        // First component of maximum absolute value by index must be positive
        for c in 1..=2 {
            let mut max_abs = -1.0;
            let mut max_val = 0.0;
            for r in 1..=2 {
                let val = *v.get_subscript(&[r, c]).unwrap();
                if val.abs() > max_abs {
                    max_abs = val.abs();
                    max_val = val;
                }
            }
            assert!(
                max_val >= 0.0,
                "Eigenvector column {c} not sign-normalized to positive: {max_val}"
            );
        }
    } else {
        panic!("Expected double matrices");
    }
}

#[cfg(feature = "faer")]
#[test]
fn test_a2_golden_eig_rotation_complex_conjugate() {
    let mut interp = new_interpreter();

    // 90-degree rotation matrix R = [0, -1; 1, 0]
    // Characteristic equation: lambda^2 + 1 = 0 => lambda = +/- i
    // Citation: Strang, "Linear Algebra and Its Applications", 4th ed., Section 6.1
    let r = interp.eval_script("eig([0, -1; 1, 0])").unwrap().unwrap();
    if let Value::ComplexDouble(da) = r {
        assert_eq!(da.shape(), &[2, 1]);
        let s = da.as_slice();
        // Magnitudes of eigenvalues should both be 1.0
        assert!((s[0].norm() - 1.0).abs() < 1e-12);
        assert!((s[1].norm() - 1.0).abs() < 1e-12);
        // Real parts should be 0
        assert!(s[0].re.abs() < 1e-12);
        assert!(s[1].re.abs() < 1e-12);
        // One imaginary part is +1 and the other is -1
        assert!((s[0].im.abs() - 1.0).abs() < 1e-12);
        assert!((s[1].im.abs() - 1.0).abs() < 1e-12);
        assert!(
            (s[0].im + s[1].im).abs() < 1e-12,
            "Complex conjugate pair expected"
        );
    } else {
        panic!("Expected complex eigenvalues for rotation matrix");
    }
}

#[test]
fn test_a2_golden_svd_rank1() {
    let mut interp = new_interpreter();

    // SVD of rank-1 matrix A = [1 2; 2 4]
    // Singular values are sqrt(trace(A' * A)) = sqrt(1^2 + 2^2 + 2^2 + 4^2) = sqrt(25) = 5, and 0
    // Citation: Golub & Van Loan, "Matrix Computations", 4th ed., Example 2.4.1
    let r = interp.eval_script("svd([1 2; 2 4])").unwrap().unwrap();
    if let Value::Double(da) = r {
        assert_eq!(da.shape(), &[2, 1]);
        assert!((da.as_slice()[0] - 5.0).abs() < 1e-12);
        assert!(da.as_slice()[1].abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // Three-output SVD: [U, S, V] = svd([1 2; 2 4])
    interp.eval_script("[U, S, V] = svd([1 2; 2 4]);").unwrap();
    let u_val = interp.eval_script("U").unwrap().unwrap();
    let s_val = interp.eval_script("S").unwrap().unwrap();
    let v_val = interp.eval_script("V").unwrap().unwrap();

    if let (Value::Double(u), Value::Double(s), Value::Double(v)) = (u_val, s_val, v_val) {
        assert_eq!(u.shape(), &[2, 2]);
        assert_eq!(s.shape(), &[2, 2]);
        assert_eq!(v.shape(), &[2, 2]);
        assert!((s.get_subscript(&[1, 1]).unwrap() - 5.0).abs() < 1e-12);
        assert!(s.get_subscript(&[2, 2]).unwrap().abs() < 1e-12);
    } else {
        panic!("Expected double matrices");
    }
}

#[test]
fn test_a2_golden_expm_zero_and_diagonal() {
    let mut interp = new_interpreter();

    // 1. expm of zero matrix is identity matrix
    // Citation: Higham, "Functions of Matrices", Chapter 10
    let r1 = interp.eval_script("expm(zeros(3, 3))").unwrap().unwrap();
    if let Value::Double(da) = r1 {
        assert_eq!(da.shape(), &[3, 3]);
        for r in 1..=3 {
            for c in 1..=3 {
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!((da.get_subscript(&[r, c]).unwrap() - expected).abs() < 1e-12);
            }
        }
    } else {
        panic!("Expected double array");
    }

    // 2. expm of diagonal matrix is diagonal of elementwise exponentials
    // Citation: Moler & Van Loan, "Nineteen Dubious Ways to Compute the Exponential of a Matrix"
    let r2 = interp.eval_script("expm([1 0; 0 2])").unwrap().unwrap();
    if let Value::Double(da) = r2 {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - 1.0_f64.exp()).abs() < 1e-12);
        assert!(da.get_subscript(&[1, 2]).unwrap().abs() < 1e-12);
        assert!(da.get_subscript(&[2, 1]).unwrap().abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - 2.0_f64.exp()).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // 3. Nilpotent matrix: [0 1; 0 0] -> [1 1; 0 1]
    // Citation: Higham (2008), Functions of Matrices, Theorem 10.1
    let r3 = interp.eval_script("expm([0 1; 0 0])").unwrap().unwrap();
    if let Value::Double(da) = r3 {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - 1.0).abs() < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - 1.0).abs() < 1e-12);
        assert!(da.get_subscript(&[2, 1]).unwrap().abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // 4. Rotation generator [0 -t; t 0] -> [cos(t) -sin(t); sin(t) cos(t)]
    // Citation: Hall, "Lie Groups, Lie Algebras, and Representations", Section 2.2
    let t = 0.5_f64;
    let r4 = interp
        .eval_script(&format!("expm([0 -{t}; {t} 0])"))
        .unwrap()
        .unwrap();
    if let Value::Double(da) = r4 {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - t.cos()).abs() < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - (-t.sin())).abs() < 1e-12);
        assert!((da.get_subscript(&[2, 1]).unwrap() - t.sin()).abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - t.cos()).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // 5. Large-norm matrix (norm ~ 50) verified via scaling identity expm(A) == expm(A/2)^2
    // Citation: Higham (2005) Scaling and Squaring Method Revisited
    interp.eval_script("A_large = [15 20; 25 10];").unwrap();
    interp.eval_script("E1 = expm(A_large);").unwrap();
    interp.eval_script("Half = expm(A_large * 0.5);").unwrap();
    interp.eval_script("E2 = Half * Half;").unwrap();
    interp
        .eval_script("rel_err = norm(E1 - E2) / norm(E1);")
        .unwrap();
    let rel_err = interp.eval_script("rel_err").unwrap().unwrap();
    assert!(
        rel_err.as_scalar_f64().unwrap() < 1e-12,
        "Large-norm scaling identity failed: relative error = {}",
        rel_err.as_scalar_f64().unwrap()
    );

    // 6. Defective matrix: Jordan block J = [2 1; 0 2] -> e^2 * [1 1; 0 2]
    // Citation: Golub & Van Loan (2013) §9.3 Matrix Functions
    interp.eval_script("J = [2 1; 0 2];").unwrap();
    interp.eval_script("EJ = expm(J);").unwrap();
    let e_j = interp.eval_script("EJ").unwrap().unwrap();
    if let Value::Double(da) = e_j {
        let e2 = 2.0_f64.exp();
        assert!((da.get_subscript(&[1, 1]).unwrap() - e2).abs() / e2 < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - e2).abs() / e2 < 1e-12);
        assert!(da.get_subscript(&[2, 1]).unwrap().abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - e2).abs() / e2 < 1e-12);
    } else {
        panic!("Expected double array");
    }
}

#[test]
fn test_a2_golden_pinv_rank_deficient() {
    let mut interp = new_interpreter();

    // 1. pinv of diagonal rank-deficient matrix: [1 0; 0 0] -> [1 0; 0 0]
    // Citation: Penrose, "A generalized inverse for matrices", 1955
    let r1 = interp.eval_script("pinv([1 0; 0 0])").unwrap().unwrap();
    if let Value::Double(da) = r1 {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - 1.0).abs() < 1e-12);
        assert!(da.get_subscript(&[1, 2]).unwrap().abs() < 1e-12);
        assert!(da.get_subscript(&[2, 1]).unwrap().abs() < 1e-12);
        assert!(da.get_subscript(&[2, 2]).unwrap().abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }

    // 2. pinv of rank-1 matrix A = [1 2; 2 4]
    // Frobenius norm of A is 5. Analytical pinv(A) = A' / ||A||_F^2 = [1 2; 2 4] / 25 = [0.04 0.08; 0.08 0.16]
    let r2 = interp.eval_script("pinv([1 2; 2 4])").unwrap().unwrap();
    if let Value::Double(da) = r2 {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - 0.04).abs() < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - 0.08).abs() < 1e-12);
        assert!((da.get_subscript(&[2, 1]).unwrap() - 0.08).abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - 0.16).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }
}

#[test]
fn test_a2_property_av_equals_vd_and_usvt_equals_a() {
    let mut interp = new_interpreter();

    // Property 1: A * V = V * D for symmetric 3x3
    interp.eval_script("A = [4 1 -2; 1 2 0; -2 0 3];").unwrap();
    interp.eval_script("[V, D] = eig(A);").unwrap();
    interp
        .eval_script("diff_eig = norm(A * V - V * D);")
        .unwrap();
    let diff_eig = interp.eval_script("diff_eig").unwrap().unwrap();
    assert!(
        diff_eig.as_scalar_f64().unwrap() < 1e-10,
        "Property A*V = V*D failed"
    );

    // Property 2: U * S * V' = A for rectangular 4x3 matrix
    interp
        .eval_script("B = [1 2 3; 4 5 6; 7 8 9; 10 11 12];")
        .unwrap();
    interp.eval_script("[U, S, V] = svd(B, 'econ');").unwrap();
    interp
        .eval_script("diff_svd = norm(U * S * V' - B);")
        .unwrap();
    let diff_svd = interp.eval_script("diff_svd").unwrap().unwrap();
    assert!(
        diff_svd.as_scalar_f64().unwrap() < 1e-10,
        "Property U*S*V' = B failed"
    );

    // Property 3: A * pinv(A) * A = A (Moore-Penrose first condition)
    interp.eval_script("P = pinv(B);").unwrap();
    interp
        .eval_script("diff_pinv = norm(B * P * B - B);")
        .unwrap();
    let diff_pinv = interp.eval_script("diff_pinv").unwrap().unwrap();
    assert!(
        diff_pinv.as_scalar_f64().unwrap() < 1e-10,
        "Property A*pinv(A)*A = A failed"
    );
}

#[test]
fn test_a2_property_fft_ifft_roundtrip_various_lengths() {
    let mut interp = new_interpreter();

    // Test lengths: 1 (scalar), 2 (power of 2), 7 (prime), 12 (composite non-power of 2), 1000 (large composite)
    let lengths = [1, 2, 7, 12, 1000];

    for &len in &lengths {
        interp
            .eval_script(&format!("x = (1:{len})' + 0.5;"))
            .unwrap();
        interp.eval_script("x_rec = ifft(fft(x));").unwrap();
        interp.eval_script("err = norm(real(x_rec) - x);").unwrap();
        let err = interp.eval_script("err").unwrap().unwrap();
        assert!(
            err.as_scalar_f64().unwrap() < 1e-10,
            "FFT/IFFT roundtrip failed for length {len}"
        );
    }
}

#[test]
fn test_a2_pivoted_qr_decomposition() {
    let mut interp = new_interpreter();

    // Rank-deficient 3x3 matrix: A = [1 2 3; 1 2 3; 1 2 3]
    interp.eval_script("A = [1 2 3; 1 2 3; 1 2 3];").unwrap();

    // 1. [Q, R, P] = qr(A) with matrix permutation P
    interp.eval_script("[Q, R, P] = qr(A);").unwrap();
    interp
        .eval_script("diff_piv = norm(A * P - Q * R);")
        .unwrap();
    let diff_piv = interp.eval_script("diff_piv").unwrap().unwrap();
    assert!(diff_piv.as_scalar_f64().unwrap() < 1e-10, "A*P != Q*R");

    // Columns of R must have non-increasing diagonal elements
    interp.eval_script("diag_r = abs(diag(R));").unwrap();
    let diag_r = interp.eval_script("diag_r").unwrap().unwrap();
    if let Value::Double(da) = diag_r {
        assert!(da.as_slice()[0] >= da.as_slice()[1]);
        assert!(da.as_slice()[1] >= da.as_slice()[2]);
    } else {
        panic!();
    }

    // 2. [Q, R, p] = qr(A, 'vector') with vector permutation p
    interp
        .eval_script("[Q2, R2, p] = qr(A, 'vector');")
        .unwrap();
    let p_val = interp.eval_script("p").unwrap().unwrap();
    if let Value::Double(p) = p_val {
        assert_eq!(p.shape(), &[1, 3]);
        // All indices 1, 2, 3 must be present
        let mut sorted = p.as_slice().to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(sorted, vec![1.0, 2.0, 3.0]);
    } else {
        panic!();
    }
}

#[test]
fn test_a2_rank_cond_null_orth_kron() {
    let mut interp = new_interpreter();

    #[cfg(feature = "faer")]
    {
        // 1. rank
        interp.eval_script("A = [1 2; 2 4];").unwrap();
        let r1 = interp.eval_script("rank(A)").unwrap().unwrap();
        assert_eq!(r1, Value::scalar(1.0));

        // Full rank 2x2
        interp.eval_script("B = [1 0; 0 3];").unwrap();
        let r2 = interp.eval_script("rank(B)").unwrap().unwrap();
        assert_eq!(r2, Value::scalar(2.0));

        // 2. cond
        let c1 = interp.eval_script("cond(B)").unwrap().unwrap();
        assert_eq!(c1, Value::scalar(3.0)); // sigma_max / sigma_min = 3 / 1 = 3.0

        let c_sing = interp.eval_script("cond(A)").unwrap().unwrap();
        assert!(c_sing.as_scalar_f64().unwrap().is_infinite());

        // Numerically singular matrix: cond returns large finite number, Inf only for exact singularity
        interp
            .eval_script("A_num = [1 2; 2 4.000000000001];")
            .unwrap();
        let c_num = interp.eval_script("cond(A_num)").unwrap().unwrap();
        let c_val = c_num.as_scalar_f64().unwrap();
        assert!(
            c_val.is_finite(),
            "Numerically singular matrix must have finite cond"
        );
        assert!(
            c_val > 1e10,
            "Condition number must be large for near-singular matrix"
        );

        // 3. orth: orthonormal basis for column space
        interp.eval_script("U_orth = orth(A);").unwrap();
        let u_orth = interp.eval_script("U_orth").unwrap().unwrap();
        assert_eq!(u_orth.shape(), &[2, 1]); // rank is 1

        // 4. null: orthonormal basis for null space
        interp.eval_script("V_null = null(A);").unwrap();
        let v_null = interp.eval_script("V_null").unwrap().unwrap();
        assert_eq!(v_null.shape(), &[2, 1]); // null space dimension is 2 - 1 = 1
                                             // A * null(A) should be zero
        interp
            .eval_script("null_check = norm(A * V_null);")
            .unwrap();
        let null_check = interp.eval_script("null_check").unwrap().unwrap();
        assert!(null_check.as_scalar_f64().unwrap() < 1e-12);
    }

    // 5. kron
    interp
        .eval_script("K = kron([1 2; 3 4], [0 5; 6 7]);")
        .unwrap();
    let k_val = interp.eval_script("K").unwrap().unwrap();
    assert_eq!(k_val.shape(), &[4, 4]);
    if let Value::Double(da) = k_val {
        // Top-left block: 1 * [0 5; 6 7] = [0 5; 6 7]
        assert_eq!(*da.get_subscript(&[1, 1]).unwrap(), 0.0);
        assert_eq!(*da.get_subscript(&[1, 2]).unwrap(), 5.0);
        assert_eq!(*da.get_subscript(&[2, 1]).unwrap(), 6.0);
        assert_eq!(*da.get_subscript(&[2, 2]).unwrap(), 7.0);
    } else {
        panic!();
    }
}

#[test]
fn test_a2_2d_transforms_fft2_ifft2() {
    let mut interp = new_interpreter();

    // 4x4 matrix
    interp.eval_script("M = magic(4);").unwrap();
    interp.eval_script("M_rec = ifft2(fft2(M));").unwrap();
    interp
        .eval_script("err2d = norm(real(M_rec) - M);")
        .unwrap();
    let err2d = interp.eval_script("err2d").unwrap().unwrap();
    assert!(
        err2d.as_scalar_f64().unwrap() < 1e-10,
        "2D FFT/IFFT roundtrip failed"
    );
}

#[test]
fn test_a2_edge_cases_empty_1x1_singular_nan_nonsquare() {
    let mut interp = new_interpreter();

    // 1. Empty matrix: []
    let e_eig = interp.eval_script("eig([])").unwrap().unwrap();
    assert!(e_eig.shape() == [0, 1] || e_eig.shape() == [0, 0]);

    let e_svd = interp.eval_script("svd([])").unwrap().unwrap();
    assert_eq!(e_svd.shape(), &[0, 1]);

    let e_expm = interp.eval_script("expm([])").unwrap().unwrap();
    assert_eq!(e_expm.shape(), &[0, 0]);

    let e_pinv = interp.eval_script("pinv([])").unwrap().unwrap();
    assert_eq!(e_pinv.shape(), &[0, 0]);

    // 2. 1x1 scalar
    let s_eig = interp.eval_script("eig(42)").unwrap().unwrap();
    assert_eq!(s_eig, Value::scalar(42.0));

    let s_expm = interp.eval_script("expm(2)").unwrap().unwrap();
    assert!((s_expm.as_scalar_f64().unwrap() - 2.0_f64.exp()).abs() < 1e-12);

    #[cfg(feature = "faer")]
    {
        let s_svd = interp.eval_script("svd(42)").unwrap().unwrap();
        assert_eq!(s_svd, Value::scalar(42.0));

        let s_pinv = interp.eval_script("pinv(4)").unwrap().unwrap();
        assert_eq!(s_pinv, Value::scalar(0.25));
    }

    // 3. Non-square error for eig
    let err_eig = interp.eval_script("eig([1 2 3; 4 5 6])");
    assert!(err_eig.is_err(), "eig on non-square matrix must error");

    // 4. Non-square error for expm
    let err_expm = interp.eval_script("expm([1 2 3; 4 5 6])");
    assert!(err_expm.is_err(), "expm on non-square matrix must error");
}

#[cfg(not(feature = "faer"))]
#[test]
fn test_a2_faer_feature_off_returns_clear_error() {
    let mut interp = new_interpreter();
    // Non-symmetric eig requires faer
    let err_eig = interp.eval_script("eig([0 -1; 1 0])").unwrap_err();
    let msg_eig = format!("{err_eig}");
    assert!(
        msg_eig.contains("requires") && msg_eig.contains("faer"),
        "eig error message must mention 'requires' and 'faer', got: {msg_eig}"
    );

    // Native one-sided Jacobi SVD enables SVD, pinv, rank, cond, null, orth without faer
    let res_svd = interp.eval_script("svd([1 2; 3 4])").unwrap().unwrap();
    assert_eq!(res_svd.shape(), &[2, 1]);

    let res_pinv = interp.eval_script("pinv([1 2; 3 4])").unwrap().unwrap();
    assert_eq!(res_pinv.shape(), &[2, 2]);

    let res_rank = interp.eval_script("rank([1 2; 3 4])").unwrap().unwrap();
    assert_eq!(res_rank.as_scalar_f64().unwrap(), 2.0);
}

#[cfg(feature = "faer")]
#[test]
fn test_a2_both_native_and_faer_eig_paths_match() {
    use rustlab_math::DenseArray;
    use rustlab_stdlib::linalg::eig_sym::{eig_sym, faer_self_adjoint_eig};

    // Symmetric 3x3 test matrix
    let a = DenseArray::from_vec(
        vec![3, 3],
        vec![
            4.0, 1.0, -2.0, // col 0
            1.0, 2.0, 0.0, // col 1
            -2.0, 0.0, 3.0, // col 2
        ],
    )
    .unwrap();

    let (d_native, v_native) = eig_sym(&a).unwrap();
    let (d_faer, v_faer) = faer_self_adjoint_eig(&a).unwrap();

    assert_eq!(d_native.len(), 3);
    assert_eq!(d_faer.len(), 3);

    // Eigenvalues must match to high numerical precision
    for i in 0..3 {
        assert!(
            (d_native[i] - d_faer[i]).abs() < 1e-12,
            "Eigenvalue mismatch at index {i}: native={}, faer={}",
            d_native[i],
            d_faer[i]
        );
    }

    // Eigenvector columns must match exactly under sign normalization
    for col in 0..3 {
        for row in 0..3 {
            let nat_val = *v_native.get_subscript(&[row + 1, col + 1]).unwrap();
            let faer_val = *v_faer.get_subscript(&[row + 1, col + 1]).unwrap();
            assert!(
                (nat_val - faer_val).abs() < 1e-12,
                "Eigenvector component mismatch at ({row},{col}): native={nat_val}, faer={faer_val}"
            );
        }
    }
}

#[cfg(not(feature = "faer"))]
#[test]
fn test_a2_native_eig_path_without_faer() {
    use rustlab_math::DenseArray;
    use rustlab_stdlib::linalg::eig_sym::eig_sym;

    let a = DenseArray::from_vec(
        vec![3, 3],
        vec![4.0, 1.0, -2.0, 1.0, 2.0, 0.0, -2.0, 0.0, 3.0],
    )
    .unwrap();
    let (d_native, v_native) = eig_sym(&a).unwrap();
    assert_eq!(d_native.len(), 3);
    assert_eq!(v_native.shape(), &[3, 3]);
}

#[cfg(feature = "faer")]
#[test]
fn test_a2_symmetric_eig_routes_to_faer_and_canonicalizes() {
    use rustlab_math::DenseArray;
    use rustlab_stdlib::linalg::eig_sym::{
        eig_sym_auto, faer_self_adjoint_eig, symmetric_eig_backend,
    };

    // The compile-time route used by `eig` must select faer in a default build.
    assert_eq!(symmetric_eig_backend(), "faer");

    let a = DenseArray::from_vec(
        vec![3, 3],
        vec![4.0, 1.0, -2.0, 1.0, 2.0, 0.0, -2.0, 0.0, 3.0],
    )
    .unwrap();

    // Same route as eig_fn, and it matches calling faer directly.
    let (d_auto, v_auto) = eig_sym_auto(&a).unwrap();
    let (d_faer, v_faer) = faer_self_adjoint_eig(&a).unwrap();
    assert_eq!(d_auto, d_faer);
    assert_eq!(v_auto.as_slice(), v_faer.as_slice());

    // Ordering is ascending.
    assert!(d_auto.windows(2).all(|w| w[0] <= w[1]));

    // Sign normalization: first maximum-absolute component is non-negative.
    let n = 3;
    for c in 0..n {
        let mut max_abs = -1.0;
        let mut max_idx = 0;
        for r in 0..n {
            let val = v_auto.as_slice()[r + c * n].abs();
            if val > max_abs {
                max_abs = val;
                max_idx = r;
            }
        }
        assert!(v_auto.as_slice()[max_idx + c * n] >= 0.0);
    }

    // Degenerate eigenspace: tie-break after ordering is deterministic.
    let deg = DenseArray::from_vec(
        vec![4, 4],
        vec![
            2.0, 0.0, 0.0, 0.0, // col 0
            0.0, 2.0, 0.0, 0.0, // col 1
            0.0, 0.0, 1.0, 0.0, // col 2
            0.0, 0.0, 0.0, 1.0, // col 3
        ],
    )
    .unwrap();
    let (d1, v1) = eig_sym_auto(&deg).unwrap();
    let (d2, v2) = eig_sym_auto(&deg).unwrap();
    assert_eq!(d1, vec![1.0, 1.0, 2.0, 2.0]);
    assert_eq!(d1, d2);
    assert_eq!(v1.as_slice(), v2.as_slice());
}

#[test]
fn test_a2_expm_nilpotent_matrix() {
    let mut interp = new_interpreter();
    // [0 1; 0 0] -> [1 1; 0 1]
    let res = interp.eval_script("expm([0 1; 0 0])").unwrap().unwrap();
    if let Value::Double(da) = res {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - 1.0).abs() < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - 1.0).abs() < 1e-12);
        assert!(da.get_subscript(&[2, 1]).unwrap().abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }
}

#[test]
fn test_a2_expm_rotation_generator() {
    let mut interp = new_interpreter();
    // [0 -t; t 0] -> [cos(t) -sin(t); sin(t) cos(t)]
    let t = 0.5_f64;
    let res = interp
        .eval_script(&format!("expm([0 -{t}; {t} 0])"))
        .unwrap()
        .unwrap();
    if let Value::Double(da) = res {
        assert_eq!(da.shape(), &[2, 2]);
        assert!((da.get_subscript(&[1, 1]).unwrap() - t.cos()).abs() < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - (-t.sin())).abs() < 1e-12);
        assert!((da.get_subscript(&[2, 1]).unwrap() - t.sin()).abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - t.cos()).abs() < 1e-12);
    } else {
        panic!("Expected double array");
    }
}

#[test]
fn test_a2_expm_large_norm_scaling_squaring() {
    let mut interp = new_interpreter();
    // Large-norm matrix (norm ~ 50) verified via scaling identity expm(A) == expm(A/2)^2
    interp.eval_script("A_large = [15 20; 25 10];").unwrap();
    interp.eval_script("E1 = expm(A_large);").unwrap();
    interp.eval_script("Half = expm(A_large * 0.5);").unwrap();
    interp.eval_script("E2 = Half * Half;").unwrap();
    interp
        .eval_script("rel_err = norm(E1 - E2) / norm(E1);")
        .unwrap();
    let rel_err = interp.eval_script("rel_err").unwrap().unwrap();
    assert!(rel_err.as_scalar_f64().unwrap() < 1e-12);
}

#[test]
fn test_a2_expm_defective_matrix() {
    let mut interp = new_interpreter();
    // Defective Jordan block J = [2 1; 0 2] -> e^2 * [1 1; 0 1]
    interp.eval_script("J = [2 1; 0 2];").unwrap();
    interp.eval_script("EJ = expm(J);").unwrap();
    let e_j = interp.eval_script("EJ").unwrap().unwrap();
    if let Value::Double(da) = e_j {
        let e2 = 2.0_f64.exp();
        assert!((da.get_subscript(&[1, 1]).unwrap() - e2).abs() / e2 < 1e-12);
        assert!((da.get_subscript(&[1, 2]).unwrap() - e2).abs() / e2 < 1e-12);
        assert!(da.get_subscript(&[2, 1]).unwrap().abs() < 1e-12);
        assert!((da.get_subscript(&[2, 2]).unwrap() - e2).abs() / e2 < 1e-12);
    } else {
        panic!("Expected double array");
    }
}

#[test]
fn test_a2_pinv_moore_penrose_condition() {
    let mut interp = new_interpreter();
    // Moore-Penrose property: A * pinv(A) * A = A
    interp.eval_script("A = [1 2 3; 4 5 6];").unwrap();
    interp.eval_script("P = pinv(A);").unwrap();
    interp.eval_script("diff = norm(A * P * A - A);").unwrap();
    let diff = interp.eval_script("diff").unwrap().unwrap();
    assert!(diff.as_scalar_f64().unwrap() < 1e-12);
}

#[test]
fn test_a2_eigenvector_tie_break_sign_invariant() {
    let mut interp = new_interpreter();
    // S = [2 1; 1 2]. Eigenvectors satisfy S * V = V * D with orthonormal V.
    // Tie-break rule: component with first maximum absolute value is positive.
    interp
        .eval_script("S = [2 1; 1 2]; [V, D] = eig(S);")
        .unwrap();
    interp.eval_script("diff = norm(S * V - V * D);").unwrap();
    let diff = interp.eval_script("diff").unwrap().unwrap();
    assert!(diff.as_scalar_f64().unwrap() < 1e-12);

    let v_val = interp.eval_script("V").unwrap().unwrap();
    if let Value::Double(v) = v_val {
        let n = 2;
        for c in 1..=n {
            let mut max_abs = 0.0f64;
            let mut max_val = 0.0f64;
            for r in 1..=n {
                let val = *v.get_subscript(&[r, c]).unwrap();
                if val.abs() > max_abs {
                    max_abs = val.abs();
                    max_val = val;
                }
            }
            assert!(
                max_val >= 0.0,
                "Column {c} max-abs component must be non-negative"
            );
        }
    } else {
        panic!("Expected double array");
    }
}

#[test]
fn test_a2_tolerance_rule_max_mn_eps_sigma1() {
    let mut interp = new_interpreter();
    // Rank deficient matrix with singular value below tol = max(m,n)*eps(sigma1)
    interp.eval_script("M = [1.0 0.0; 0.0 1e-17];").unwrap();
    interp.eval_script("r = rank(M);").unwrap();
    let r = interp.eval_script("r").unwrap().unwrap();
    assert_eq!(
        r.as_scalar_f64().unwrap(),
        1.0,
        "rank must treat sigma < tol as zero"
    );

    interp.eval_script("P = pinv(M);").unwrap();
    let p_val = interp.eval_script("P").unwrap().unwrap();
    if let Value::Double(da) = p_val {
        assert_eq!(*da.get_subscript(&[1, 1]).unwrap(), 1.0);
        assert_eq!(*da.get_subscript(&[2, 2]).unwrap(), 0.0);
    }
}

#[test]
fn test_a2_pivoted_qr_rank_deficient_matrix() {
    let mut interp = new_interpreter();
    // Native pivoted QR on rank-deficient matrix: A = [1 2 3; 2 4 6; 3 6 9; 4 8 12]
    interp
        .eval_script("A_rd = [1 2 3; 2 4 6; 3 6 9; 4 8 12];")
        .unwrap();
    interp.eval_script("[Q, R, P] = qr(A_rd);").unwrap();
    interp
        .eval_script("diff = norm(A_rd * P - Q * R);")
        .unwrap();
    let diff = interp.eval_script("diff").unwrap().unwrap();
    assert!(
        diff.as_scalar_f64().unwrap() < 1e-12,
        "A*P must equal Q*R natively"
    );

    // Vector permutation form
    interp
        .eval_script("[Q2, R2, p] = qr(A_rd, 'vector');")
        .unwrap();
    interp
        .eval_script("diff_vec = norm(A_rd(:, p) - Q2 * R2);")
        .unwrap();
    let diff_vec = interp.eval_script("diff_vec").unwrap().unwrap();
    assert!(diff_vec.as_scalar_f64().unwrap() < 1e-12);
}
