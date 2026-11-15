//! Unit tests for Phase F17 Task MG4 Optimization Suite:
//! `optimset`, `fminbnd`, `fminunc`, `fsolve`, `lsqnonlin`, `lsqcurvefit`, `linprog`.

use rustlab_stdlib::new_interpreter;

#[test]
fn test_optimset_options_management() {
    let mut interp = new_interpreter();
    let script = r#"
        opts0 = optimset();
        opts1 = optimset('fminunc');
        opts2 = optimset('TolFun', 1e-8, 'MaxIter', 1000);
        tol_fun = opts2.TolFun;
        max_iter = opts2.MaxIter;
    "#;
    interp.eval_script(script).unwrap();

    let tol_val = interp.env.get("tol_fun").unwrap().as_scalar_f64().unwrap();
    let max_iter_val = interp.env.get("max_iter").unwrap().as_scalar_f64().unwrap();
    assert!((tol_val - 1e-8).abs() < 1e-15);
    assert!((max_iter_val - 1000.0).abs() < 1e-6);
}

#[test]
fn test_fminbnd_scalar_minimization() {
    let mut interp = new_interpreter();
    let script = r#"
        % Parabola minimum at x = 3, fval = 4
        f_parab = @(x) (x - 3).^2 + 4;
        [x_p, fval_p, exit_p] = fminbnd(f_parab, 0, 5);

        % Sine minimum in [3, 6] at x = 3*pi/2 ~ 4.71238898
        [x_s, fval_s] = fminbnd(@(x) sin(x), 3, 6);
    "#;
    interp.eval_script(script).unwrap();

    let x_p = interp.env.get("x_p").unwrap().as_scalar_f64().unwrap();
    let fval_p = interp.env.get("fval_p").unwrap().as_scalar_f64().unwrap();
    let exit_p = interp.env.get("exit_p").unwrap().as_scalar_f64().unwrap();
    assert!((x_p - 3.0).abs() < 1e-4, "Expected x=3.0, got {x_p}");
    assert!(
        (fval_p - 4.0).abs() < 1e-4,
        "Expected fval=4.0, got {fval_p}"
    );
    assert_eq!(exit_p, 1.0);

    let x_s = interp.env.get("x_s").unwrap().as_scalar_f64().unwrap();
    let fval_s = interp.env.get("fval_s").unwrap().as_scalar_f64().unwrap();
    let expected_s = 1.5 * std::f64::consts::PI;
    assert!(
        (x_s - expected_s).abs() < 1e-4,
        "Expected x ~ 4.7124, got {x_s}"
    );
    assert!(
        (fval_s - (-1.0)).abs() < 1e-4,
        "Expected fval ~ -1.0, got {fval_s}"
    );
}

#[test]
fn test_fminunc_multivariable_bfgs() {
    let mut interp = new_interpreter();
    let script = r#"
        % 2D quadratic bowl: f(x) = (x1 - 2)^2 + 4*(x2 - 5)^2 -> min at [2, 5], fval = 0
        f_quad = @(x) (x(1) - 2.0).^2 + 4.0 * (x(2) - 5.0).^2;
        [x_q, fval_q, exit_q] = fminunc(f_quad, [0.0; 0.0]);
        xq1 = x_q(1);
        xq2 = x_q(2);
    "#;
    interp.eval_script(script).unwrap();

    let xq1 = interp.env.get("xq1").unwrap().as_scalar_f64().unwrap();
    let xq2 = interp.env.get("xq2").unwrap().as_scalar_f64().unwrap();
    let fval_q = interp.env.get("fval_q").unwrap().as_scalar_f64().unwrap();
    let exit_q = interp.env.get("exit_q").unwrap().as_scalar_f64().unwrap();

    assert!((xq1 - 2.0).abs() < 1e-4, "Expected xq1=2.0, got {xq1}");
    assert!((xq2 - 5.0).abs() < 1e-4, "Expected xq2=5.0, got {xq2}");
    assert!(fval_q < 1e-6, "Expected fval near 0, got {fval_q}");
    assert_eq!(exit_q, 1.0);
}

