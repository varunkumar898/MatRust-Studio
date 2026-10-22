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

fn env_complex(res: &matlab_runtime::RunResult, name: &str) -> ComplexDenseMatrix {
    let val = res.env.get(name).unwrap_or_else(|| panic!("variable '{}' missing", name));
    val.as_complex_matrix()
        .unwrap_or_else(|| panic!("variable '{}' is not a complex matrix, value is {:?}", name, val))
        .clone()
}

fn env_mat(res: &matlab_runtime::RunResult, name: &str) -> DenseMatrix {
    let val = res.env.get(name).unwrap_or_else(|| panic!("variable '{}' missing", name));
    val.as_matrix()
        .unwrap_or_else(|| panic!("variable '{}' is not a real matrix, value is {:?}", name, val))
        .clone()
}

// ============================================================================
// Group A: Scalar Complex Construction, Constants, and Literals
// ============================================================================

#[test]
fn test_complex_constructor_scalar() {
    let r = run_source("z = complex(3, 4);").unwrap();
    let z = env_complex(&r, "z");
    assert_eq!(z.rows, 1);
    assert_eq!(z.cols, 1);
    assert_eq!(z.data[0], Complex64::new(3.0, 4.0));
}

#[test]
fn test_complex_literals_i_and_j() {
    let r = run_source("z1 = 3 + 4i; z2 = 2 - 5j; z3 = 2.5i;").unwrap();
    let z1 = env_complex(&r, "z1");
    let z2 = env_complex(&r, "z2");
    let z3 = env_complex(&r, "z3");
    assert_eq!(z1.data[0], Complex64::new(3.0, 4.0));
    assert_eq!(z2.data[0], Complex64::new(2.0, -5.0));
    assert_eq!(z3.data[0], Complex64::new(0.0, 2.5));
}

#[test]
fn test_complex_constants_i_and_j() {
    let r = run_source("c1 = i; c2 = j;").unwrap();
    let c1 = env_complex(&r, "c1");
    let c2 = env_complex(&r, "c2");
    assert_eq!(c1.data[0], Complex64::new(0.0, 1.0));
    assert_eq!(c2.data[0], Complex64::new(0.0, 1.0));
}

#[test]
fn test_complex_shadowing_i_and_j() {
    // In MATLAB, user variables shadow the built-in i/j constants
    let r = run_source("i = 10; y = i + 2;").unwrap();
    let y = env_mat(&r, "y");
    assert_eq!(y.data[0], 12.0);
}

// ============================================================================
// Group B: Built-ins: real, imag, conj, abs, angle, isreal
// ============================================================================

#[test]
fn test_component_extraction_and_properties() {
    let r = run_source(
        "z = 3 + 4i; \
         r = real(z); \
         im = imag(z); \
         c = conj(z); \
         m = abs(z); \
         phi = angle(z); \
         zr = real(5); \
         zi = imag(5);",
    )
    .unwrap();

    let r_val = env_mat(&r, "r");
    let im_val = env_mat(&r, "im");
    let c_val = env_complex(&r, "c");
    let m_val = env_mat(&r, "m");
    let phi_val = env_mat(&r, "phi");
    let zr = env_mat(&r, "zr");
    let zi = env_mat(&r, "zi");

    assert_eq!(r_val.data[0], 3.0);
    assert_eq!(im_val.data[0], 4.0);
    assert_eq!(c_val.data[0], Complex64::new(3.0, -4.0));
    assert!((m_val.data[0] - 5.0).abs() < 1e-12);
    assert!((phi_val.data[0] - (4.0_f64).atan2(3.0)).abs() < 1e-12);
    assert_eq!(zr.data[0], 5.0);
    assert_eq!(zi.data[0], 0.0);
}

