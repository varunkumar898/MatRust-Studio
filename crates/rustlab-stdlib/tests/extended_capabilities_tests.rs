//! Integration tests for Simulation, Signal & Image Processing, Machine Learning, and Robotics.

use rustlab_math::Value;

#[test]
fn test_simulation_tf_and_step() {
    // 1st order system step response: H(s) = 5 / (2*s + 1) => K = 5, tau = 2
    // Analytical response: y(t) = K * (1 - e^(-t / tau)) = 5 * (1 - e^(-t / 2))
    // Check at multiple points: t = tau (2.0), t = 2*tau (4.0), t = 5*tau (10.0)
    let script = "sys = tf(5, [2, 1]); [y, t] = step(sys, 12.0);";
    let mut interp = rustlab_stdlib::new_interpreter();
    interp.eval_script(script).unwrap();

    let y_val = interp.env.get("y").unwrap();
    let t_val = interp.env.get("t").unwrap();

    match (y_val, t_val) {
        (Value::Double(y), Value::Double(t)) => {
            let t_slice = t.as_slice();
            let y_slice = y.as_slice();
            // Confirmation of purely analytical closed-form equations:
            // For 1st order systems H(s) = K / (tau*s + 1), step evaluates
            // y(t) = K * (1 - exp(-t / tau)) in closed form at every sampled point.
            // Pointwise error against analytical closed-form formula must be < 1e-12:
            let k_gain = 5.0;
            let tau = 2.0;
            for (&ti, &yi) in t_slice.iter().zip(y_slice.iter()) {
                let y_exact = k_gain * (1.0 - (-ti / tau).exp());
                assert!(
                    (yi - y_exact).abs() < 1e-12,
                    "Exact analytical 1st-order mismatch at t={ti}: got {yi}, expected {y_exact}"
                );
            }
        }
        _ => panic!("Expected double vectors"),
    }

    // 2nd order underdamped system: H(s) = 4 / (s^2 + 2*s + 4)
    // wn = 2, zeta = 0.5 (underdamped, < 1.0)
    // Analytical closed-form: wd = wn*sqrt(1-zeta^2) = sqrt(3) ~ 1.73205
    // phi = atan2(sqrt(1-0.25), 0.5) = pi/3
    // y(t) = 1 - (e^(-zeta*wn*t)/sqrt(1-zeta^2)) * sin(wd*t + phi)
    let script_under = "sys_under = tf(4, [1, 2, 4]); [y_u, t_u] = step(sys_under, 5.0);";
    interp.eval_script(script_under).unwrap();
    let y_u = interp.env.get("y_u").unwrap();
    let t_u = interp.env.get("t_u").unwrap();
    match (y_u, t_u) {
        (Value::Double(y), Value::Double(t)) => {
            let wn = 2.0_f64;
            let zeta = 0.5_f64;
            let wd = wn * (1.0 - zeta * zeta).sqrt();
            let phi = (1.0 - zeta * zeta).sqrt().atan2(zeta);
            let t_slice = t.as_slice();
            let y_slice = y.as_slice();
            // Confirmation of analytical closed-form at all sampled points:
            for (&ti, &yi) in t_slice.iter().zip(y_slice.iter()) {
                let expected = 1.0
                    - (-zeta * wn * ti).exp() / (1.0 - zeta * zeta).sqrt() * (wd * ti + phi).sin();
                assert!(
                    (yi - expected).abs() < 1e-12,
                    "Exact analytical underdamped mismatch at t={ti}: got {yi}, expected {expected}"
                );
            }
        }
        _ => panic!("Expected double vectors"),
    }

    // 2nd order critically damped: H(s) = 9 / (s^2 + 6*s + 9)
    // wn = 3, zeta = 1.0
    // Analytical: y(t) = 1 - (1 + wn*t)*e^(-wn*t)
    let script_crit = "sys_crit = tf(9, [1, 6, 9]); [y_c, t_c] = step(sys_crit, 4.0);";
    interp.eval_script(script_crit).unwrap();
    let y_c = interp.env.get("y_c").unwrap();
    let t_c = interp.env.get("t_c").unwrap();
    match (y_c, t_c) {
        (Value::Double(y), Value::Double(t)) => {
            let wn = 3.0_f64;
            let t_slice = t.as_slice();
            let y_slice = y.as_slice();
            // Confirmation of analytical closed-form at all sampled points:
            for (&ti, &yi) in t_slice.iter().zip(y_slice.iter()) {
                let expected = 1.0 - (1.0 + wn * ti) * (-wn * ti).exp();
                assert!(
                    (yi - expected).abs() < 1e-12,
                    "Exact analytical critically-damped mismatch at t={ti}: got {yi}, expected {expected}"
                );
            }
        }
        _ => panic!("Expected double vectors"),
    }

    // 2nd order overdamped: H(s) = 2 / (s^2 + 3*s + 2) = 2 / ((s+1)(s+2))
    // Roots s1 = -1, s2 = -2
    // Analytical: y(t) = 1 - 2*e^(-t) + e^(-2t)
    let script_over = "sys_over = tf(2, [1, 3, 2]); [y_o, t_o] = step(sys_over, 4.0);";
    interp.eval_script(script_over).unwrap();
    let y_o = interp.env.get("y_o").unwrap();
    let t_o = interp.env.get("t_o").unwrap();
    match (y_o, t_o) {
        (Value::Double(y), Value::Double(t)) => {
            let t_slice = t.as_slice();
            let y_slice = y.as_slice();
            // Confirmation of analytical closed-form at all sampled points:
            for (&ti, &yi) in t_slice.iter().zip(y_slice.iter()) {
                let expected = 1.0 - 2.0 * (-ti).exp() + (-2.0 * ti).exp();
                assert!(
                    (yi - expected).abs() < 1e-12,
                    "Exact analytical overdamped mismatch at t={ti}: got {yi}, expected {expected}"
                );
            }
        }
        _ => panic!("Expected double vectors"),
    }

    // Error on invalid parameters
    assert!(
        interp.eval_script("tf([], [1, 1]);").is_err(),
        "Empty num must error"
    );
    assert!(
        interp.eval_script("tf([1], [0, 0]);").is_err(),
        "All zero den must error"
    );
    assert!(
        interp.eval_script("step(123);").is_err(),
        "Invalid sys must error"
    );
}

