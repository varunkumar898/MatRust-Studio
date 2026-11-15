//! Integration tests for Phase F17 Task MG7:
//! Math Gallery Sets 1 (Calculus) and 2 (ODE).

use rustlab_math::Value;
use rustlab_stdlib::new_interpreter;
use std::path::Path;

fn read_gallery_script(rel: &str) -> String {
    let p1 = Path::new(rel);
    if p1.exists() {
        return std::fs::read_to_string(p1).unwrap();
    }
    let p2 = Path::new("../../").join(rel);
    if p2.exists() {
        return std::fs::read_to_string(p2).unwrap();
    }
    panic!("Could not find {} at {:?} or {:?}", rel, p1, p2);
}

#[test]
fn test_gallery_helmholtz_gradient() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/calculus/helmholtz_gradient.m");
    interp.eval_script(&script).unwrap();

    let u = interp.env.get("U").expect("U matrix must exist");
    let v = interp.env.get("V").expect("V matrix must exist");
    assert!(matches!(u, Value::Double(_)));
    assert!(matches!(v, Value::Double(_)));
}

#[test]
fn test_gallery_laplacian_del2() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/calculus/laplacian_del2.m");
    interp.eval_script(&script).unwrap();

    let l = interp.env.get("L").expect("L matrix must exist");
    assert!(matches!(l, Value::Double(_)));
}

#[test]
fn test_gallery_vector_curl() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/calculus/vector_curl.m");
    interp.eval_script(&script).unwrap();

    let u = interp.env.get("U").expect("U matrix must exist");
    let v = interp.env.get("V").expect("V matrix must exist");
    assert!(matches!(u, Value::Double(_)));
    assert!(matches!(v, Value::Double(_)));
}

#[test]
fn test_gallery_adaptive_quadrature() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/calculus/adaptive_quadrature.m");
    interp.eval_script(&script).unwrap();

    let i_val = interp
        .env
        .get("I")
        .expect("I must exist")
        .as_scalar_f64()
        .unwrap();
    let exact = (1.0 - (-1.0_f64).exp()).powi(2) / 4.0;
    assert!(
        (i_val - exact).abs() < 1e-5,
        "Expected I ~ {}, got {}",
        exact,
        i_val
    );
}

#[test]
fn test_gallery_van_der_pol() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/ode/van_der_pol.m");
    interp.eval_script(&script).unwrap();

    let t = interp.env.get("t").expect("t must exist");
    let y = interp.env.get("y").expect("y must exist");
    assert!(matches!(t, Value::Double(_)));
    assert!(matches!(y, Value::Double(_)));
}

#[test]
fn test_gallery_lorenz_attractor() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/ode/lorenz_attractor.m");
    interp.eval_script(&script).unwrap();

    let y = interp.env.get("y").expect("y must exist");
    if let Value::Double(y_arr) = y {
        assert_eq!(y_arr.shape().get(1).copied().unwrap_or(0), 3);
    } else {
        panic!("Expected double matrix for y");
    }
}

#[test]
fn test_gallery_stiff_robertson() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/ode/stiff_robertson.m");
    interp.eval_script(&script).unwrap();

    let y = interp.env.get("y").expect("y must exist");
    assert!(matches!(y, Value::Double(_)));
}

#[test]
fn test_gallery_damped_pendulum() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/ode/damped_pendulum.m");
    interp.eval_script(&script).unwrap();

    let y = interp.env.get("y").expect("y must exist");
    assert!(matches!(y, Value::Double(_)));
}
