//! Integration tests for numerical methods: roots, polyval, polyfit, trapz, cumtrapz, gradient, interp1.

use rustlab_math::Value;
use rustlab_stdlib::eval_matlab;

#[test]
fn test_roots_linear_and_quadratic() {
    // Degree 1: 2*x - 4 = 0 => x = 2
    let res = eval_matlab("roots([2, -4])").unwrap().unwrap();
    match res {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[1, 1]);
            assert!((d.as_slice()[0] - 2.0).abs() < 1e-12);
        }
        _ => panic!("Expected double array"),
    }

    // Degree 2 real: x^2 - 3x + 2 = 0 => x = 2, 1
    let res2 = eval_matlab("roots([1, -3, 2])").unwrap().unwrap();
    match res2 {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[2, 1]);
            let s = d.as_slice();
            assert!((s[0] - 2.0).abs() < 1e-12);
            assert!((s[1] - 1.0).abs() < 1e-12);
        }
        _ => panic!("Expected double array"),
    }

    // Degree 2 complex: x^2 + 1 = 0 => x = +/- 1i
    let res3 = eval_matlab("roots([1, 0, 1])").unwrap().unwrap();
    match res3 {
        Value::ComplexDouble(cd) => {
            assert_eq!(cd.shape(), &[2, 1]);
            let s = cd.as_slice();
            assert!(s[0].re.abs() < 1e-12);
            assert!((s[0].im.abs() - 1.0).abs() < 1e-12);
            assert!(s[1].re.abs() < 1e-12);
            assert!((s[1].im.abs() - 1.0).abs() < 1e-12);
        }
        _ => panic!("Expected complex double array"),
    }

    // Residual check on polynomial roots: p(x) = x^2 - 4x + 13 => roots = 2 +/- 3i
    // Evaluate p(r) = r^2 - 4*r + 13 for each root and assert residual is < 1e-10
    let res4 = eval_matlab("roots([1, -4, 13])").unwrap().unwrap();
    match res4 {
        Value::ComplexDouble(cd) => {
            for r in cd.as_slice() {
                // r^2 - 4*r + 13
                let r2_re = r.re * r.re - r.im * r.im;
                let r2_im = 2.0 * r.re * r.im;
                let val_re = r2_re - 4.0 * r.re + 13.0;
                let val_im = r2_im - 4.0 * r.im;
                let residual = (val_re * val_re + val_im * val_im).sqrt();
                assert!(
                    residual < 1e-10,
                    "Root residual {residual:e} must be < 1e-10"
                );
            }
        }
        _ => panic!("Expected complex double roots"),
    }
}

#[test]
fn test_polyval_scalar_and_vector() {
    // p = 3x^2 + 2x + 1
    // p(5) = 3*25 + 2*5 + 1 = 86
    let res = eval_matlab("polyval([3, 2, 1], 5)").unwrap().unwrap();
    assert!((res.as_scalar_f64().unwrap() - 86.0).abs() < 1e-12);

    // p([5, 7]) = [86, 3*49 + 2*7 + 1] = [86, 162]
    let res2 = eval_matlab("polyval([3, 2, 1], [5, 7])").unwrap().unwrap();
    match res2 {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[86.0, 162.0]);
        }
        _ => panic!("Expected double array"),
    }
}

