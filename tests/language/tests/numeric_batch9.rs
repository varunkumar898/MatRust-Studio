use matlab_runtime::run_source;
use matlab_values::DenseMatrix;

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

fn env_mat(res: &matlab_runtime::RunResult, name: &str) -> DenseMatrix {
    res.env
        .get(name)
        .unwrap_or_else(|| panic!("variable '{}' missing", name))
        .as_matrix()
        .unwrap_or_else(|| panic!("variable '{}' is not a matrix", name))
        .clone()
}

// ============================================================================
// Group A: Matrix Left Division and Operator Backslash (\)
// ============================================================================

#[test]
fn test_mldivide_backslash_syntax_and_residual() {
    // Solve square system requiring row swap
    let r = run_source(
        "A = [0, 2; 3, 1]; \
         b = [4; 7]; \
         x = A \\ b; \
         res = A * x;",
    )
    .unwrap();
    let x = env_mat(&r, "x");
    let res = env_mat(&r, "res");
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 1);
    // 0*x1 + 2*x2 = 4 => x2 = 2; 3*x1 + 2 = 7 => x1 = 5/3
    assert!((x.data[0] - 5.0 / 3.0).abs() < 1e-12);
    assert!((x.data[1] - 2.0).abs() < 1e-12);
    assert!(approx_eq(&res.data, &[4.0, 7.0], 1e-12));
}

#[test]
fn test_mldivide_builtin_and_multiple_rhs_columns() {
    let r = run_source(
        "A = [2, 1; 1, 3]; \
         B = [5, 8; 5, 9]; \
         X = mldivide(A, B); \
         AX = A * X;",
    )
    .unwrap();
    let x = env_mat(&r, "X");
    let ax = env_mat(&r, "AX");
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 2);
    // A * X == B
    assert!(approx_eq(&ax.data, &[5.0, 8.0, 5.0, 9.0], 1e-12));
}