#[test]
fn test_fsolve_nonlinear_system() {
    let mut interp = new_interpreter();
    let script = r#"
        % System:
        % x(1)^2 + x(2)^2 - 2 = 0
        % x(1) - x(2) = 0
        % Exact positive root: [1.0, 1.0]
        f_sys = @(x) [x(1)^2 + x(2)^2 - 2.0; x(1) - x(2)];
        [x_sol, fval_sol, exit_sol] = fsolve(f_sys, [0.5; 0.5]);
        x1 = x_sol(1);
        x2 = x_sol(2);
    "#;
    interp.eval_script(script).unwrap();

    let x1 = interp.env.get("x1").unwrap().as_scalar_f64().unwrap();
    let x2 = interp.env.get("x2").unwrap().as_scalar_f64().unwrap();
    let exit_sol = interp.env.get("exit_sol").unwrap().as_scalar_f64().unwrap();

    assert!((x1 - 1.0).abs() < 1e-4, "Expected x1=1.0, got {x1}");
    assert!((x2 - 1.0).abs() < 1e-4, "Expected x2=1.0, got {x2}");
    assert_eq!(exit_sol, 1.0);
}

#[test]
fn test_lsqnonlin_and_lsqcurvefit() {
    let mut interp = new_interpreter();
    let script = r#"
        % lsqnonlin: solve 2*x(1) - 6 = 0, 3*x(2) - 12 = 0 => x = [3, 4]
        f_res = @(x) [2.0*x(1) - 6.0; 3.0*x(2) - 12.0];
        [x_lsq, resnorm] = lsqnonlin(f_res, [0.0; 0.0]);
        xl1 = x_lsq(1);
        xl2 = x_lsq(2);

        % lsqcurvefit: fit y = a * exp(b * x)
        % true a = 2.0, true b = 0.5
        xdata = [0.0, 1.0, 2.0, 3.0];
        ydata = [2.0, 3.2974425414, 5.4365636569, 8.9633781406];
        model = @(p, x) p(1) * exp(p(2) * x);
        [p_fit, resnorm_c] = lsqcurvefit(model, [1.0; 0.1], xdata, ydata);
        a_fit = p_fit(1);
        b_fit = p_fit(2);
    "#;
    interp.eval_script(script).unwrap();

    let xl1 = interp.env.get("xl1").unwrap().as_scalar_f64().unwrap();
    let xl2 = interp.env.get("xl2").unwrap().as_scalar_f64().unwrap();
    let resnorm = interp.env.get("resnorm").unwrap().as_scalar_f64().unwrap();

    assert!((xl1 - 3.0).abs() < 1e-4, "Expected xl1=3.0, got {xl1}");
    assert!((xl2 - 4.0).abs() < 1e-4, "Expected xl2=4.0, got {xl2}");
    assert!(resnorm < 1e-6, "Expected resnorm near 0, got {resnorm}");

    let a_fit = interp.env.get("a_fit").unwrap().as_scalar_f64().unwrap();
    let b_fit = interp.env.get("b_fit").unwrap().as_scalar_f64().unwrap();
    assert!((a_fit - 2.0).abs() < 1e-3, "Expected a ~ 2.0, got {a_fit}");
    assert!((b_fit - 0.5).abs() < 1e-3, "Expected b ~ 0.5, got {b_fit}");
}

#[test]
fn test_linprog_two_phase_simplex() {
    let mut interp = new_interpreter();
    let script = r#"
        % Maximize x1 + 2*x2  <=> Minimize -x1 - 2*x2
        % subject to:
        %   x1 + x2 <= 4
        %   2*x1 + x2 <= 5
        %   x1 >= 0, x2 >= 0
        % Optimal vertex at (x1=0, x2=4), fval = -8.0
        f = [-1.0; -2.0];
        A = [1.0, 1.0; 2.0, 1.0];
        b = [4.0; 5.0];
        lb = [0.0; 0.0];
        [x_lp, fval_lp, exit_lp] = linprog(f, A, b, [], [], lb);
        x_lp1 = x_lp(1);
        x_lp2 = x_lp(2);
    "#;
    interp.eval_script(script).unwrap();

    let x_lp1 = interp.env.get("x_lp1").unwrap().as_scalar_f64().unwrap();
    let x_lp2 = interp.env.get("x_lp2").unwrap().as_scalar_f64().unwrap();
    let fval_lp = interp.env.get("fval_lp").unwrap().as_scalar_f64().unwrap();
    let exit_lp = interp.env.get("exit_lp").unwrap().as_scalar_f64().unwrap();

    assert!(
        (x_lp1 - 0.0).abs() < 1e-4,
        "Expected x_lp1=0.0, got {x_lp1}"
    );
    assert!(
        (x_lp2 - 4.0).abs() < 1e-4,
        "Expected x_lp2=4.0, got {x_lp2}"
    );
    assert!(
        (fval_lp - (-8.0)).abs() < 1e-4,
        "Expected fval=-8.0, got {fval_lp}"
    );
    assert_eq!(exit_lp, 1.0);
}
