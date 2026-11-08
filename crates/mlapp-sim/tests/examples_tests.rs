//! Integration and oracle tests for Milestone M10:
//! 8 Worked Examples from JSON models.

use std::fs;
use std::path::PathBuf;

use mlapp_sim::model::BlockDiagram;
use mlapp_sim::solver::simulate_diagram;

fn load_example(name: &str) -> BlockDiagram {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates
    path.pop(); // workspace root
    path.push("examples");
    path.push("models");
    path.push(format!("{name}.json"));

    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to deserialize {}: {e}", path.display()))
}

#[test]
fn test_example_rlc_transient() {
    let diag = load_example("rlc_transient");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    // Theoretical analytical solution:
    // Step 10V into series RLC: R = 4 Ohm, L = 0.01 H, C = 100 uF (1e-4 F).
    // alpha = R / (2L) = 4 / 0.02 = 200 s^-1.
    // omega0^2 = 1 / (LC) = 1e6 -> omega0 = 1000 rad/s.
    // omegad = sqrt(omega0^2 - alpha^2) = sqrt(1e6 - 40000) = sqrt(960000) ~ 979.7959 rad/s.
    // v_C(t) = 10 * (1 - exp(-alpha * t) * (cos(omegad * t) + (alpha / omegad) * sin(omegad * t)))
    let alpha = 200.0_f64;
    let omegad = 960000.0_f64.sqrt();

    let vc = res
        .signals
        .get("C1.v")
        .or_else(|| res.signals.get("V_Sensor.v"))
        .or_else(|| res.signals.get("V_Sensor"))
        .expect("Capacitor voltage or sensor signal logged");

    assert!(!res.tout.is_empty(), "Time vector non-empty");
    assert_eq!(res.tout.len(), vc.len());

    let mut max_err = 0.0_f64;
    let mut peak_val = 0.0_f64;
    for (&t, &v) in res.tout.iter().zip(vc.iter()) {
        if v > peak_val {
            peak_val = v;
        }
        let expected = 10.0
            * (1.0
                - (-alpha * t).exp()
                    * ((omegad * t).cos() + (alpha / omegad) * (omegad * t).sin()));
        let err = (v - expected).abs();
        if err > max_err {
            max_err = err;
        }
    }

    // Analytic match within numerical integration tolerance (Euler/RK4)
    assert!(
        max_err < 0.1,
        "RLC transient max analytical error {max_err} must be < 0.1V"
    );

    // Analytical peak overshoot: t_p = pi / omegad ~ 3.2064 ms.
    // v_peak = 10 * (1 + exp(-alpha * pi / omegad)) ~ 15.266V
    let expected_peak = 10.0 * (1.0 + (-alpha * std::f64::consts::PI / omegad).exp());
    assert!(
        (peak_val - expected_peak).abs() < 0.1,
        "Peak voltage {peak_val} must match theoretical peak {expected_peak} within 0.1V"
    );
}

#[test]
fn test_example_rectifiers() {
    let diag = load_example("rectifiers");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let vc = res
        .signals
        .get("C_filter.v")
        .or_else(|| res.signals.get("R_load.v"))
        .or_else(|| res.signals.get("V_Sensor.v"))
        .expect("Filter capacitor voltage logged");

    // Vac peak = 10.0V, Diode Vf = 0.7V.
    // Filter peak voltage must reach approx 10.0 - 0.7 = 9.3V (within 0.15V).
    let max_v = vc.iter().copied().fold(0.0_f64, f64::max);
    assert!(
        (max_v - 9.3).abs() < 0.15,
        "Peak rectified voltage {max_v} must be near 9.3V (10V - 0.7V)"
    );

    // In steady state (t > 0.03s), capacitor discharge ripple:
    // Delta V ~ I_load * T / C = (9.3V / 100 Ohm) * 0.02s / 1e-3F = 0.093 * 20 = 1.86V.
    let steady_samples: Vec<f64> = res
        .tout
        .iter()
        .zip(vc.iter())
        .filter(|(&t, _)| t > 0.035)
        .map(|(_, &v)| v)
        .collect();

    assert!(!steady_samples.is_empty(), "Steady state samples present");
    let v_min = steady_samples.iter().copied().fold(f64::INFINITY, f64::min);
    let v_max = steady_samples
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let ripple = v_max - v_min;

    // Theoretical ripple ~ 1.86V; verify within 1.0V to 2.5V
    assert!(
        ripple > 1.0 && ripple < 2.5,
        "Rectifier filter ripple {ripple}V must match theoretical formula ~1.86V"
    );
}

