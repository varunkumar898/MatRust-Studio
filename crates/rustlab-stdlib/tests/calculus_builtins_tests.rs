//! Unit tests for Phase F17 Task MG1 Calculus Built-ins:
//! `del2`, `divergence`, `curl`, `integral`, `integral2`, `integral3`, `quad`, `quadgk`.

use rustlab_math::Value;
use rustlab_stdlib::new_interpreter;

#[test]
fn test_del2_1d_and_2d() {
    let mut interp = new_interpreter();
    let script = r#"
        % 1D vector test: u = x^2, d2u/dx2 = 2. In 1D, del2 computes 0.5 * d2u/dx2 = 1.0
        x = [0 1 2 3 4];
        u = x.^2;
        L1 = del2(u);

        % 2D surface test: u = x^2 + y^2 => Laplacian is 2 + 2 = 4 => del2 is 4/4 = 1.0
        [X, Y] = meshgrid(-2:2, -2:2);
        Z = X.^2 + Y.^2;
        L2 = del2(Z);

        % Spacing test with hx=2, hy=2
        Z_scaled = (2*X).^2 + (2*Y).^2;
        L2_scaled = del2(Z_scaled, 2.0, 2.0);
    "#;
    interp.eval_script(script).unwrap();

    let l1_val = interp.env.get("L1").unwrap();
    if let Value::Double(d) = l1_val {
        let s = d.as_slice();
        assert!((s[1] - 1.0).abs() < 1e-12);
        assert!((s[2] - 1.0).abs() < 1e-12);
        assert!((s[3] - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array for L1");
    }

    let l2_val = interp.env.get("L2").unwrap();
    if let Value::Double(d) = l2_val {
        // (3,3) in 5x5 is row 2, col 2 -> index 2 + 2*5 = 12
        let s = d.as_slice();
        assert!((s[12] - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array for L2");
    }

    let l2_scaled_val = interp.env.get("L2_scaled").unwrap();
    if let Value::Double(d) = l2_scaled_val {
        let s = d.as_slice();
        assert!((s[12] - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array for L2_scaled");
    }
}

#[test]
fn test_divergence_and_curl_2d() {
    let mut interp = new_interpreter();
    let script = r#"
        % Test field F = [-y, x]
        % div(F) = d(-y)/dx + d(x)/dy = 0 + 0 = 0
        % vor = dv/dx - du/dy = 1 - (-1) = 2 => cav = 0.5 * vor = 1.0
        [X, Y] = meshgrid(-2:2, -2:2);
        U = -Y;
        V = X;

        div = divergence(X, Y, U, V);
        [cav, mag] = curl(X, Y, U, V);
        div_def = divergence(U, V);
        cav_def = curl(U, V);
    "#;
    interp.eval_script(script).unwrap();

    let div_val = interp.env.get("div").unwrap();
    if let Value::Double(d) = div_val {
        for &v in d.as_slice() {
            assert!(v.abs() < 1e-12);
        }
    } else {
        panic!("Expected double array for div");
    }

    let cav_val = interp.env.get("cav").unwrap();
    if let Value::Double(d) = cav_val {
        // Center interior point (3,3) -> index 12 in 5x5
        let s = d.as_slice();
        assert!((s[12] - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array for cav");
    }

    let cav_def_val = interp.env.get("cav_def").unwrap();
    if let Value::Double(d) = cav_def_val {
        let s = d.as_slice();
        assert!((s[12] - 1.0).abs() < 1e-12);
    } else {
        panic!("Expected double array for cav_def");
    }
}

#[test]
fn test_quadrature_1d_integral_quad_quadgk() {
    let mut interp = new_interpreter();
    let script = r#"
        % Integral of x^2 from 0 to 1 is 1/3
        q1 = integral(@(x) x.^2, 0, 1);
        q2 = quad(@(x) x.^2, 0, 1);
        q3 = quadgk(@(x) x.^2, 0, 1);

        % Integral of sin(x) from 0 to pi is 2.0
        q_sin = integral(@(x) sin(x), 0, 3.141592653589793);

        % Integral of exp(x) from 0 to 1 is e - 1
        q_exp = integral(@(x) exp(x), 0, 1);
    "#;
    interp.eval_script(script).unwrap();

    let q1 = interp.env.get("q1").unwrap().as_scalar_f64().unwrap();
    assert!((q1 - 1.0 / 3.0).abs() < 1e-7);

    let q2 = interp.env.get("q2").unwrap().as_scalar_f64().unwrap();
    assert!((q2 - 1.0 / 3.0).abs() < 1e-5);

    let q3 = interp.env.get("q3").unwrap().as_scalar_f64().unwrap();
    assert!((q3 - 1.0 / 3.0).abs() < 1e-8);

    let q_sin = interp.env.get("q_sin").unwrap().as_scalar_f64().unwrap();
    assert!((q_sin - 2.0).abs() < 1e-6);

    let q_exp = interp.env.get("q_exp").unwrap().as_scalar_f64().unwrap();
    assert!((q_exp - (std::f64::consts::E - 1.0)).abs() < 1e-6);
}

#[test]
fn test_quadrature_2d_and_3d() {
    let mut interp = new_interpreter();
    let script = r#"
        % 2D integral of f(x,y) = x*y over [0, 1] x [0, 2]
        % \int_0^2 \int_0^1 x*y dx dy = (1/2) * (4/2) = 1.0
        q2d = integral2(@(x, y) x .* y, 0, 1, 0, 2);

        % 3D integral of f(x,y,z) = 1 over [0, 1] x [0, 2] x [0, 3] = Volume = 6.0
        q3d = integral3(@(x, y, z) 1.0, 0, 1, 0, 2, 0, 3);
    "#;
    interp.eval_script(script).unwrap();

    let q2d = interp.env.get("q2d").unwrap().as_scalar_f64().unwrap();
    assert!((q2d - 1.0).abs() < 1e-5);

    let q3d = interp.env.get("q3d").unwrap().as_scalar_f64().unwrap();
    assert!((q3d - 6.0).abs() < 1e-4);
}

#[test]
fn test_odeset_options_struct() {
    let mut interp = new_interpreter();
    interp
        .eval_script(
            r#"
            opts = odeset('RelTol', 1e-5, 'AbsTol', 1e-7, 'MaxStep', 0.05);
            has_rel = isfield(opts, 'RelTol');
            rel_val = opts.RelTol;
            abs_val = opts.AbsTol;
            max_val = opts.MaxStep;
            "#,
        )
        .unwrap();

    let has_rel = interp.env.get("has_rel").unwrap();
    assert_eq!(has_rel.as_scalar_f64().unwrap(), 1.0);

    let rel_val = interp.env.get("rel_val").unwrap();
    assert!((rel_val.as_scalar_f64().unwrap() - 1e-5).abs() < 1e-12);

    let abs_val = interp.env.get("abs_val").unwrap();
    assert!((abs_val.as_scalar_f64().unwrap() - 1e-7).abs() < 1e-14);

    let max_val = interp.env.get("max_val").unwrap();
    assert!((max_val.as_scalar_f64().unwrap() - 0.05).abs() < 1e-12);
}

#[test]
fn test_ode45_and_ode23_with_options() {
    let mut interp = new_interpreter();
    interp
        .eval_script(
            r#"
            f = @(t, y) -2 * y;
            opts = odeset('RelTol', 1e-6, 'AbsTol', 1e-8, 'MaxStep', 0.1);
            [t45, y45] = ode45(f, [0, 1], 1.0, opts);
            [t23, y23] = ode23(f, [0, 1], 1.0, opts);
            
            y45_final = y45(end);
            y23_final = y23(end);
            exact_final = exp(-2.0);
            err45 = abs(y45_final - exact_final);
            err23 = abs(y23_final - exact_final);
            "#,
        )
        .unwrap();

    let err45 = interp.env.get("err45").unwrap().as_scalar_f64().unwrap();
    let err23 = interp.env.get("err23").unwrap().as_scalar_f64().unwrap();

    assert!(err45 < 1e-4, "ode45 error too large: {err45}");
    assert!(err23 < 1e-3, "ode23 error too large: {err23}");
}

#[test]
fn test_ode15s_stiff_system() {
    let mut interp = new_interpreter();
    // Stiff decay: y' = -50*y, y(0) = 1.0 -> exact y(t) = exp(-50*t)
    interp
        .eval_script(
            r#"
            f_stiff = @(t, y) -50.0 * y;
            opts = odeset('RelTol', 1e-4, 'AbsTol', 1e-6);
            [t, y] = ode15s(f_stiff, [0, 0.2], 1.0, opts);
            y_final = y(end);
            y_exact = exp(-50.0 * 0.2);
            err_stiff = abs(y_final - y_exact);
            "#,
        )
        .unwrap();

    let err_stiff = interp
        .env
        .get("err_stiff")
        .unwrap()
        .as_scalar_f64()
        .unwrap();
    assert!(
        err_stiff < 5e-3,
        "ode15s stiff error too large: {err_stiff}"
    );
}

#[test]
fn test_ode15s_vanderpol_oscillator() {
    let mut interp = new_interpreter();
    // 2D Van der Pol oscillator: y1' = y2, y2' = 5 * (1 - y1^2) * y2 - y1
    interp
        .eval_script(
            r#"
            vdp = @(t, y) [y(2); 5.0 * (1.0 - y(1)^2) * y(2) - y(1)];
            [t, y] = ode15s(vdp, [0, 2.0], [2.0; 0.0]);
            sz = size(y);
            rows = sz(1);
            cols = sz(2);
            y1_end = y(end, 1);
            y2_end = y(end, 2);
            "#,
        )
        .unwrap();

    let rows = interp.env.get("rows").unwrap().as_scalar_f64().unwrap();
    let cols = interp.env.get("cols").unwrap().as_scalar_f64().unwrap();
    assert!(rows > 5.0);
    assert_eq!(cols, 2.0);

    let y1_end = interp.env.get("y1_end").unwrap().as_scalar_f64().unwrap();
    let y2_end = interp.env.get("y2_end").unwrap().as_scalar_f64().unwrap();
    assert!(y1_end.is_finite());
    assert!(y2_end.is_finite());
}

#[test]
fn test_expm_and_interp1_integration() {
    let mut interp = new_interpreter();
    interp
        .eval_script(
            r#"
            % Matrix exponential for 90-degree rotation generator
            theta = pi / 2;
            A = [0, -theta; theta, 0];
            R = expm(A);
            % R should be approx [0, -1; 1, 0]
            r11 = R(1, 1);
            r12 = R(1, 2);
            r21 = R(2, 1);
            r22 = R(2, 2);

            % 1D Interpolation
            x = [0, 1, 2, 3];
            v = [0, 10, 20, 30];
            xq = [0.5, 1.5, 2.5];
            vq = interp1(x, v, xq);
            "#,
        )
        .unwrap();

    let r11 = interp.env.get("r11").unwrap().as_scalar_f64().unwrap();
    let r12 = interp.env.get("r12").unwrap().as_scalar_f64().unwrap();
    let r21 = interp.env.get("r21").unwrap().as_scalar_f64().unwrap();
    let r22 = interp.env.get("r22").unwrap().as_scalar_f64().unwrap();

    assert!(r11.abs() < 1e-10);
    assert!((r12 - (-1.0)).abs() < 1e-10);
    assert!((r21 - 1.0).abs() < 1e-10);
    assert!(r22.abs() < 1e-10);

    let vq = interp.env.get("vq").unwrap();
    if let Value::Double(d) = vq {
        let vq_slice = d.as_slice();
        assert!((vq_slice[0] - 5.0).abs() < 1e-10);
        assert!((vq_slice[1] - 15.0).abs() < 1e-10);
        assert!((vq_slice[2] - 25.0).abs() < 1e-10);
    } else {
        panic!("Expected double array for vq");
    }
}

#[test]
fn test_signal_deconv_and_freqz() {
    let mut interp = new_interpreter();
    interp
        .eval_script(
            r#"
            % Deconvolution: b = conv(a, q) + r
            a = [1, 2, 1];
            q_exact = [1, 3, 2];
            b = [1, 5, 9, 7, 2];
            [q, r] = deconv(b, a);

            % freqz of 2-tap moving average FIR filter b = [0.5, 0.5], a = 1
            [h, w] = freqz([0.5, 0.5], 1.0, 128);
            h_dc = abs(h(1));
            h_nyq = abs(h(end));
            "#,
        )
        .unwrap();

    let q = interp.env.get("q").unwrap();
    if let Value::Double(d) = q {
        let s = d.as_slice();
        assert_eq!(s.len(), 3);
        assert!((s[0] - 1.0).abs() < 1e-12);
        assert!((s[1] - 3.0).abs() < 1e-12);
        assert!((s[2] - 2.0).abs() < 1e-12);
    } else {
        panic!("Expected double vector for q");
    }

    let r = interp.env.get("r").unwrap();
    if let Value::Double(d) = r {
        for &val in d.as_slice() {
            assert!(val.abs() < 1e-12);
        }
    } else {
        panic!("Expected double vector for r");
    }

    let h_dc = interp.env.get("h_dc").unwrap().as_scalar_f64().unwrap();
    let h_nyq = interp.env.get("h_nyq").unwrap().as_scalar_f64().unwrap();
    assert!(
        (h_dc - 1.0).abs() < 1e-6,
        "Expected H(DC) = 1.0, got {h_dc}"
    );
    assert!(
        h_nyq < 0.02,
        "Expected H near Nyquist to be attenuated (< 0.02), got {h_nyq}"
    );
}

#[test]
fn test_signal_residue() {
    let mut interp = new_interpreter();
    // B(s) = s, A(s) = (s+1)(s+2) = s^2 + 3s + 2
    // B/A = -1/(s+1) + 2/(s+2) -> residues [-1, 2], poles [-1, -2]
    interp
        .eval_script(
            r#"
            b = [1, 0];
            a = [1, 3, 2];
            [r, p, k] = residue(b, a);
            p1 = p(1);
            p2 = p(2);
            r1 = r(1);
            r2 = r(2);
            "#,
        )
        .unwrap();

    let r1 = interp.env.get("r1").unwrap();
    let r2 = interp.env.get("r2").unwrap();
    let p1 = interp.env.get("p1").unwrap();
    let p2 = interp.env.get("p2").unwrap();

    // Check residues match poles (r / (s - p))
    // Poles are -1 and -2 in some order
    let (p1_re, p2_re) = match (p1, p2) {
        (Value::ComplexDouble(cd1), Value::ComplexDouble(cd2)) => {
            (cd1.as_slice()[0].re, cd2.as_slice()[0].re)
        }
        _ => panic!("Expected complex double array for poles"),
    };
    let (r1_re, r2_re) = match (r1, r2) {
        (Value::ComplexDouble(cd1), Value::ComplexDouble(cd2)) => {
            (cd1.as_slice()[0].re, cd2.as_slice()[0].re)
        }
        _ => panic!("Expected complex double array for residues"),
    };

    if (p1_re - (-1.0)).abs() < 1e-5 {
        assert!((r1_re - (-1.0)).abs() < 1e-5);
        assert!((p2_re - (-2.0)).abs() < 1e-5);
        assert!((r2_re - 2.0).abs() < 1e-5);
    } else {
        assert!((p1_re - (-2.0)).abs() < 1e-5);
        assert!((r1_re - 2.0).abs() < 1e-5);
        assert!((p2_re - (-1.0)).abs() < 1e-5);
        assert!((r2_re - (-1.0)).abs() < 1e-5);
    }
}

#[test]
fn test_interpolation_spline_and_pchip() {
    let mut interp = new_interpreter();
    interp
        .eval_script(
            r#"
            % Cubic test: y = x^3
            x = [0, 1, 2, 3];
            y = [0, 1, 8, 27];
            xq = [0.5, 1.5, 2.5];
            y_spline = spline(x, y, xq);
            y_pchip = pchip(x, y, xq);

            % Monotone step test: PCHIP must preserve monotonicity
            x_step = [0, 1, 2, 3, 4];
            y_step = [0, 0, 1, 1, 1];
            xq_step = [0.5, 1.5, 2.5, 3.5];
            y_pchip_step = pchip(x_step, y_step, xq_step);
            "#,
        )
        .unwrap();

    let y_spline = interp.env.get("y_spline").unwrap();
    if let Value::Double(d) = y_spline {
        let s = d.as_slice();
        // At x = 1.5, exact x^3 is 3.375
        assert!((s[1] - 3.375).abs() < 0.5);
    } else {
        panic!("Expected double array for y_spline");
    }

    let y_pchip_step = interp.env.get("y_pchip_step").unwrap();
    if let Value::Double(d) = y_pchip_step {
        for &val in d.as_slice() {
            // PCHIP must never overshoot outside [0, 1]
            assert!(
                (-1e-12..=1.0 + 1e-12).contains(&val),
                "PCHIP overshoot: {val}"
            );
        }
    } else {
        panic!("Expected double array for y_pchip_step");
    }
}

#[test]
fn test_spectral_pwelch_and_spectrogram() {
    let mut interp = new_interpreter();
    interp
        .eval_script(
            r#"
            Fs = 1000.0;
            t = 0:(1/Fs):0.5;
            % 50 Hz sine wave
            x = sin(2 * pi * 50 * t);
            [pxx, f] = pwelch(x, 128, 64, 128, Fs);
            [s, f_stft, t_stft] = spectrogram(x, 128, 64, 128, Fs);

            sz_s = size(s);
            s_rows = sz_s(1);
            s_cols = sz_s(2);
            "#,
        )
        .unwrap();

    let pxx = interp.env.get("pxx").unwrap();
    let f = interp.env.get("f").unwrap();
    if let (Value::Double(d_p), Value::Double(d_f)) = (pxx, f) {
        let p_slice = d_p.as_slice();
        let f_slice = d_f.as_slice();
        // Find peak frequency index
        let mut max_idx = 0;
        let mut max_pwr = 0.0;
        for (i, &pwr) in p_slice.iter().enumerate() {
            if pwr > max_pwr {
                max_pwr = pwr;
                max_idx = i;
            }
        }
        let peak_freq = f_slice[max_idx];
        assert!(
            (peak_freq - 50.0).abs() < 15.0,
            "Expected peak near 50 Hz, got {peak_freq}"
        );
    } else {
        panic!("Expected double vectors for pxx and f");
    }

    let s_rows = interp.env.get("s_rows").unwrap().as_scalar_f64().unwrap();
    let s_cols = interp.env.get("s_cols").unwrap().as_scalar_f64().unwrap();
    assert_eq!(s_rows, 65.0); // 128/2 + 1 = 65 frequencies
    assert!(s_cols >= 4.0);
}