#[test]
fn test_polyfit_linear_and_quadratic() {
    // Linear fit: y = 2x
    let res = eval_matlab("polyfit([1, 2, 3], [2, 4, 6], 1)")
        .unwrap()
        .unwrap();
    match res {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[1, 2]);
            let s = d.as_slice();
            assert!((s[0] - 2.0).abs() < 1e-12);
            assert!(s[1].abs() < 1e-12);
        }
        _ => panic!("Expected double row vector"),
    }

    // Quadratic fit: y = x^2 + 1 => p = [1, 0, 1]
    let res2 = eval_matlab("polyfit([0, 1, 2], [1, 2, 5], 2)")
        .unwrap()
        .unwrap();
    match res2 {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[1, 3]);
            let s = d.as_slice();
            assert!((s[0] - 1.0).abs() < 1e-12);
            assert!(s[1].abs() < 1e-12);
            assert!((s[2] - 1.0).abs() < 1e-12);
        }
        _ => panic!("Expected double row vector"),
    }

    // Exact polynomial recovery within 1e-10:
    // True polynomial: p(x) = 3*x^3 - 2*x^2 + 5*x - 7
    // Sample at 5 points: x = [1, 2, 3, 4, 5]
    let mut interp = rustlab_stdlib::new_interpreter();
    let script_exact = "x = [1, 2, 3, 4, 5]; y = 3*x.^3 - 2*x.^2 + 5*x - 7; p = polyfit(x, y, 3);";
    interp.eval_script(script_exact).unwrap();
    let p_val = interp.env.get("p").unwrap();
    match p_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[1, 4]);
            let s = d.as_slice();
            assert!(
                (s[0] - 3.0).abs() < 1e-10,
                "Cubic coefficient mismatch: {}",
                s[0]
            );
            assert!(
                (s[1] - (-2.0)).abs() < 1e-10,
                "Quadratic coefficient mismatch: {}",
                s[1]
            );
            assert!(
                (s[2] - 5.0).abs() < 1e-10,
                "Linear coefficient mismatch: {}",
                s[2]
            );
            assert!(
                (s[3] - (-7.0)).abs() < 1e-10,
                "Constant coefficient mismatch: {}",
                s[3]
            );
        }
        _ => panic!("Expected double vector"),
    }

    // Noisy line vs closed-form OLS:
    // x = [1, 2, 3, 4, 5], y = [2.1, 3.9, 6.2, 8.0, 9.8]
    // Closed form formulas:
    // x_mean = 3.0, y_mean = 30.0 / 5 = 6.0
    // sum((x - x_mean)^2) = 4 + 1 + 0 + 1 + 4 = 10
    // sum((x - x_mean)*(y - y_mean)) = (-2)*(-3.9) + (-1)*(-2.1) + 0 + 1*(2.0) + 2*(3.8)
    //                                = 7.8 + 2.1 + 0 + 2.0 + 7.6 = 19.5
    // slope m = 19.5 / 10 = 1.95
    // intercept c = 6.0 - 1.95 * 3.0 = 6.0 - 5.85 = 0.15
    let script_ols = "p_line = polyfit([1, 2, 3, 4, 5], [2.1, 3.9, 6.2, 8.0, 9.8], 1);";
    interp.eval_script(script_ols).unwrap();
    let p_line = interp.env.get("p_line").unwrap();
    match p_line {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[1, 2]);
            let s = d.as_slice();
            assert!((s[0] - 1.95).abs() < 1e-12, "Slope mismatch: got {}", s[0]);
            assert!(
                (s[1] - 0.15).abs() < 1e-12,
                "Intercept mismatch: got {}",
                s[1]
            );
        }
        _ => panic!("Expected double vector"),
    }

    // Ill-conditioned degree-8 fit residual:
    // x in [0.1, 0.2, ..., 1.2] (12 points), degree 8 fit on smooth exp(x)
    let script_deg8 = "x8 = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
                       y8 = exp(x8);
                       p8 = polyfit(x8, y8, 8);
                       y8_fit = polyval(p8, x8);
                       err8 = max(abs(y8 - y8_fit));";
    interp.eval_script(script_deg8).unwrap();
    let err8 = interp.env.get("err8").unwrap().as_scalar_f64().unwrap();
    assert!(
        err8 < 1e-5,
        "Degree-8 fit max residual {err8:e} must be < 1e-5"
    );

    // Degree-8 fit against polynomial with exactly representable dyadic float coefficients:
    // p_true(x) = 1*x^8 - 2*x^6 + 0.5*x^4 - 0.25*x^2 + 1
    // p_true vector: [1, 0, -2, 0, 0.5, 0, -0.25, 0, 1] (length 9)
    // Sample at 13 distinct points in [-1.0, 1.0]
    let script_deg8_exact = "
        p_true = [1.0, 0.0, -2.0, 0.0, 0.5, 0.0, -0.25, 0.0, 1.0];
        x_pts = [-1.0, -0.8, -0.6, -0.4, -0.2, -0.1, 0.0, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0];
        y_pts = polyval(p_true, x_pts);
        p8_fit = polyfit(x_pts, y_pts, 8);
        fit_resid = max(abs(y_pts - polyval(p8_fit, x_pts)));
        coeff_err = max(abs(p8_fit - p_true));
    ";
    interp.eval_script(script_deg8_exact).unwrap();
    let fit_resid = interp
        .env
        .get("fit_resid")
        .unwrap()
        .as_scalar_f64()
        .unwrap();
    let coeff_err = interp
        .env
        .get("coeff_err")
        .unwrap()
        .as_scalar_f64()
        .unwrap();
    assert!(
        fit_resid < 1e-10,
        "Degree-8 exact fit residual {fit_resid:e} must be < 1e-10"
    );
    assert!(
        coeff_err < 1e-7,
        "Degree-8 coefficient error {coeff_err:e} must be < 1e-7"
    );
}