#[test]
fn test_signal_filters_windows_and_xcorr() {
    // 1. Moving average filter: b = [1 1]/2, a = 1, x = [1 2 3 4]
    // Expected y = [0.5, 1.5, 2.5, 3.5]
    let script = "y = filter([0.5, 0.5], 1, [1, 2, 3, 4]);";
    let mut interp = rustlab_stdlib::new_interpreter();
    interp.eval_script(script).unwrap();
    let y_val = interp.env.get("y").unwrap();
    match y_val {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[0.5, 1.5, 2.5, 3.5]);
        }
        _ => panic!("Expected double vector"),
    }

    // 2. Impulse response of FIR filter: b = [0.2, 0.5, 0.3], a = 1
    // Input impulse delta = [1, 0, 0, 0, 0] => output must match b exactly
    let script_imp = "h = filter([0.2, 0.5, 0.3], 1, [1, 0, 0, 0, 0]);";
    interp.eval_script(script_imp).unwrap();
    let h_val = interp.env.get("h").unwrap();
    match h_val {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[0.2, 0.5, 0.3, 0.0, 0.0]);
        }
        _ => panic!("Expected double vector"),
    }

    // 3. FIR / Window symmetry
    let script_win = "w_ham = hamming(7); w_han = hann(7); w_blk = blackman(7);";
    interp.eval_script(script_win).unwrap();
    for win_name in &["w_ham", "w_han", "w_blk"] {
        let win = interp.env.get(win_name).unwrap();
        match win {
            Value::Double(d) => {
                let s = d.as_slice();
                assert_eq!(s.len(), 7);
                for i in 0..3 {
                    assert!(
                        (s[i] - s[6 - i]).abs() < 1e-12,
                        "Window {} symmetry violated at index {}: {} vs {}",
                        win_name,
                        i,
                        s[i],
                        s[6 - i]
                    );
                }
            }
            _ => panic!("Expected double vector for window"),
        }
    }

    // 4. Cross-correlation / Auto-correlation vs direct sum:
    // x = [1, 2, 3], length N = 3 => xcorr length = 2*N - 1 = 5
    // Analytical direct sum:
    // lag -2: 1*3 = 3
    // lag -1: 1*2 + 2*3 = 8
    // lag  0: 1*1 + 2*2 + 3*3 = 14
    // lag +1: 2*1 + 3*2 = 8
    // lag +2: 3*1 = 3
    let script_xcorr = "c = xcorr([1, 2, 3]);";
    interp.eval_script(script_xcorr).unwrap();
    let c_val = interp.env.get("c").unwrap();
    match c_val {
        Value::Double(d) => {
            assert_eq!(d.as_slice(), &[3.0, 8.0, 14.0, 8.0, 3.0]);
        }
        _ => panic!("Expected double vector"),
    }

    // 5. Zero-phase filtering verification:
    // filtfilt on symmetric impulse: response must remain symmetric around center
    let script_ff = "y_ff = filtfilt([0.25, 0.5, 0.25], 1, [0, 0, 1, 0, 0]);";
    interp.eval_script(script_ff).unwrap();
    let ff_val = interp.env.get("y_ff").unwrap();
    match ff_val {
        Value::Double(d) => {
            let s = d.as_slice();
            assert_eq!(s.len(), 5);
            // Symmetry around index 2 (center): s[0] == s[4] and s[1] == s[3]
            assert!(
                (s[0] - s[4]).abs() < 1e-12,
                "Zero-phase symmetry failed: {} vs {}",
                s[0],
                s[4]
            );
            assert!(
                (s[1] - s[3]).abs() < 1e-12,
                "Zero-phase symmetry failed: {} vs {}",
                s[1],
                s[3]
            );
        }
        _ => panic!("Expected double vector"),
    }
}

