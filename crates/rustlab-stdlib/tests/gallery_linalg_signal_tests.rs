//! Integration tests for Phase F17 Task MG8:
//! Math Gallery Sets 3 (Linear Algebra) and 4 (Signal Processing).

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
fn test_gallery_svd_image_compression() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/linear_algebra/svd_image_compression.m");
    interp.eval_script(&script).unwrap();

    let s = interp.env.get("sing_vals").expect("sing_vals must exist");
    assert!(matches!(s, Value::Double(_)));
}

#[test]
fn test_gallery_qr_least_squares() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/linear_algebra/qr_least_squares.m");
    interp.eval_script(&script).unwrap();

    let c = interp.env.get("c").expect("c coefficients must exist");
    assert!(matches!(c, Value::Double(_)));
}

#[test]
fn test_gallery_eigen_modes() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/linear_algebra/eigen_modes.m");
    interp.eval_script(&script).unwrap();

    let omega = interp.env.get("omega").expect("omega must exist");
    assert!(matches!(omega, Value::Double(_)));
}

#[test]
fn test_gallery_matrix_exponential() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/linear_algebra/matrix_exponential.m");
    interp.eval_script(&script).unwrap();

    let x1 = interp.env.get("x1").expect("x1 must exist");
    assert!(matches!(x1, Value::Double(_)));
}

#[test]
fn test_gallery_spectral_pwelch() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/signal_processing/spectral_pwelch.m");
    interp.eval_script(&script).unwrap();

    let pxx = interp.env.get("pxx").expect("pxx must exist");
    let f = interp.env.get("f").expect("f must exist");
    assert!(matches!(pxx, Value::Double(_)));
    assert!(matches!(f, Value::Double(_)));
}

#[test]
fn test_gallery_spectrogram_chirp() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/signal_processing/spectrogram_chirp.m");
    interp.eval_script(&script).unwrap();

    let s = interp.env.get("S").expect("S must exist");
    assert!(matches!(s, Value::ComplexDouble(_)) || matches!(s, Value::Double(_)));
}

#[test]
fn test_gallery_butterworth_filter() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/signal_processing/butterworth_filter.m");
    interp.eval_script(&script).unwrap();

    let h = interp.env.get("h").expect("h must exist");
    assert!(matches!(h, Value::ComplexDouble(_)) || matches!(h, Value::Double(_)));
}

#[test]
fn test_gallery_spline_interpolation() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/signal_processing/spline_interpolation.m");
    interp.eval_script(&script).unwrap();

    let y_spline = interp.env.get("y_spline").expect("y_spline must exist");
    let y_pchip = interp.env.get("y_pchip").expect("y_pchip must exist");
    assert!(matches!(y_spline, Value::Double(_)));
    assert!(matches!(y_pchip, Value::Double(_)));
}