#[test]
fn test_trapz_and_cumtrapz() {
    // trapz([1, 2, 3]) with unit spacing: (1+2)/2 + (2+3)/2 = 1.5 + 2.5 = 4.0
    let res = eval_matlab("trapz([1, 2, 3])").unwrap().unwrap();
    assert!((res.as_scalar_f64().unwrap() - 4.0).abs() < 1e-12);

    // trapz([0, 1, 2], [0, 1, 4]): (0+1)/2*1 + (1+4)/2*1 = 0.5 + 2.5 = 3.0
    let res2 = eval_matlab("trapz([0, 1, 2], [0, 1, 4])").unwrap().unwrap();
    assert!((res2.as_scalar_f64().unwrap() - 3.0).abs() < 1e-12);

    // cumtrapz([1, 2, 3]) -> [0, 1.5, 4.0]
    let res3 = eval_matlab("cumtrapz([1, 2, 3])").unwrap().unwrap();
    match res3 {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[0.0, 1.5, 4.0]);
        }
        _ => panic!("Expected double vector"),
    }

    // Non-uniform spacing hand-computed:
    // x = [0, 1, 3, 6], y = [1, 2, 4, 7]
    // intervals: dx = [1, 2, 3]
    // trapezoids: (1+2)/2 * 1 = 1.5
    //             (2+4)/2 * 2 = 6.0
    //             (4+7)/2 * 3 = 16.5
    // total = 1.5 + 6.0 + 16.5 = 24.0
    // cumtrapz = [0, 1.5, 7.5, 24.0]
    let res_nu = eval_matlab("trapz([0, 1, 3, 6], [1, 2, 4, 7])")
        .unwrap()
        .unwrap();
    assert!((res_nu.as_scalar_f64().unwrap() - 24.0).abs() < 1e-12);

    let res_cnu = eval_matlab("cumtrapz([0, 1, 3, 6], [1, 2, 4, 7])")
        .unwrap()
        .unwrap();
    match res_cnu {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[0.0, 1.5, 7.5, 24.0]);
        }
        _ => panic!("Expected double vector"),
    }
}

#[test]
fn test_gradient() {
    // gradient([1, 4, 9, 16]):
    // g[0] = 4 - 1 = 3
    // g[1] = (9 - 1)/2 = 4
    // g[2] = (16 - 4)/2 = 6
    // g[3] = 16 - 9 = 7
    let res = eval_matlab("gradient([1, 4, 9, 16])").unwrap().unwrap();
    match res {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[3.0, 4.0, 6.0, 7.0]);
        }
        _ => panic!("Expected double vector"),
    }

    // With spacing h = 2: divide by 2
    let res2 = eval_matlab("gradient([1, 4, 9, 16], 2)").unwrap().unwrap();
    match res2 {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[1.5, 2.0, 3.0, 3.5]);
        }
        _ => panic!("Expected double vector"),
    }
}

