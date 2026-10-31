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