#[test]
fn test_isreal_builtin() {
    let r = run_source(
        "t1 = isreal(42); \
         t2 = isreal([1, 2; 3, 4]); \
         t3 = isreal(3 + 4i); \
         t4 = isreal(complex(3, 0));",
    )
    .unwrap();

    assert_eq!(r.env.get("t1").unwrap(), &Value::Boolean(true));
    assert_eq!(r.env.get("t2").unwrap(), &Value::Boolean(true));
    assert_eq!(r.env.get("t3").unwrap(), &Value::Boolean(false));
    assert_eq!(r.env.get("t4").unwrap(), &Value::Boolean(true));
}

// ============================================================================
// Group C: Complex Scalar & Array Arithmetic
// ============================================================================

#[test]
fn test_scalar_complex_arithmetic() {
    let r = run_source(
        "z1 = 1 + 2i; \
         z2 = 3 + 4i; \
         add = z1 + z2; \
         sub = z1 - z2; \
         mul = z1 * z2; \
         div = z1 / z2;",
    )
    .unwrap();

    let add = env_complex(&r, "add");
    let sub = env_complex(&r, "sub");
    let mul = env_complex(&r, "mul");
    let div = env_complex(&r, "div");

    // (1+2i) + (3+4i) = 4 + 6i
    assert_eq!(add.data[0], Complex64::new(4.0, 6.0));
    // (1+2i) - (3+4i) = -2 - 2i
    assert_eq!(sub.data[0], Complex64::new(-2.0, -2.0));
    // (1+2i) * (3+4i) = (3 - 8) + (4 + 6)i = -5 + 10i
    assert_eq!(mul.data[0], Complex64::new(-5.0, 10.0));
    // (1+2i) / (3+4i) = ((1*3 + 2*4) + (2*3 - 1*4)i) / (9 + 16) = (11 + 2i)/25 = 0.44 + 0.08i
    assert!((div.data[0].re - 0.44).abs() < 1e-12);
    assert!((div.data[0].im - 0.08).abs() < 1e-12);
}

#[test]
fn test_mixed_real_and_complex_promotion() {
    let r = run_source(
        "A = [1, 2; 3, 4]; \
         z = 2i; \
         B = A + z; \
         C = A * z;",
    )
    .unwrap();

    let b = env_complex(&r, "B");
    let c = env_complex(&r, "C");

    assert_eq!(b.rows, 2);
    assert_eq!(b.cols, 2);
    assert_eq!(b.data[0], Complex64::new(1.0, 2.0));
    assert_eq!(b.data[1], Complex64::new(2.0, 2.0));
    assert_eq!(b.data[2], Complex64::new(3.0, 2.0));
    assert_eq!(b.data[3], Complex64::new(4.0, 2.0));

    assert_eq!(c.data[0], Complex64::new(0.0, 2.0));
    assert_eq!(c.data[1], Complex64::new(0.0, 4.0));
    assert_eq!(c.data[2], Complex64::new(0.0, 6.0));
    assert_eq!(c.data[3], Complex64::new(0.0, 8.0));
}

#[test]
fn test_complex_matrix_multiplication() {
    let r = run_source(
        "A = [1 + 1i, 2; 0, 1 - 1i]; \
         B = [1, 0; 1i, 2]; \
         C = A * B;",
    )
    .unwrap();

    let c = env_complex(&r, "C");
    assert_eq!(c.rows, 2);
    assert_eq!(c.cols, 2);
    // Row 0:
    // C(1,1) = (1+1i)*1 + 2*1i = 1 + 3i
    // C(1,2) = (1+1i)*0 + 2*2 = 4
    // Row 1:
    // C(2,1) = 0*1 + (1-1i)*1i = 1i - 1i^2 = 1 + 1i
    // C(2,2) = 0*0 + (1-1i)*2 = 2 - 2i
    assert_eq!(c.data[0], Complex64::new(1.0, 3.0));
    assert_eq!(c.data[1], Complex64::new(4.0, 0.0));
    assert_eq!(c.data[2], Complex64::new(1.0, 1.0));
    assert_eq!(c.data[3], Complex64::new(2.0, -2.0));
}