#[test]
fn test_interp1_linear_and_nearest() {
    // Linear
    let res = eval_matlab("interp1([1, 2, 3], [10, 20, 30], [1.5, 2.5])")
        .unwrap()
        .unwrap();
    match res {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[15.0, 25.0]);
        }
        _ => panic!("Expected double vector"),
    }

    // Nearest
    let res2 = eval_matlab("interp1([1, 2, 3], [10, 20, 30], [1.2, 2.8], 'nearest')")
        .unwrap()
        .unwrap();
    match res2 {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[10.0, 30.0]);
        }
        _ => panic!("Expected double vector"),
    }
}

#[test]
fn test_2d_gradient_and_del2() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // Z = [1 2 3; 4 5 6; 7 8 9]
    // dZ/dx along columns is 1.0 everywhere
    // dZ/dy along rows is 3.0 everywhere
    let script = "[Fx, Fy] = gradient([1, 2, 3; 4, 5, 6; 7, 8, 9]);";
    interp.eval_script(script).unwrap();
    let fx = interp.env.get("Fx").unwrap();
    let fy = interp.env.get("Fy").unwrap();

    match (fx, fy) {
        (Value::Double(fx_arr), Value::Double(fy_arr)) => {
            assert_eq!(fx_arr.shape(), &[3, 3]);
            assert_eq!(fy_arr.shape(), &[3, 3]);
            for &v in fx_arr.as_slice() {
                assert!(
                    (v - 1.0).abs() < 1e-12,
                    "Expected Fx = 1.0 everywhere, got {v}"
                );
            }
            for &v in fy_arr.as_slice() {
                assert!(
                    (v - 3.0).abs() < 1e-12,
                    "Expected Fy = 3.0 everywhere, got {v}"
                );
            }
        }
        _ => panic!("Expected Double matrices for gradient outputs"),
    }

    // del2 of a plane is 0 everywhere
    let script_del2 = "L = del2([1, 2, 3; 4, 5, 6; 7, 8, 9]);";
    interp.eval_script(script_del2).unwrap();
    let l_val = interp.env.get("L").unwrap();
    match l_val {
        Value::Double(l_arr) => {
            for &v in l_arr.as_slice() {
                assert!(
                    v.abs() < 1e-12,
                    "Expected del2 of linear surface = 0.0, got {v}"
                );
            }
        }
        _ => panic!("Expected Double matrix for del2 output"),
    }
}

#[test]
fn test_ind2sub_and_sub2ind() {
    let mut interp = rustlab_stdlib::new_interpreter();
    let script = "[r, c] = ind2sub([3, 3], [1, 5, 9]); idx = sub2ind([3, 3], r, c);";
    interp.eval_script(script).unwrap();

    let r_val = interp.env.get("r").unwrap();
    let c_val = interp.env.get("c").unwrap();
    let idx_val = interp.env.get("idx").unwrap();

    match (r_val, c_val, idx_val) {
        (Value::Double(r), Value::Double(c), Value::Double(idx)) => {
            assert_eq!(r.as_slice(), &[1.0, 2.0, 3.0]);
            assert_eq!(c.as_slice(), &[1.0, 2.0, 3.0]);
            assert_eq!(idx.as_slice(), &[1.0, 5.0, 9.0]);
        }
        _ => panic!("Expected double vectors"),
    }
}

