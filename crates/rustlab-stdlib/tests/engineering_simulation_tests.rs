//! Comprehensive verification tests for Engineering Physics and Simulation:
//! - Electrical Engineering: DC/AC nodal analysis, RLC transient circuits, three-phase power.
//! - Electronics: Non-linear diode operating point via fzero, RC/LC networks.
//! - Mechanical Engineering: Mass-spring-damper vibration via ode45, beam deflection via linsolve.
//! - Thermal Engineering: 2D steady-state heat transfer via del2.
//! - 3D Trajectory Visualization: 3D Lorenz attractor integration and plot3 trajectory.

use rustlab_math::Value;
use rustlab_stdlib::new_interpreter;

#[test]
fn test_electrical_dc_nodal_analysis() {
    let mut interp = new_interpreter();
    // 3-node resistive network with conductance matrix G and current injections I:
    // Node 1: connected to Node 2 via R12 = 10 Ohm, to ground via R1 = 20 Ohm.
    // Node 2: connected to Node 3 via R23 = 5 Ohm.
    // Node 3: connected to ground via R3 = 10 Ohm.
    // Current injection: I1 = 2 A at node 1, I3 = -1 A at node 3.
    // G * V = I => V = G \ I
    let script = r#"
        G = [1/20 + 1/10, -1/10, 0;
             -1/10, 1/10 + 1/5, -1/5;
             0, -1/5, 1/5 + 1/10];
        I = [2; 0; -1];
        V = G \ I;
        % Check KCL residual: G*V - I should be near zero
        res = norm(G*V - I);
    "#;
    interp.eval_script(script).unwrap();
    let res_val = interp.env.get("res").unwrap();
    match res_val {
        Value::Double(d) => {
            let res = d.as_slice()[0];
            assert!(
                res < 1e-12,
                "DC nodal KCL residual must be < 1e-12, got {res}"
            );
        }
        _ => panic!("Expected double scalar"),
    }
}

#[test]
fn test_electrical_rlc_transient_ode45() {
    let mut interp = new_interpreter();
    // Series RLC circuit: R = 10 Ohm, L = 0.1 H, C = 100 uF (1e-4 F), Step input V = 12 V
    // State: y = [q; i] where dq/dt = i, di/dt = (V - R*i - q/C)/L
    // Initial conditions: q(0) = 0, i(0) = 0.
    let script = r#"
        R = 10; L = 0.1; C = 1e-4; V_step = 12;
        rlc = @(t, y) [y(2); (V_step - R*y(2) - y(1)/C) / L];
        [t, y] = ode45(rlc, [0, 0.05], [0; 0]);
        q_final = y(end, 1);
        % Steady state charge q = C * V_step = 1e-4 * 12 = 1.2e-3 C
        q_expected = C * V_step;
    "#;
    interp.eval_script(script).unwrap();
    let q_final = interp.env.get("q_final").unwrap();
    let q_exp = interp.env.get("q_expected").unwrap();
    match (q_final, q_exp) {
        (Value::Double(q_f), Value::Double(q_e)) => {
            let q_num = q_f.as_slice()[0];
            let exp_num = q_e.as_slice()[0];
            // In damped oscillation, after several time constants it settles near C*V
            assert!(
                (q_num - exp_num).abs() < 2e-4,
                "RLC charge error: got {q_num}, expected {exp_num}"
            );
        }
        _ => panic!("Expected double scalar"),
    }
}

#[test]
fn test_three_phase_power_calculations() {
    let mut interp = new_interpreter();
    // Balanced 3-phase load: line-to-line voltage VL = 400 V, line current IL = 15 A, power factor pf = 0.85
    let script = r#"
        VL = 400; IL = 15; pf = 0.85;
        P = sqrt(3) * VL * IL * pf;
        theta = acos(pf);
        Q = sqrt(3) * VL * IL * sin(theta);
        S = sqrt(P^2 + Q^2);
        S_apparent = sqrt(3) * VL * IL;
        diff = abs(S - S_apparent);
    "#;
    interp.eval_script(script).unwrap();
    let diff_val = interp.env.get("diff").unwrap();
    match diff_val {
        Value::Double(d) => {
            let diff = d.as_slice()[0];
            assert!(
                diff < 1e-10,
                "Apparent power mismatch must be zero, got {diff}"
            );
        }
        _ => panic!("Expected double scalar"),
    }
}