// ============================================================================
// Group D: Transpose vs Conjugate Transpose Semantics
// ============================================================================

#[test]
fn test_transpose_semantics() {
    let r = run_source(
        "A = [1 + 2i, 3 + 4i; 5 + 6i, 7 + 8i]; \
         H = A'; \
         T = A.';",
    )
    .unwrap();

    let h = env_complex(&r, "H");
    let t = env_complex(&r, "T");

    assert_eq!(h.rows, 2);
    assert_eq!(h.cols, 2);
    assert_eq!(t.rows, 2);
    assert_eq!(t.cols, 2);

    // Conjugate transpose conjugates:
    assert_eq!(h.data[0], Complex64::new(1.0, -2.0));
    assert_eq!(h.data[1], Complex64::new(5.0, -6.0));
    assert_eq!(h.data[2], Complex64::new(3.0, -4.0));
    assert_eq!(h.data[3], Complex64::new(7.0, -8.0));

    // Non-conjugate transpose does NOT conjugate:
    assert_eq!(t.data[0], Complex64::new(1.0, 2.0));
    assert_eq!(t.data[1], Complex64::new(5.0, 6.0));
    assert_eq!(t.data[2], Complex64::new(3.0, 4.0));
    assert_eq!(t.data[3], Complex64::new(7.0, 8.0));
}

#[test]
fn test_real_transpose_agreement() {
    let r = run_source(
        "R = [1, 2; 3, 4]; \
         H = R'; \
         T = R.';",
    )
    .unwrap();

    let h = env_mat(&r, "H");
    let t = env_mat(&r, "T");
    assert_eq!(h, t);
}

// ============================================================================
// Group E: Complex Indexing and Growth
// ============================================================================

#[test]
fn test_complex_indexing_and_promotion() {
    let r = run_source(
        "A = [1, 2; 3, 4]; \
         A(1, 1) = 5 + 6i; \
         e1 = A(1, 1); \
         e2 = A(1, 2);",
    )
    .unwrap();

    let a = env_complex(&r, "A");
    let e1 = env_complex(&r, "e1");
    let e2 = env_complex(&r, "e2");

    assert_eq!(a.data[0], Complex64::new(5.0, 6.0));
    assert_eq!(a.data[1], Complex64::new(2.0, 0.0));
    assert_eq!(e1.data[0], Complex64::new(5.0, 6.0));
    assert_eq!(e2.data[0], Complex64::new(2.0, 0.0));
}

#[test]
fn test_complex_growth_linear() {
    let r = run_source("z = 1 + 1i; z(4) = 2 - 3i;").unwrap();
    let z = env_complex(&r, "z");
    assert_eq!(z.rows, 1);
    assert_eq!(z.cols, 4);
    assert_eq!(z.data[0], Complex64::new(1.0, 1.0));
    assert_eq!(z.data[1], Complex64::zero());
    assert_eq!(z.data[2], Complex64::zero());
    assert_eq!(z.data[3], Complex64::new(2.0, -3.0));
}

// ============================================================================
// Group F: Elementary Functions and Domain Promotion
// ============================================================================

#[test]
fn test_sqrt_negative_promotion() {
    let r = run_source(
        "s1 = sqrt(-4); \
         s2 = sqrt([-1, 4; 9, -16]);",
    )
    .unwrap();

    let s1 = env_complex(&r, "s1");
    let s2 = env_complex(&r, "s2");

    assert!((s1.data[0].re).abs() < 1e-12);
    assert!((s1.data[0].im - 2.0).abs() < 1e-12);

    assert!((s2.data[0].re).abs() < 1e-12);
    assert!((s2.data[0].im - 1.0).abs() < 1e-12);
    assert!((s2.data[1].re - 2.0).abs() < 1e-12);
    assert!((s2.data[1].im).abs() < 1e-12);
    assert!((s2.data[2].re - 3.0).abs() < 1e-12);
    assert!((s2.data[2].im).abs() < 1e-12);
    assert!((s2.data[3].re).abs() < 1e-12);
    assert!((s2.data[3].im - 4.0).abs() < 1e-12);
}