#[test]
fn test_mldivide_rectangular_least_squares_and_linsolve() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // Overdetermined system:
    // A = [1 1; 1 2; 1 3]
    // b = [2; 3; 4]
    // Exact line y = 1 + 1*x => x = [1; 1]
    let script =
        "x = [1, 1; 1, 2; 1, 3] \\ [2; 3; 4]; x_lin = linsolve([1, 1; 1, 2; 1, 3], [2; 3; 4]);";
    interp.eval_script(script).unwrap();

    let x_val = interp.env.get("x").unwrap();
    let x_lin = interp.env.get("x_lin").unwrap();

    match (x_val, x_lin) {
        (Value::Double(x), Value::Double(x_l)) => {
            assert_eq!(x.shape(), &[2, 1]);
            assert!((x.as_slice()[0] - 1.0).abs() < 1e-10);
            assert!((x.as_slice()[1] - 1.0).abs() < 1e-10);

            assert_eq!(x_l.shape(), &[2, 1]);
            assert!((x_l.as_slice()[0] - 1.0).abs() < 1e-10);
            assert!((x_l.as_slice()[1] - 1.0).abs() < 1e-10);
        }
        _ => panic!("Expected double vectors"),
    }
}

#[test]
fn test_ode45_and_ode23_exponential_decay() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // dy/dt = -2*y, y(0) = 5. Exact solution at t=1: 5*exp(-2) ~ 0.6766764
    let script = "[t, y] = ode45(@(t, y) -2*y, [0, 1], 5);";
    interp.eval_script(script).unwrap();

    let t_val = interp.env.get("t").unwrap();
    let y_val = interp.env.get("y").unwrap();

    match (t_val, y_val) {
        (Value::Double(t), Value::Double(y)) => {
            let t_slice = t.as_slice();
            let y_slice = y.as_slice();
            assert!(t_slice.len() > 2);
            let final_t = *t_slice.last().unwrap();
            let final_y = *y_slice.last().unwrap();
            assert!((final_t - 1.0).abs() < 1e-10);
            let expected = 5.0 * (-2.0_f64).exp();
            assert!(
                (final_y - expected).abs() < 1e-3,
                "ode45 final error: got {final_y}, expected {expected}"
            );
        }
        _ => panic!("Expected double outputs from ode45"),
    }

    // Test ode23
    let script23 = "[t23, y23] = ode23(@(t, y) -2*y, [0, 1], 5);";
    interp.eval_script(script23).unwrap();
    let y23_val = interp.env.get("y23").unwrap();
    match y23_val {
        Value::Double(y) => {
            let final_y = *y.as_slice().last().unwrap();
            let expected = 5.0 * (-2.0_f64).exp();
            assert!(
                (final_y - expected).abs() < 1e-2,
                "ode23 final error: got {final_y}, expected {expected}"
            );
        }
        _ => panic!("Expected double outputs from ode23"),
    }
}

#[test]
fn test_fzero_and_fminsearch() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // fzero: cos(x) - x = 0 -> Dottie number ~ 0.7390851332
    let script_fz = "x_root = fzero(@(x) cos(x) - x, 0.5);";
    interp.eval_script(script_fz).unwrap();
    let x_root = interp.env.get("x_root").unwrap();
    match x_root {
        Value::Double(d) => {
            let r = d.as_slice()[0];
            assert!((r - 0.7390851332).abs() < 1e-6, "fzero root error: got {r}");
        }
        _ => panic!("Expected double scalar from fzero"),
    }

    // fminsearch: parabola f(x) = (x(1) - 3)^2 + (x(2) - 4)^2 -> min at [3; 4]
    let script_fmin = "x_min = fminsearch(@(x) (x(1) - 3)^2 + (x(2) - 4)^2, [0; 0]);";
    interp.eval_script(script_fmin).unwrap();
    let x_min = interp.env.get("x_min").unwrap();
    match x_min {
        Value::Double(d) => {
            let s = d.as_slice();
            assert!(
                (s[0] - 3.0).abs() < 1e-3,
                "fminsearch x1 error: got {}",
                s[0]
            );
            assert!(
                (s[1] - 4.0).abs() < 1e-3,
                "fminsearch x2 error: got {}",
                s[1]
            );
        }
        _ => panic!("Expected double vector from fminsearch"),
    }
}