#[test]
fn test_image_processing_rgb2gray_and_filters() {
    // 2x2 RGB image where all pixels are [100, 100, 100]
    // 0.2989*100 + 0.5870*100 + 0.1140*100 = 100.0
    let mut interp = rustlab_stdlib::new_interpreter();
    let script = "RGB = ones(2, 2, 3) * 100; gray = rgb2gray(RGB);";
    interp.eval_script(script).unwrap();
    let gray_val = interp.env.get("gray").unwrap();
    match gray_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[2, 2]);
            for &v in d.as_slice() {
                // 0.2989*100 + 0.5870*100 + 0.1140*100 = 99.99
                assert!((v - 99.99).abs() < 1e-10);
            }
        }
        _ => panic!("Expected double matrix"),
    }

    // imresize: 2x2 scaled by 2 => 4x4
    let script_resize = "J = imresize(ones(2, 2), 2);";
    interp.eval_script(script_resize).unwrap();
    let j_val = interp.env.get("J").unwrap();
    match j_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[4, 4]);
        }
        _ => panic!("Expected 4x4 matrix"),
    }
}

#[test]
fn test_machine_learning_fitlm_predict_kmeans() {
    // 1. Closed-form OLS multi-variable regression:
    // y = 5.0 + 3.0*x1 - 2.0*x2
    // X = [1 2; 2 1; 3 4; 4 3];
    // y = [5+3-4; 5+6-2; 5+9-8; 5+12-6] = [4; 9; 6; 11];
    let script_ols = "X = [1 2; 2 1; 3 4; 4 3]; y = [4; 9; 6; 11]; model = fitlm(X, y); coefs = model.Coefficients;";
    let mut interp = rustlab_stdlib::new_interpreter();
    interp.eval_script(script_ols).unwrap();
    let coefs = interp.env.get("coefs").unwrap();
    match coefs {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 1]);
            let s = d.as_slice();
            assert!((s[0] - 5.0).abs() < 1e-10, "Intercept mismatch: {}", s[0]);
            assert!((s[1] - 3.0).abs() < 1e-10, "Beta1 mismatch: {}", s[1]);
            assert!((s[2] - (-2.0)).abs() < 1e-10, "Beta2 mismatch: {}", s[2]);
        }
        _ => panic!("Expected double coefficients column vector"),
    }

    // Predict on new unseen points: [5 2] => 5 + 3*5 - 2*2 = 16
    let script_pred = "y_hat = predict(model, [5 2]);";
    interp.eval_script(script_pred).unwrap();
    let y_hat = interp.env.get("y_hat").unwrap();
    assert!((y_hat.as_scalar_f64().unwrap() - 16.0).abs() < 1e-10);

    // 2. Deterministic k-means on well-separated clusters with exact known centres and assignments:
    // Cluster A points: 0, 1 -> Known analytical centre = (0 + 1) / 2 = 0.5
    // Cluster B points: 100, 101 -> Known analytical centre = (100 + 101) / 2 = 100.5
    let script_kmeans =
        "data = [0; 1; 100; 101]; [idx1, C1] = kmeans(data, 2); [idx2, C2] = kmeans(data, 2);";
    interp.eval_script(script_kmeans).unwrap();
    let idx1 = interp.env.get("idx1").unwrap();
    let idx2 = interp.env.get("idx2").unwrap();
    let c1 = interp.env.get("C1").unwrap();
    match (idx1, idx2, c1) {
        (Value::Double(d1), Value::Double(d2), Value::Double(c)) => {
            assert_eq!(
                d1.as_slice(),
                d2.as_slice(),
                "K-means must be deterministic across calls"
            );
            // Assert known cluster assignments: points 0, 1 together; points 100, 101 together
            assert_eq!(d1.as_slice()[0], d1.as_slice()[1]);
            assert_eq!(d1.as_slice()[2], d1.as_slice()[3]);
            assert_ne!(d1.as_slice()[0], d1.as_slice()[2]);

            // Assert exact known cluster centres: {0.5, 100.5}
            let centers = c.as_slice();
            assert_eq!(centers.len(), 2, "Must produce 2 cluster centers");
            let mut sorted_c = centers.to_vec();
            sorted_c.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert!(
                (sorted_c[0] - 0.5).abs() < 1e-12,
                "Known cluster centre 0.5 mismatch: {}",
                sorted_c[0]
            );
            assert!(
                (sorted_c[1] - 100.5).abs() < 1e-12,
                "Known cluster centre 100.5 mismatch: {}",
                sorted_c[1]
            );

            // Verify point assignments correspond exactly to the respective center
            let cluster_0 = (d1.as_slice()[0] - 1.0) as usize;
            let cluster_100 = (d1.as_slice()[2] - 1.0) as usize;
            assert!(
                (centers[cluster_0] - 0.5).abs() < 1e-12,
                "Point [0, 1] must map to center 0.5"
            );
            assert!(
                (centers[cluster_100] - 100.5).abs() < 1e-12,
                "Point [100, 101] must map to center 100.5"
            );
        }
        _ => panic!("Expected double indices and centers"),
    }

    // 3. Hand-checked confusion matrix:
    // actual = [1, 1, 2, 2, 3, 3]
    // predicted = [1, 2, 2, 2, 1, 3]
    // Hand-derived count matrix (3 classes):
    // actual 1: pred 1 -> 1, pred 2 -> 1, pred 3 -> 0
    // actual 2: pred 1 -> 0, pred 2 -> 2, pred 3 -> 0
    // actual 3: pred 1 -> 1, pred 2 -> 0, pred 3 -> 1
    // Matrix in row/col:
    // [1 1 0; 0 2 0; 1 0 1]
    // In column-major slice:
    // Col 0: [1, 0, 1]
    // Col 1: [1, 2, 0]
    // Col 2: [0, 0, 1]
    let script_cm = "cm = confusionmat([1; 1; 2; 2; 3; 3], [1; 2; 2; 2; 1; 3]);";
    interp.eval_script(script_cm).unwrap();
    let cm_val = interp.env.get("cm").unwrap();
    match cm_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 3]);
            assert_eq!(d.as_slice(), &[1.0, 0.0, 1.0, 1.0, 2.0, 0.0, 0.0, 0.0, 1.0]);
        }
        _ => panic!("Expected 3x3 confusion matrix"),
    }
}

