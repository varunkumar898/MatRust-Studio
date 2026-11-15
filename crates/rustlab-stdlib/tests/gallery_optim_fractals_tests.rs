//! Integration tests for Phase F17 Task MG9:
//! Math Gallery Sets 5 (Optimization) and 6 (Fractals & Chaos).

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
fn test_gallery_rosenbrock_fminunc() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/optimization/rosenbrock_fminunc.m");
    interp.eval_script(&script).unwrap();

    let x_opt = interp.env.get("x_opt").expect("x_opt must exist");
    assert!(matches!(x_opt, Value::Double(_)));
}

#[test]
fn test_gallery_nonlinear_fsolve() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/optimization/nonlinear_fsolve.m");
    interp.eval_script(&script).unwrap();

    let x_sol = interp.env.get("x_sol").expect("x_sol must exist");
    assert!(matches!(x_sol, Value::Double(_)));
}

#[test]
fn test_gallery_curve_fitting_lsq() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/optimization/curve_fitting_lsq.m");
    interp.eval_script(&script).unwrap();

    let p_opt = interp.env.get("p_opt").expect("p_opt must exist");
    assert!(matches!(p_opt, Value::Double(_)));
}

#[test]
fn test_gallery_simplex_linprog() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/optimization/simplex_linprog.m");
    interp.eval_script(&script).unwrap();

    let x_opt = interp.env.get("x_opt").expect("x_opt must exist");
    assert!(matches!(x_opt, Value::Double(_)));
}

#[test]
fn test_gallery_mandelbrot_set() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/fractals/mandelbrot_set.m");
    interp.eval_script(&script).unwrap();

    let m = interp.env.get("M").expect("M must exist");
    assert!(matches!(m, Value::Double(_)));
}

#[test]
fn test_gallery_julia_set() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/fractals/julia_set.m");
    interp.eval_script(&script).unwrap();

    let j = interp.env.get("J").expect("J must exist");
    assert!(matches!(j, Value::Double(_)));
}

#[test]
fn test_gallery_bifurcation_diagram() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/fractals/bifurcation_diagram.m");
    interp.eval_script(&script).unwrap();

    let r_plot = interp.env.get("r_plot").expect("r_plot must exist");
    let x_plot = interp.env.get("x_plot").expect("x_plot must exist");
    assert!(matches!(r_plot, Value::Double(_)));
    assert!(matches!(x_plot, Value::Double(_)));
}

#[test]
fn test_gallery_barnsley_fern() {
    let mut interp = new_interpreter();
    let script = read_gallery_script("examples/math/fractals/barnsley_fern.m");
    interp.eval_script(&script).unwrap();

    let x = interp.env.get("x").expect("x must exist");
    let y = interp.env.get("y").expect("y must exist");
    assert!(matches!(x, Value::Double(_)));
    assert!(matches!(y, Value::Double(_)));
}