#[test]
fn test_power_negative_fractional_promotion() {
    let r = run_source("p = (-1) ^ 0.5;").unwrap();
    let p = env_complex(&r, "p");
    assert!((p.data[0].re).abs() < 1e-12);
    assert!((p.data[0].im - 1.0).abs() < 1e-12);
}

#[test]
fn test_euler_identity() {
    let r = run_source("e = exp(1i * pi);").unwrap();
    let e = env_complex(&r, "e");
    // e^(i*pi) = -1 + 0i
    assert!((e.data[0].re - (-1.0)).abs() < 1e-12);
    assert!(e.data[0].im.abs() < 1e-12);
}

#[test]
fn test_log_negative_promotion() {
    let r = run_source("l = log(-1);").unwrap();
    let l = env_complex(&r, "l");
    // ln(-1) = 0 + pi*i
    assert!(l.data[0].re.abs() < 1e-12);
    assert!((l.data[0].im - std::f64::consts::PI).abs() < 1e-12);
}

// ============================================================================
// Group G: Comparisons
// ============================================================================

#[test]
fn test_complex_comparisons() {
    let r = run_source(
        "eq1 = (1 + 2i) == (1 + 2i); \
         eq2 = (1 + 2i) == (1 + 3i); \
         ne1 = (1 + 2i) ~= (1 + 3i); \
         lt1 = (1 + 10i) < (2 + 1i); \
         gt1 = (3 + 1i) > (2 + 50i);",
    )
    .unwrap();

    let eq1 = env_mat(&r, "eq1");
    let eq2 = env_mat(&r, "eq2");
    let ne1 = env_mat(&r, "ne1");
    let lt1 = env_mat(&r, "lt1");
    let gt1 = env_mat(&r, "gt1");

    assert_eq!(eq1.data[0], 1.0);
    assert_eq!(eq2.data[0], 0.0);
    assert_eq!(ne1.data[0], 1.0);
    // Relational comparisons compare real parts in MATLAB:
    assert_eq!(lt1.data[0], 1.0); // 1 < 2 is true
    assert_eq!(gt1.data[0], 1.0); // 3 > 2 is true
}

// ============================================================================
// Group H: Array Introspection (size, length, numel, isempty)
// ============================================================================

#[test]
fn test_complex_array_introspection() {
    let r = run_source(
        "Z = [1 + 1i, 2 + 2i, 3 + 3i; 4 + 4i, 5 + 5i, 6 + 6i]; \
         s = size(Z); \
         [r, c] = size(Z); \
         len = length(Z); \
         n = numel(Z); \
         emp = isempty(Z);",
    )
    .unwrap();

    let s = env_mat(&r, "s");
    let r_dim = env_mat(&r, "r");
    let c_dim = env_mat(&r, "c");
    let len = env_mat(&r, "len");
    let n = env_mat(&r, "n");
    let emp = env_mat(&r, "emp");

    assert_eq!(s.data, vec![2.0, 3.0]);
    assert_eq!(r_dim.data[0], 2.0);
    assert_eq!(c_dim.data[0], 3.0);
    assert_eq!(len.data[0], 3.0);
    assert_eq!(n.data[0], 6.0);
    assert_eq!(emp.data[0], 0.0);
}

// ============================================================================
// Group I: Persistent Session Integration
// ============================================================================

#[test]
fn test_complex_session_persistence() {
    let mut session = Session::new();
    let r1 = session.exec("z = 3 + 4i;", None).unwrap();
    assert_eq!(r1.env.get("z").unwrap().type_name(), "complex double");

    let r2 = session.exec("w = z * 2i;", None).unwrap();
    let w = r2.env.get("w").unwrap().as_complex_matrix().unwrap();
    // (3 + 4i) * 2i = 6i - 8 = -8 + 6i
    assert_eq!(w.data[0], Complex64::new(-8.0, 6.0));
}
