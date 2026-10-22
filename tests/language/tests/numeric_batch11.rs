use matlab_runtime::{run_source, Session};
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix, Value};

#[allow(dead_code)]
fn approx_eq(a: &[f64], b: &[f64], tol: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (&x, &y) in a.iter().zip(b.iter()) {
        if (x - y).abs() > tol {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn approx_eq_complex(a: &[Complex64], b: &[Complex64], tol: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (&x, &y) in a.iter().zip(b.iter()) {
        if (x.re - y.re).abs() > tol || (x.im - y.im).abs() > tol {
            return false;
        }
    }
    true
}

fn env_matrix(res: &matlab_runtime::RunResult, name: &str) -> DenseMatrix {
    let val = res
        .env
        .get(name)
        .unwrap_or_else(|| panic!("variable '{}' missing", name));
    val.as_matrix()
        .unwrap_or_else(|| panic!("variable '{}' is not a real matrix: {:?}", name, val))
        .clone()
}

fn env_complex(res: &matlab_runtime::RunResult, name: &str) -> ComplexDenseMatrix {
    let val = res
        .env
        .get(name)
        .unwrap_or_else(|| panic!("variable '{}' missing", name));
    val.as_complex_matrix()
        .unwrap_or_else(|| panic!("variable '{}' is not a complex matrix: {:?}", name, val))
        .clone()
}

fn env_val(res: &matlab_runtime::RunResult, name: &str) -> Value {
    res.env
        .get(name)
        .unwrap_or_else(|| panic!("variable '{}' missing", name))
        .clone()
}

// ============================================================================
// Phase 1: Complex Storage Semantics, isreal, and Arithmetic Cancellation
// ============================================================================

#[test]
fn test_complex_storage_semantics_and_isreal() {
    let r = run_source(
        "z_scalar = complex(12); \
         z_vec = complex([1, 2]); \
         r_scalar = real(complex(12)); \
         r_vec = real([1, 2]); \
         im_val = imag(complex(12)); \
         c_scalar = conj(complex(12)); \
         c_real = conj([1, 2]); \
         tp_c = (complex(12)).'; \
         ctp_c = (complex(12))'; \
         is_z_scalar = isreal(z_scalar); \
         is_z_vec = isreal(z_vec); \
         is_r_scalar = isreal(r_scalar); \
         is_r_vec = isreal(r_vec); \
         is_im_val = isreal(im_val); \
         is_c_scalar = isreal(c_scalar); \
         is_c_real = isreal(c_real); \
         is_tp_c = isreal(tp_c); \
         is_ctp_c = isreal(ctp_c);",
    )
    .unwrap();

    // Explicit complex constructor maintains complex storage even with zero imaginary part
    assert_eq!(r.env.get("is_z_scalar").unwrap(), &Value::Boolean(false));
    assert_eq!(r.env.get("is_z_vec").unwrap(), &Value::Boolean(false));

    // real() and imag() produce real-stored values
    assert_eq!(r.env.get("is_r_scalar").unwrap(), &Value::Boolean(true));
    assert_eq!(r.env.get("is_r_vec").unwrap(), &Value::Boolean(true));
    assert_eq!(r.env.get("is_im_val").unwrap(), &Value::Boolean(true));

    // conj() preserves input storage class
    assert_eq!(r.env.get("is_c_scalar").unwrap(), &Value::Boolean(false));
    assert_eq!(r.env.get("is_c_real").unwrap(), &Value::Boolean(true));

    // Transpose operations preserve storage class
    assert_eq!(r.env.get("is_tp_c").unwrap(), &Value::Boolean(false));
    assert_eq!(r.env.get("is_ctp_c").unwrap(), &Value::Boolean(false));
}

#[test]
fn test_arithmetic_cancellation_and_promotion() {
    let r = run_source(
        "x = 3 + 4i; \
         y = 5 - 4i; \
         r_add = x + y; \
         is_add = isreal(r_add); \
         \
         y_sub = 1 + 4i; \
         r_sub = x - y_sub; \
         is_sub = isreal(r_sub); \
         \
         c1 = complex(2, 0); \
         c2 = complex(3, 0); \
         r_mul = c1 * c2; \
         is_mul = isreal(r_mul); \
         \
         M1 = [1+2i, 3-4i]; \
         M2 = [2-2i, 1+4i]; \
         S_mat = M1 + M2; \
         is_mat = isreal(S_mat); \
         \
         r_exact = (3 + 4i) + (5 - 4i); \
         r_residual = (3 + 4i) + (5 - 4.0000001i); \
         is_exact = isreal(r_exact); \
         is_residual = isreal(r_residual);",
    )
    .unwrap();

    // Adding conjugate values cancels imaginary parts to zero, yielding real storage
    assert_eq!(r.env.get("is_add").unwrap(), &Value::Boolean(true));
    assert_eq!(env_matrix(&r, "r_add").data[0], 8.0);

    // Subtracting equal imaginary parts produces real storage
    assert_eq!(r.env.get("is_sub").unwrap(), &Value::Boolean(true));
    assert_eq!(env_matrix(&r, "r_sub").data[0], 2.0);

    // Multiplying complex values with zero imaginary parts produces real storage
    assert_eq!(r.env.get("is_mul").unwrap(), &Value::Boolean(true));
    assert_eq!(env_matrix(&r, "r_mul").data[0], 6.0);

    // Array cancellation
    assert_eq!(r.env.get("is_mat").unwrap(), &Value::Boolean(true));
    assert_eq!(&env_matrix(&r, "S_mat").data, &[3.0, 4.0]);

    // Exact cancellation vs non-zero residual
    assert_eq!(r.env.get("is_exact").unwrap(), &Value::Boolean(true));
    assert_eq!(r.env.get("is_residual").unwrap(), &Value::Boolean(false));
}

#[test]
fn test_complex_indexing_and_session_persistence() {
    let mut session = Session::new();
    session.exec("A = complex([1, 2; 3, 4]);", None).unwrap();
    let r1 = session.exec("is_A1 = isreal(A);", None).unwrap();
    assert_eq!(r1.env.get("is_A1").unwrap(), &Value::Boolean(false));

    // Assigning a real value to an element of a complex matrix preserves complex storage
    session.exec("A(1, 1) = 10;", None).unwrap();
    let r2 = session.exec("is_A2 = isreal(A);", None).unwrap();
    assert_eq!(r2.env.get("is_A2").unwrap(), &Value::Boolean(false));

    // Assigning a complex value to a real matrix promotes it to complex
    session.exec("B = [10, 20]; B(2) = 5 + 6i;", None).unwrap();
    let r3 = session.exec("is_B = isreal(B);", None).unwrap();
    assert_eq!(r3.env.get("is_B").unwrap(), &Value::Boolean(false));
}

#[test]
fn test_complex_division_stability_and_specials() {
    let r = run_source(
        "z1 = 1e200 + 1e200*i; \
         z2 = 1e200 + 1e200*i; \
         q_large = z1 / z2; \
         \
         s1 = 1e-200 + 1e-200*i; \
         s2 = 1e-200 + 1e-200*i; \
         q_small = s1 / s2; \
         \
         w = 1 + 2i; \
         div_zero = w / 0;",
    )
    .unwrap();

    let ql = env_matrix(&r, "q_large");
    assert!((ql.data[0] - 1.0).abs() < 1e-12);

    let qs = env_matrix(&r, "q_small");
    assert!((qs.data[0] - 1.0).abs() < 1e-12);

    let dz = env_complex(&r, "div_zero");
    assert!(dz.data[0].re.is_infinite());
    assert!(dz.data[0].im.is_infinite());
}

// ============================================================================
// Phase 3.A: Complex SVD
// ============================================================================

#[test]
fn test_complex_svd_square_reconstruction() {
    let r = run_source(
        "A = [1+1i, 2; -1i, 3-2i]; \
         s = svd(A); \
         [U, S, V] = svd(A); \
         recon = U * S * V'; \
         diff = norm(recon - A, 'fro'); \
         u_orth = norm(U' * U - eye(2), 'fro'); \
         v_orth = norm(V' * V - eye(2), 'fro');",
    )
    .unwrap();

    // Singular values sorted non-negative
    let s_vec = env_matrix(&r, "s");
    assert_eq!(s_vec.rows, 2);
    assert_eq!(s_vec.cols, 1);
    assert!(s_vec.data[0] >= s_vec.data[1]);
    assert!(s_vec.data[1] >= 0.0);

    // Unitarity of U and V
    let u_orth = env_matrix(&r, "u_orth").data[0];
    let v_orth = env_matrix(&r, "v_orth").data[0];
    assert!(u_orth < 1e-12, "U' * U must be identity: residual={}", u_orth);
    assert!(v_orth < 1e-12, "V' * V must be identity: residual={}", v_orth);

    // Reconstruction A = U * S * V'
    let diff = env_matrix(&r, "diff").data[0];
    assert!(diff < 1e-12, "U * S * V' must equal A: residual={}", diff);
}

#[test]
fn test_complex_svd_rectangular_matrices() {
    // Overdetermined (3 x 2)
    let r1 = run_source(
        "A = [1, 1i; 2-1i, 3; 0, 1+1i]; \
         [U, S, V] = svd(A); \
         diff = norm(U * S * V' - A, 'fro'); \
         u_orth = norm(U' * U - eye(3), 'fro'); \
         v_orth = norm(V' * V - eye(2), 'fro');",
    )
    .unwrap();

    let diff1 = env_matrix(&r1, "diff").data[0];
    let u_orth1 = env_matrix(&r1, "u_orth").data[0];
    let v_orth1 = env_matrix(&r1, "v_orth").data[0];
    assert!(diff1 < 1e-12, "3x2 reconstruction residual: {}", diff1);
    assert!(u_orth1 < 1e-12, "3x3 U unitarity residual: {}", u_orth1);
    assert!(v_orth1 < 1e-12, "2x2 V unitarity residual: {}", v_orth1);

    // Underdetermined (2 x 3)
    let r2 = run_source(
        "A = [1+1i, 2, -1i; 3, 1-1i, 4]; \
         [U, S, V] = svd(A); \
         diff = norm(U * S * V' - A, 'fro'); \
         u_orth = norm(U' * U - eye(2), 'fro'); \
         v_orth = norm(V' * V - eye(3), 'fro');",
    )
    .unwrap();

    let diff2 = env_matrix(&r2, "diff").data[0];
    let u_orth2 = env_matrix(&r2, "u_orth").data[0];
    let v_orth2 = env_matrix(&r2, "v_orth").data[0];
    assert!(diff2 < 1e-12, "2x3 reconstruction residual: {}", diff2);
    assert!(u_orth2 < 1e-12, "2x2 U unitarity residual: {}", u_orth2);
    assert!(v_orth2 < 1e-12, "3x3 V unitarity residual: {}", v_orth2);
}

#[test]
fn test_complex_svd_rank_deficient_and_zero() {
    // Rank-deficient 2x2 matrix
    let r = run_source(
        "A = [1+1i, 2+2i; 2-1i, 4-2i]; \
         [U, S, V] = svd(A); \
         diff = norm(U * S * V' - A, 'fro'); \
         u_orth = norm(U' * U - eye(2), 'fro'); \
         v_orth = norm(V' * V - eye(2), 'fro'); \
         r_def = rank(A);",
    )
    .unwrap();

    let diff = env_matrix(&r, "diff").data[0];
    assert!(diff < 1e-12, "Rank-deficient reconstruction: {}", diff);
    assert!(env_matrix(&r, "u_orth").data[0] < 1e-12);
    assert!(env_matrix(&r, "v_orth").data[0] < 1e-12);
    assert_eq!(env_matrix(&r, "r_def").data[0], 1.0);

    // Zero matrix
    let r_zero = run_source(
        "Z = complex(zeros(2, 2)); \
         [U, S, V] = svd(Z); \
         diff = norm(U * S * V' - Z, 'fro'); \
         r_z = rank(Z);",
    )
    .unwrap();
    assert!(env_matrix(&r_zero, "diff").data[0] < 1e-14);
    assert_eq!(env_matrix(&r_zero, "r_z").data[0], 0.0);
}

// ============================================================================
// Phase 3.B: Complex Rank and Moore-Penrose Pseudoinverse
// ============================================================================

#[test]
fn test_complex_rank_full_and_deficient() {
    let r = run_source(
        "A_full = [1+1i, 2; 3, 4-1i]; \
         r_full = rank(A_full); \
         \
         A_def = [1+1i, 2+2i; 3-3i, 6-6i]; \
         r_def = rank(A_def); \
         \
         r_tol = rank(A_def, 1e-1);",
    )
    .unwrap();

    assert_eq!(env_matrix(&r, "r_full").data[0], 2.0);
    assert_eq!(env_matrix(&r, "r_def").data[0], 1.0);
}

#[test]
fn test_complex_pinv_moore_penrose_conditions() {
    // 3 x 2 complex matrix
    let r = run_source(
        "A = [1+1i, 2; 3, -1i; 2-1i, 1+2i]; \
         P = pinv(A); \
         cond1 = norm(A * P * A - A, 'fro'); \
         cond2 = norm(P * A * P - P, 'fro'); \
         cond3 = norm((A * P)' - A * P, 'fro'); \
         cond4 = norm((P * A)' - P * A, 'fro');",
    )
    .unwrap();

    let c1 = env_matrix(&r, "cond1").data[0];
    let c2 = env_matrix(&r, "cond2").data[0];
    let c3 = env_matrix(&r, "cond3").data[0];
    let c4 = env_matrix(&r, "cond4").data[0];

    assert!(c1 < 1e-10, "Moore-Penrose 1 (A*P*A ≈ A) failed: {}", c1);
    assert!(c2 < 1e-10, "Moore-Penrose 2 (P*A*P ≈ P) failed: {}", c2);
    assert!(c3 < 1e-10, "Moore-Penrose 3 ((A*P)' ≈ A*P) failed: {}", c3);
    assert!(c4 < 1e-10, "Moore-Penrose 4 ((P*A)' ≈ P*A) failed: {}", c4);
}

// ============================================================================
// Phase 3.C: Complex Matrix Division and Solving
// ============================================================================

#[test]
fn test_complex_mldivide_square_system() {
    let r = run_source(
        "A = [2+1i, 1; 1-1i, 3]; \
         B = [1+2i; 4-1i]; \
         X = A \\ B; \
         res = norm(A * X - B, 'fro'); \
         \
         B_multi = [1+2i, 3; 4-1i, 2i]; \
         X_multi = mldivide(A, B_multi); \
         res_multi = norm(A * X_multi - B_multi, 'fro');",
    )
    .unwrap();

    let res = env_matrix(&r, "res").data[0];
    assert!(res < 1e-12, "Complex square solve residual: {}", res);

    let res_multi = env_matrix(&r, "res_multi").data[0];
    assert!(
        res_multi < 1e-12,
        "Complex multiple-RHS solve residual: {}",
        res_multi
    );
}

#[test]
fn test_complex_mldivide_least_squares_overdetermined() {
    let r = run_source(
        "A = [1+1i, 2; 3, -1i; 2-1i, 1+2i]; \
         B = [1; 2i; 3-1i]; \
         X = A \\ B; \
         normal_res = norm(A' * (A * X - B), 'fro');",
    )
    .unwrap();

    let normal_res = env_matrix(&r, "normal_res").data[0];
    assert!(
        normal_res < 1e-10,
        "Complex QR least squares normal equations residual: {}",
        normal_res
    );
}

#[test]
fn test_complex_mrdivide() {
    let r = run_source(
        "A = [2+1i, 1; 1-1i, 3]; \
         B = [3-1i, 4+2i]; \
         X = B / A; \
         res = norm(X * A - B, 'fro');",
    )
    .unwrap();

    let res = env_matrix(&r, "res").data[0];
    assert!(res < 1e-12, "Complex right division residual: {}", res);
}

// ============================================================================
// Phase 3.D: Complex Determinant, Inverse, Trace, Norms, LU
// ============================================================================

#[test]
fn test_complex_determinant_and_inverse() {
    let r = run_source(
        "A = [1+1i, 2; 3, 4-1i]; \
         d = det(A); \
         invA = inv(A); \
         res_inv = norm(A * invA - eye(2), 'fro');",
    )
    .unwrap();

    // det([1+i, 2; 3, 4-i]) = (1+i)(4-i) - 6 = (5 + 3i) - 6 = -1 + 3i
    let d_val = env_val(&r, "d");
    match d_val {
        Value::ComplexMatrix(c) => {
            assert!((c.data[0].re - (-1.0)).abs() < 1e-12);
            assert!((c.data[0].im - 3.0).abs() < 1e-12);
        }
        _ => panic!("Expected complex determinant, got {:?}", d_val),
    }

    let res_inv = env_matrix(&r, "res_inv").data[0];
    assert!(res_inv < 1e-12, "inv(A) residual: {}", res_inv);
}

#[test]
fn test_complex_trace_and_norms() {
    let r = run_source(
        "A = [1+2i, 3; 4, 5-2i]; \
         tr = trace(A); \
         \
         v = [3+4i, 5-12i]; \
         n1 = norm(v, 1); \
         n2 = norm(v, 2); \
         n_inf = norm(v, inf); \
         \
         M = [1+1i, 2; 3, 4-1i]; \
         mn1 = norm(M, 1); \
         mn2 = norm(M, 2); \
         mn_inf = norm(M, inf); \
         mnf = norm(M, 'fro');",
    )
    .unwrap();

    // Trace cancellation: (1 + 2i) + (5 - 2i) = 6 (real)
    let tr = env_matrix(&r, "tr").data[0];
    assert!((tr - 6.0).abs() < 1e-12);

    // Vector norms: |3+4i| = 5, |5-12i| = 13
    let n1 = env_matrix(&r, "n1").data[0];
    let n2 = env_matrix(&r, "n2").data[0];
    let n_inf = env_matrix(&r, "n_inf").data[0];
    assert!((n1 - 18.0).abs() < 1e-12);
    assert!((n2 - (194.0f64).sqrt()).abs() < 1e-12);
    assert!((n_inf - 13.0).abs() < 1e-12);

    // Matrix norms
    let mn1 = env_matrix(&r, "mn1").data[0];
    let mn_inf = env_matrix(&r, "mn_inf").data[0];
    let mnf = env_matrix(&r, "mnf").data[0];
    // Column 1: |1+i| + 3 = sqrt(2) + 3 ≈ 4.4142. Column 2: 2 + |4-i| = 2 + sqrt(17) ≈ 6.1231
    assert!((mn1 - (2.0 + (17.0f64).sqrt())).abs() < 1e-12);
    // Row 1: |1+i| + 2 = sqrt(2) + 2 ≈ 3.4142. Row 2: 3 + |4-i| = 3 + sqrt(17) ≈ 7.1231
    assert!((mn_inf - (3.0 + (17.0f64).sqrt())).abs() < 1e-12);
    // Frobenius: sqrt(2 + 4 + 9 + 17) = sqrt(32)
    assert!((mnf - (32.0f64).sqrt()).abs() < 1e-12);
    // 2-norm equals largest singular value
    let mn2 = env_matrix(&r, "mn2").data[0];
    assert!(mn2 > 0.0);
}

#[test]
fn test_complex_lu_forms() {
    let r = run_source(
        "A = [1+1i, 2; 3, 4-1i]; \
         Y = lu(A); \
         [L2, U2] = lu(A); \
         [L3, U3, P3] = lu(A); \
         res2 = norm(L2 * U2 - A, 'fro'); \
         res3 = norm(P3 * A - L3 * U3, 'fro');",
    )
    .unwrap();

    let res2 = env_matrix(&r, "res2").data[0];
    let res3 = env_matrix(&r, "res3").data[0];
    assert!(res2 < 1e-12, "2-output LU A = L*U residual: {}", res2);
    assert!(res3 < 1e-12, "3-output LU P*A = L*U residual: {}", res3);
}

// ============================================================================
// Phase 4: Regression Tests (Batches 1-10 Functionality)
// ============================================================================

#[test]
fn test_regression_real_linear_algebra_preserved() {
    let r = run_source(
        "A = [4, 1; 2, 3]; \
         B = [1; 2]; \
         X = A \\ B; \
         d = det(A); \
         tr = trace(A); \
         rk = rank(A); \
         P = pinv(A); \
         n = norm(A, 1); \
         [U, S, V] = svd(A); \
         recon = norm(U * S * V' - A, 'fro');",
    )
    .unwrap();

    // det([4 1; 2 3]) = 12 - 2 = 10
    assert!((env_matrix(&r, "d").data[0] - 10.0).abs() < 1e-12);
    // trace([4 1; 2 3]) = 7
    assert!((env_matrix(&r, "tr").data[0] - 7.0).abs() < 1e-12);
    // rank = 2
    assert_eq!(env_matrix(&r, "rk").data[0], 2.0);
    // norm(A, 1) = max(4+2, 1+3) = 6
    assert!((env_matrix(&r, "n").data[0] - 6.0).abs() < 1e-12);
    // SVD reconstruction
    assert!(env_matrix(&r, "recon").data[0] < 1e-12);
    // A * X = B
    let x = env_matrix(&r, "X");
    assert!((4.0 * x.data[0] + 1.0 * x.data[1] - 1.0).abs() < 1e-12);
    assert!((2.0 * x.data[0] + 3.0 * x.data[1] - 2.0).abs() < 1e-12);
}