#[test]
fn test_mldivide_scalar_broadcasting() {
    let r = run_source("s = 4; B = [8, 12; 16, 20]; X = s \\ B;").unwrap();
    let x = env_mat(&r, "X");
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 2);
    assert_eq!(x.data, vec![2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn test_mldivide_overdetermined_least_squares() {
    // 3 x 2 overdetermined system: A = [1 1; 1 2; 1 3], b = [2; 3; 4]
    // Line y = 1 + x fits exactly: x1 = 1, x2 = 1
    let r = run_source(
        "A = [1, 1; 1, 2; 1, 3]; \
         b = [2; 3; 4]; \
         x = A \\ b;",
    )
    .unwrap();
    let x = env_mat(&r, "x");
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 1);
    assert!(approx_eq(&x.data, &[1.0, 1.0], 1e-12));
}

#[test]
fn test_elementwise_ldivide() {
    let r = run_source(
        "A = [2, 4; 5, 10]; \
         B = [6, 16; 25, 50]; \
         C1 = A .\\ B; \
         C2 = ldivide(A, B);",
    )
    .unwrap();
    let c1 = env_mat(&r, "C1");
    let c2 = env_mat(&r, "C2");
    assert_eq!(c1.data, vec![3.0, 4.0, 5.0, 5.0]);
    assert_eq!(c2.data, vec![3.0, 4.0, 5.0, 5.0]);
}

#[test]
fn test_mldivide_singular_and_dimension_errors() {
    let err_sing = run_source("A = [1, 2; 2, 4]; b = [1; 2]; x = A \\ b;").unwrap_err();
    assert!(format!("{}", err_sing).contains("singular"));

    let err_dim = run_source("A = [1, 2; 3, 4]; b = [1; 2; 3]; x = A \\ b;").unwrap_err();
    assert!(format!("{}", err_dim).contains("Matrix dimensions must agree"));
}

// ============================================================================
// Group B: LU Factorization
// ============================================================================

#[test]
fn test_lu_three_outputs_partial_pivoting() {
    // Matrix where row pivoting is strictly required: A(1,1) is 0
    let r = run_source(
        "A = [0, 2, 1; 3, 1, 2; 1, -1, 0]; \
         [L, U, P] = lu(A); \
         PA = P * A; \
         LU = L * U;",
    )
    .unwrap();
    let l = env_mat(&r, "L");
    let u = env_mat(&r, "U");
    let p = env_mat(&r, "P");
    let pa = env_mat(&r, "PA");
    let lu = env_mat(&r, "LU");

    assert_eq!(l.rows, 3);
    assert_eq!(l.cols, 3);
    assert_eq!(u.rows, 3);
    assert_eq!(u.cols, 3);
    assert_eq!(p.rows, 3);
    assert_eq!(p.cols, 3);

    // L must be unit lower triangular: diag is 1, above diag is 0
    assert_eq!(l.data[0], 1.0);
    assert_eq!(l.data[1], 0.0);
    assert_eq!(l.data[2], 0.0);
    assert_eq!(l.data[4], 1.0);
    assert_eq!(l.data[5], 0.0);
    assert_eq!(l.data[8], 1.0);

    // U must be upper triangular: below diag is 0
    assert_eq!(u.data[3], 0.0);
    assert_eq!(u.data[6], 0.0);
    assert_eq!(u.data[7], 0.0);

    // Identity P * A = L * U
    assert!(approx_eq(&pa.data, &lu.data, 1e-12));
}

#[test]
fn test_lu_two_outputs_permutation_in_l() {
    let r = run_source(
        "A = [1, 4, 7; 2, 5, 8; 3, 6, 10]; \
         [L, U] = lu(A); \
         LU = L * U;",
    )
    .unwrap();
    let a = env_mat(&r, "A");
    let lu = env_mat(&r, "LU");
    assert!(approx_eq(&a.data, &lu.data, 1e-12));
}

#[test]
fn test_lu_single_output() {
    let r = run_source("A = [4, 3; 6, 3]; Y = lu(A);").unwrap();
    let y = env_mat(&r, "Y");
    assert_eq!(y.rows, 2);
    assert_eq!(y.cols, 2);
}

// ============================================================================
// Group C: Determinant, Inverse, and Trace
// ============================================================================

#[test]
fn test_det_identity_triangular_and_permutation_sign() {
    let r = run_source(
        "d_eye = det(eye(4)); \
         d_tri = det([2, 5, 1; 0, 3, 4; 0, 0, 5]); \
         d_swap = det([0, 1; 1, 0]); \
         d_sing = det([1, 2, 3; 4, 5, 6; 7, 8, 9]); \
         d_empty = det([]);",
    )
    .unwrap();
    assert_eq!(env_mat(&r, "d_eye").data[0], 1.0);
    assert_eq!(env_mat(&r, "d_tri").data[0], 30.0);
    assert_eq!(env_mat(&r, "d_swap").data[0], -1.0);
    assert!(env_mat(&r, "d_sing").data[0].abs() < 1e-12);
    assert_eq!(env_mat(&r, "d_empty").data[0], 1.0);
}

#[test]
fn test_inv_orthogonality_and_singular_error() {
    let r = run_source(
        "A = [4, 7; 2, 6]; \
         A_inv = inv(A); \
         I1 = A * A_inv; \
         I2 = A_inv * A;",
    )
    .unwrap();
    let i1 = env_mat(&r, "I1");
    let i2 = env_mat(&r, "I2");
    assert!(approx_eq(&i1.data, &[1.0, 0.0, 0.0, 1.0], 1e-12));
    assert!(approx_eq(&i2.data, &[1.0, 0.0, 0.0, 1.0], 1e-12));

    // Singular matrix must produce diagnostic and not pseudoinverse
    let err_sing = run_source("inv([1, 2; 2, 4]);").unwrap_err();
    assert!(format!("{}", err_sing).contains("singular"));

    // Nonsquare matrix must error
    let err_nonsq = run_source("inv([1, 2, 3; 4, 5, 6]);").unwrap_err();
    assert!(format!("{}", err_nonsq).contains("square"));
}

#[test]
fn test_trace_properties() {
    let r = run_source(
        "t1 = trace([1, 2, 3; 4, 5, 6; 7, 8, 9]); \
         t_eye = trace(eye(5)); \
         t_rect = trace([1, 2, 3; 4, 5, 6]); \
         t_empty = trace([]);",
    )
    .unwrap();
    assert_eq!(env_mat(&r, "t1").data[0], 15.0);
    assert_eq!(env_mat(&r, "t_eye").data[0], 5.0);
    assert_eq!(env_mat(&r, "t_rect").data[0], 6.0);
    assert_eq!(env_mat(&r, "t_empty").data[0], 0.0);
}

// ============================================================================
// Group D: SVD, Rank, and Pseudoinverse
// ============================================================================

#[test]
fn test_svd_single_and_three_outputs_reconstruction() {
    let r = run_source(
        "A = [1, 2; 3, 4; 5, 6]; \
         s = svd(A); \
         [U, S, V] = svd(A); \
         A_rec = U * S * V'; \
         UtU = U' * U; \
         VtV = V' * V;",
    )
    .unwrap();
    let s = env_mat(&r, "s");
    let u = env_mat(&r, "U");
    let smat = env_mat(&r, "S");
    let v = env_mat(&r, "V");
    let a_rec = env_mat(&r, "A_rec");
    let utu = env_mat(&r, "UtU");
    let vtv = env_mat(&r, "VtV");

    assert_eq!(s.rows, 2);
    assert_eq!(s.cols, 1);
    assert!(s.data[0] >= s.data[1]);
    assert!(s.data[1] >= 0.0);

    assert_eq!(u.rows, 3);
    assert_eq!(u.cols, 3);
    assert_eq!(smat.rows, 3);
    assert_eq!(smat.cols, 2);
    assert_eq!(v.rows, 2);
    assert_eq!(v.cols, 2);

    // Reconstruction A == U * S * V'
    assert!(approx_eq(
        &a_rec.data,
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        1e-12
    ));

    // Orthogonality: U' * U == eye(3), V' * V == eye(2)
    assert!(approx_eq(
        &utu.data,
        &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        1e-12
    ));
    assert!(approx_eq(&vtv.data, &[1.0, 0.0, 0.0, 1.0], 1e-12));
}

#[test]
fn test_rank_full_and_deficient() {
    let r = run_source(
        "r_full = rank(eye(4)); \
         r_def = rank([1, 2; 2, 4]); \
         r_tol = rank([1, 0; 0, 1e-12], 1e-6); \
         r_zero = rank(zeros(3, 3));",
    )
    .unwrap();
    assert_eq!(env_mat(&r, "r_full").data[0], 4.0);
    assert_eq!(env_mat(&r, "r_def").data[0], 1.0);
    assert_eq!(env_mat(&r, "r_tol").data[0], 1.0);
    assert_eq!(env_mat(&r, "r_zero").data[0], 0.0);
}

#[test]
fn test_pinv_moore_penrose_identities() {
    // 3 x 2 matrix
    let r = run_source(
        "A = [1, 2; 3, 4; 5, 6]; \
         P = pinv(A); \
         id1 = A * P * A; \
         id2 = P * A * P; \
         id3_diff = (A * P)' - (A * P); \
         id4_diff = (P * A)' - (P * A);",
    )
    .unwrap();
    let a = env_mat(&r, "A");
    let p = env_mat(&r, "P");
    let id1 = env_mat(&r, "id1");
    let id2 = env_mat(&r, "id2");
    let id3_diff = env_mat(&r, "id3_diff");
    let id4_diff = env_mat(&r, "id4_diff");

    assert_eq!(p.rows, 2);
    assert_eq!(p.cols, 3);

    // 1. A * P * A ≈ A
    assert!(approx_eq(&id1.data, &a.data, 1e-12));
    // 2. P * A * P ≈ P
    assert!(approx_eq(&id2.data, &p.data, 1e-12));
    // 3. (A * P)' == A * P
    for &val in &id3_diff.data {
        assert!(val.abs() < 1e-12);
    }
    // 4. (P * A)' == P * A
    for &val in &id4_diff.data {
        assert!(val.abs() < 1e-12);
    }
}

// ============================================================================
// Group E: Norms (Vector and Matrix)
// ============================================================================

#[test]
fn test_norm_vectors() {
    let r = run_source(
        "v = [3, -4]; \
         n1 = norm(v, 1); \
         n2 = norm(v); \
         n_inf = norm(v, Inf); \
         n_neg_inf = norm(v, -Inf); \
         n_p = norm(v, 3);",
    )
    .unwrap();
    assert_eq!(env_mat(&r, "n1").data[0], 7.0);
    assert_eq!(env_mat(&r, "n2").data[0], 5.0);
    assert_eq!(env_mat(&r, "n_inf").data[0], 4.0);
    assert_eq!(env_mat(&r, "n_neg_inf").data[0], 3.0);
    let expected_p3 = (27.0f64 + 64.0).powf(1.0 / 3.0);
    assert!((env_mat(&r, "n_p").data[0] - expected_p3).abs() < 1e-12);
}

#[test]
fn test_norm_matrices() {
    let r = run_source(
        "M = [1, 2; 3, 4]; \
         n1 = norm(M, 1); \
         n2 = norm(M); \
         n_inf = norm(M, Inf); \
         n_fro = norm(M, 'fro');",
    )
    .unwrap();
    // 1-norm: max column sum = max(1+3, 2+4) = 6
    assert_eq!(env_mat(&r, "n1").data[0], 6.0);
    // Inf-norm: max row sum = max(1+2, 3+4) = 7
    assert_eq!(env_mat(&r, "n_inf").data[0], 7.0);
    // Fro-norm: sqrt(1 + 4 + 9 + 16) = sqrt(30)
    assert!((env_mat(&r, "n_fro").data[0] - 30.0f64.sqrt()).abs() < 1e-12);
    // 2-norm: largest singular value of [1 2; 3 4]
    let s_r = run_source("s = svd([1, 2; 3, 4]);").unwrap();
    let s_max = env_mat(&s_r, "s").data[0];
    assert!((env_mat(&r, "n2").data[0] - s_max).abs() < 1e-12);
}

#[test]
fn test_norm_matrix_invalid_p_error() {
    let err = run_source("norm([1, 2; 3, 4], 3);").unwrap_err();
    assert!(format!("{}", err).contains("valid matrix norms are 1, 2, inf, and 'fro'"));
}