#[test]
fn test_robotics_rotations_and_kinematics() {
    let mut interp = rustlab_stdlib::new_interpreter();
    // rotz(0) => eye(3), transl(1, 2, 3)
    let script_rot = "R = rotz(0); T = transl(1, 2, 3);";
    interp.eval_script(script_rot).unwrap();
    let r_val = interp.env.get("R").unwrap();
    match r_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 3]);
            assert_eq!(d.as_slice(), &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        }
        _ => panic!("Expected 3x3 matrix"),
    }

    // FK / IK round-trip:
    // For joint angles q = [0.4, 0.6], link lengths l1 = 1.2, l2 = 0.8:
    // Compute p = fkine_2link(l1, l2, q1, q2)
    // Invert: q_sol = ikine_2link(l1, l2, p(1), p(2))
    // Recompute: p_check = fkine_2link(l1, l2, q_sol(1), q_sol(2))
    // Assert p_check matches p within 1e-10
    let script_roundtrip = "l1 = 1.2; l2 = 0.8;
                           p_orig = fkine_2link(l1, l2, 0.4, 0.6);
                           q_inv = ikine_2link(l1, l2, p_orig(1), p_orig(2));
                           p_recon = fkine_2link(l1, l2, q_inv(1), q_inv(2));
                           diff_pos = max(abs(p_orig - p_recon));";
    interp.eval_script(script_roundtrip).unwrap();
    let diff_pos = interp.env.get("diff_pos").unwrap().as_scalar_f64().unwrap();
    assert!(
        diff_pos < 1e-10,
        "FK/IK round-trip residual {diff_pos:e} must be < 1e-10"
    );

    // Unreachable workspace error
    let err_ik = interp.eval_script("ikine_2link(1.0, 1.0, 3.0, 0.0);");
    assert!(err_ik.is_err(), "Target outside robot reach must error");

    // Quintic joint trajectory: jtraj(0, 10, 101)
    // Check boundary velocity = 0 and acceleration = 0
    let script_jtraj = "q_tr = jtraj(0, 10, 101);";
    interp.eval_script(script_jtraj).unwrap();
    let q_tr = interp.env.get("q_tr").unwrap();
    match q_tr {
        Value::Double(d) => {
            let s = d.as_slice();
            assert_eq!(s.len(), 101);
            assert_eq!(s[0], 0.0);
            assert_eq!(s[100], 10.0);

            let dt = 1.0 / 100.0;
            // Velocity at boundary: (q[1] - q[0])/dt -> ~0
            let v_start = (s[1] - s[0]) / dt;
            let v_end = (s[100] - s[99]) / dt;
            assert!(v_start < 2e-2, "Start velocity must be ~0, got {v_start}");
            assert!(v_end < 2e-2, "End velocity must be ~0, got {v_end}");

            // Boundary zero-acceleration verification:
            // Since s(t) = 10*t^3 - 15*t^4 + 6*t^5, s'(0) = 0, s'(1) = 0,
            // s''(0) = 0, and s''(1) = 0.
            // Absence of linear (v0*t) and quadratic (0.5*a0*t^2) terms means
            // displacement at step 1 is O(dt^3) = 1e-4:
            assert!(
                s[1] < 1e-3,
                "Start acceleration must be zero (O(t^3) growth)"
            );
            assert!(
                (10.0 - s[99]) < 1e-3,
                "End acceleration must be zero (O((1-t)^3) growth)"
            );
        }
        _ => panic!("Expected 101x1 joint trajectory vector"),
    }
}