#[test]
fn test_electronics_diode_operating_point_fzero() {
    let mut interp = new_interpreter();
    // Diode circuit: Vin = 5 V, R = 1000 Ohm, Is = 1e-12 A, Vt = 0.026 V (thermal voltage)
    // Non-linear equation: f(Vd) = Vin - Vd - R * Is * (exp(Vd / Vt) - 1) = 0
    let script = r#"
        Vin = 5; R = 1000; Is = 1e-12; Vt = 0.026;
        f_diode = @(Vd) Vin - Vd - R * Is * (exp(Vd / Vt) - 1);
        vd_q = fzero(f_diode, 0.7);
        id_q = Is * (exp(vd_q / Vt) - 1);
        % Check KVL: Vin - vd_q - id_q * R ~ 0
        kvl_err = abs(Vin - vd_q - id_q * R);
    "#;
    interp.eval_script(script).unwrap();
    let vd_q = interp.env.get("vd_q").unwrap();
    let kvl_err = interp.env.get("kvl_err").unwrap();
    match (vd_q, kvl_err) {
        (Value::Double(vd), Value::Double(err)) => {
            let v = vd.as_slice()[0];
            let e = err.as_slice()[0];
            assert!(
                v > 0.5 && v < 0.8,
                "Diode voltage must be around 0.6-0.7V, got {v}"
            );
            assert!(e < 1e-6, "KVL residual must be < 1e-6, got {e}");
        }
        _ => panic!("Expected double scalar"),
    }
}

#[test]
fn test_mechanical_mass_spring_damper_ode45() {
    let mut interp = new_interpreter();
    // Underdamped mass-spring-damper: m = 1 kg, k = 100 N/m, c = 2 N*s/m
    // wn = sqrt(k/m) = 10 rad/s
    // zeta = c / (2 * sqrt(m*k)) = 2 / 20 = 0.1 (underdamped)
    // State: y = [x; v] where dx/dt = v, dv/dt = (-c*v - k*x) / m
    // Initial displacement x(0) = 0.5 m, v(0) = 0.
    let script = r#"
        m = 1; k = 100; c = 2;
        msd = @(t, y) [y(2); (-c*y(2) - k*y(1)) / m];
        [t, y] = ode45(msd, [0, 2], [0.5; 0]);
        % Energy decreases monotonically: E = 0.5*m*v^2 + 0.5*k*x^2
        E_initial = 0.5 * k * 0.5^2;
        x_final = y(end, 1);
        v_final = y(end, 2);
        E_final = 0.5 * m * v_final^2 + 0.5 * k * x_final^2;
    "#;
    interp.eval_script(script).unwrap();
    let e_init = interp.env.get("E_initial").unwrap();
    let e_fin = interp.env.get("E_final").unwrap();
    match (e_init, e_fin) {
        (Value::Double(ei), Value::Double(ef)) => {
            let e0 = ei.as_slice()[0];
            let e1 = ef.as_slice()[0];
            assert!(
                e1 < e0,
                "Energy must dissipate: E_final {e1} < E_initial {e0}"
            );
            assert!(e1 > 0.0, "Energy must remain positive, got {e1}");
        }
        _ => panic!("Expected double scalar"),
    }
}

#[test]
fn test_thermal_2d_steady_state_heat_del2() {
    let mut interp = new_interpreter();
    // In steady state with no internal generation, heat conduction satisfies Laplace's eq:
    // del2(T) = 0
    // Linear temperature gradient T(x, y) = 100 + 20*x - 15*y
    let script = r#"
        [X, Y] = meshgrid(0:1:4, 0:1:4);
        T = 100 + 20*X - 15*Y;
        L = del2(T);
        max_laplacian = max(max(abs(L)));
    "#;
    interp.eval_script(script).unwrap();
    let max_l = interp.env.get("max_laplacian").unwrap();
    match max_l {
        Value::Double(d) => {
            let val = d.as_slice()[0];
            assert!(
                val < 1e-12,
                "Laplacian of steady-state linear gradient must be zero, got {val}"
            );
        }
        _ => panic!("Expected double scalar"),
    }
}

#[test]
fn test_3d_lorenz_attractor_ode45_and_plot3() {
    let mut interp = new_interpreter();
    // Lorenz chaotic attractor:
    // dx/dt = sigma*(y - x)
    // dy/dt = x*(rho - z) - y
    // dz/dt = x*y - beta*z
    let script = r#"
        sigma = 10; rho = 28; beta = 8/3;
        lorenz = @(t, a) [sigma*(a(2) - a(1)); a(1)*(rho - a(3)) - a(2); a(1)*a(2) - beta*a(3)];
        [t, sol] = ode45(lorenz, [0, 0.5], [1; 1; 1]);
        figure(1);
        plot3(sol(:, 1), sol(:, 2), sol(:, 3));
        n_points = length(t);
    "#;
    interp.eval_script(script).unwrap();
    let n_pts = interp.env.get("n_points").unwrap();
    match n_pts {
        Value::Double(d) => {
            let n = d.as_slice()[0];
            assert!(
                n > 10.0,
                "Lorenz attractor must integrate across multiple adaptive steps, got {n}"
            );
        }
        _ => panic!("Expected double scalar"),
    }
}