#[test]
fn test_integral_quadrature() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // integral of sin(x) from 0 to pi is 2.0
    let script = "q = integral(@(x) sin(x), 0, 3.141592653589793);";
    interp.eval_script(script).unwrap();
    let q_val = interp.env.get("q").unwrap();
    match q_val {
        Value::Double(d) => {
            let q = d.as_slice()[0];
            assert!((q - 2.0).abs() < 1e-6, "integral error: got {q}");
        }
        _ => panic!("Expected double scalar from integral"),
    }
}

#[test]
fn test_divergence_and_curl_2d() {
    let mut interp = rustlab_stdlib::new_interpreter();
    let script = r#"
        [x, y] = meshgrid(1:3, 1:3);
        div = divergence(x, y, x, y);
        [cav, vor] = curl(x, y, -y, x);
    "#;
    interp.eval_script(script).unwrap();

    let div_val = interp.env.get("div").unwrap();
    match div_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 3]);
            for &v in d.as_slice() {
                assert!(
                    (v - 2.0).abs() < 1e-12,
                    "Expected divergence = 2.0, got {v}"
                );
            }
        }
        _ => panic!("Expected double matrix for divergence"),
    }

    let cav_val = interp.env.get("cav").unwrap();
    match cav_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 3]);
            for &v in d.as_slice() {
                assert!((v - 1.0).abs() < 1e-12, "Expected cav = 1.0, got {v}");
            }
        }
        _ => panic!("Expected double matrix for cav"),
    }

    let vor_val = interp.env.get("vor").unwrap();
    match vor_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 3]);
            for &v in d.as_slice() {
                assert!((v - 2.0).abs() < 1e-12, "Expected vor = 2.0, got {v}");
            }
        }
        _ => panic!("Expected double matrix for vor"),
    }
}

#[test]
fn test_del2_quadratic_surfaces_and_vectors() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // 2D: z = x^2 + y^2 => nabla^2 z = 4 => del2 = 4/4 = 1.0
    let script2d = r#"
        [x, y] = meshgrid(-2:2, -2:2);
        z = x.^2 + y.^2;
        L2 = del2(z);
    "#;
    interp.eval_script(script2d).unwrap();
    let l2_val = interp.env.get("L2").unwrap();
    match l2_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[5, 5]);
            for &v in d.as_slice() {
                assert!(
                    (v - 1.0).abs() < 1e-12,
                    "Expected del2(x^2+y^2) = 1.0, got {v}"
                );
            }
        }
        _ => panic!("Expected double matrix for del2"),
    }

    // 1D: u = x^2 => d^2u/dx^2 = 2 => del2 = 2/2 = 1.0
    let script1d = r#"
        x = -2:2;
        u = x.^2;
        L1 = del2(u);
    "#;
    interp.eval_script(script1d).unwrap();
    let l1_val = interp.env.get("L1").unwrap();
    match l1_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[1, 5]);
            for &v in d.as_slice() {
                assert!(
                    (v - 1.0).abs() < 1e-12,
                    "Expected 1D del2(x^2) = 1.0, got {v}"
                );
            }
        }
        _ => panic!("Expected double vector for 1D del2"),
    }
}

#[test]
fn test_quad_simpson_quadrature() {
    let mut interp = rustlab_stdlib::new_interpreter();
    let script = r#"
        q_poly = quad(@(x) x.^2, 0, 1);
        q_trig = quad(@(x) sin(x), 0, 3.141592653589793);
    "#;
    interp.eval_script(script).unwrap();

    let q_poly = interp.env.get("q_poly").unwrap();
    match q_poly {
        Value::Double(d) => {
            let q = d.as_slice()[0];
            assert!((q - 1.0 / 3.0).abs() < 1e-8, "Expected 1/3, got {q}");
        }
        _ => panic!("Expected double scalar from quad"),
    }

    let q_trig = interp.env.get("q_trig").unwrap();
    match q_trig {
        Value::Double(d) => {
            let q = d.as_slice()[0];
            assert!((q - 2.0).abs() < 1e-8, "Expected 2.0, got {q}");
        }
        _ => panic!("Expected double scalar from quad"),
    }
}