#[test]
fn test_example_buck_converter() {
    let diag = load_example("buck_converter");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let vout = res
        .signals
        .get("R_load.v")
        .or_else(|| res.signals.get("C_out.v"))
        .or_else(|| res.signals.get("V_Sensor.v"))
        .expect("Output voltage logged");

    // Buck converter with closed-loop PID control: Vin = 24V -> Vref = 5.0V.
    // After settling (second half of simulation, t > 0.001s), mean output must be near 5.0V.
    let late_samples: Vec<f64> = res
        .tout
        .iter()
        .zip(vout.iter())
        .filter(|(&t, _)| t > 0.001)
        .map(|(_, &v)| v)
        .collect();

    assert!(!late_samples.is_empty(), "Late samples present");
    let mean_vout = late_samples.iter().sum::<f64>() / (late_samples.len() as f64);

    assert!(
        (mean_vout - 5.0).abs() < 0.25,
        "Buck converter closed loop mean output {mean_vout}V must track 5.0V within 5%"
    );
}

#[test]
fn test_example_boost_converter() {
    let diag = load_example("boost_converter");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let vout = res
        .signals
        .get("R_load.v")
        .or_else(|| res.signals.get("C_out.v"))
        .or_else(|| res.signals.get("V_Sensor.v"))
        .expect("Boost converter output voltage logged");

    // Boost converter: Vin = 12V, D ~ 0.52.
    // Ideal Vout = Vin / (1 - D) = 12 / 0.48 ~ 25.0V.
    // After startup (t > 0.003s), output must be boosted above Vin (e.g. > 18.0V and < 28.0V).
    let late_samples: Vec<f64> = res
        .tout
        .iter()
        .zip(vout.iter())
        .filter(|(&t, _)| t > 0.003)
        .map(|(_, &v)| v)
        .collect();

    assert!(!late_samples.is_empty(), "Late samples present");
    let mean_vout = late_samples.iter().sum::<f64>() / (late_samples.len() as f64);

    assert!(
        mean_vout > 18.0 && mean_vout < 28.0,
        "Boost converter mean output {mean_vout}V must boost 12V to ~24V"
    );
}

#[test]
fn test_example_inverting_opamp() {
    let diag = load_example("inverting_opamp");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let vout = res
        .signals
        .get("Vout")
        .or_else(|| res.signals.get("OpAmp_VCVS.v"))
        .expect("Vout logged");

    // Closed loop inverting op amp: Vin = 1.0V, R1 = 10k, Rf = 100k -> Vout = -10.0V.
    let final_vout = *vout.last().expect("last sample");
    assert!(
        (final_vout - (-10.0)).abs() < 1e-3,
        "Inverting op-amp final output {final_vout} must equal -10.0V within 1mV"
    );
}

#[test]
fn test_example_non_inverting_opamp() {
    let diag = load_example("non_inverting_opamp");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let vout = res
        .signals
        .get("Vout")
        .or_else(|| res.signals.get("OpAmp_VCVS.v"))
        .expect("Vout logged");

    // Closed loop non-inverting op amp: Vin = 1.0V, R1 = 10k, Rf = 100k -> Vout = 1 + Rf/R1 = 11.0V.
    let final_vout = *vout.last().expect("last sample");
    assert!(
        (final_vout - 11.0).abs() < 1e-3,
        "Non-inverting op-amp final output {final_vout} must equal 11.0V within 1mV"
    );
}

#[test]
fn test_example_dc_motor() {
    let diag = load_example("dc_motor");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let omega = res
        .signals
        .get("Omega_out")
        .or_else(|| res.signals.get("Integ_Omega"))
        .expect("Omega signal logged");

    // Closed-loop DC motor velocity control: setpoint Omega_ref = 50.0 rad/s.
    // In steady state (at t = 0.5s), omega must track 50.0 rad/s within 0.1% (0.05 rad/s).
    let final_omega = *omega.last().expect("final sample");
    assert!(
        (final_omega - 50.0).abs() < 0.05,
        "DC motor steady state velocity {final_omega} rad/s must track 50.0 rad/s within 0.1%"
    );
}

#[test]
fn test_example_voice_coil_actuator() {
    let diag = load_example("voice_coil_actuator");
    let cfg = diag.simulation_config.clone().expect("sim config present");
    let res = simulate_diagram(&diag, &cfg).expect("simulation succeeded");

    let x = res
        .signals
        .get("X_out")
        .or_else(|| res.signals.get("Integ_Pos"))
        .expect("Position signal logged");

    let v = res
        .signals
        .get("V_out")
        .or_else(|| res.signals.get("Integ_Vel"))
        .expect("Velocity signal logged");

    // Verify system stability and bounded oscillations under 10Hz AC excitation:
    assert!(!x.is_empty(), "Position non-empty");
    assert!(!v.is_empty(), "Velocity non-empty");

    for &pos in x {
        assert!(
            pos.is_finite(),
            "Position must remain finite (no NaN or Inf)"
        );
        assert!(
            pos.abs() < 0.5,
            "Position {pos}m must remain physically bounded"
        );
    }

    for &vel in v {
        assert!(vel.is_finite(), "Velocity must remain finite");
        assert!(
            vel.abs() < 5.0,
            "Velocity {vel}m/s must remain physically bounded"
        );
    }
}
