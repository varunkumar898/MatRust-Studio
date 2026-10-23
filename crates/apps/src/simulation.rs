//! Simulation and Control backend — continuous-time dynamics and ODE integration.
//!
//! Provides Runge-Kutta 4th-order (RK4) numerical integration for:
//! - First-order transfer functions / ODEs: tau * dy/dt + y = K * u(t)
//! - Second-order systems: d²y/dt² + 2*zeta*omega_n*dy/dt + omega_n²*y = omega_n²*K*u(t)
//! - Linear continuous-time state-space models: dx/dt = A*x + B*u, y = C*x + D*u
//!
//! System stability and poles are computed using `matlab_linalg::decomposition::eig_general`.
//! Simulation trajectories, performance metrics, and plots integrate directly with
//! `matlab_runtime::Session` without any UI code in this crate.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use matlab_diagnostics::Diagnostic;
use matlab_linalg::decomposition::eig_general;
use matlab_runtime::graphics::{FigureState, PlotItem};
use matlab_runtime::Session;
use matlab_values::{DenseMatrix, Value};

/// Maximum allowed simulation steps to prevent runaway memory allocation.
pub const MAX_SIMULATION_STEPS: usize = 200_000;

/// System dynamics model.
#[derive(Debug, Clone, PartialEq)]
pub enum SystemModel {
    /// First-order lag: tau * dy/dt + y = K * u(t), with y(0) = y0.
    FirstOrder { tau: f64, gain: f64, y0: f64 },
    /// Second-order canonical system:
    /// d²y/dt² + 2*zeta*omega_n*dy/dt + omega_n²*y = omega_n²*K*u(t)
    SecondOrder {
        omega_n: f64,
        zeta: f64,
        gain: f64,
        y0: f64,
        v0: f64,
    },
    /// Continuous-time linear state-space system:
    /// dx/dt = A x + B u
    /// y = C x + D u
    StateSpace {
        a: DenseMatrix,
        b: DenseMatrix,
        c: DenseMatrix,
        d: f64,
        x0: Vec<f64>,
    },
}

impl SystemModel {
    /// Human-readable model description.
    pub fn description(&self) -> String {
        match self {
            Self::FirstOrder { tau, gain, .. } => {
                format!("First-Order (tau={tau:.3}, K={gain:.3})")
            }
            Self::SecondOrder {
                omega_n,
                zeta,
                gain,
                ..
            } => {
                format!("Second-Order (omega_n={omega_n:.3}, zeta={zeta:.3}, K={gain:.3})")
            }
            Self::StateSpace { a, .. } => {
                format!("State-Space ({}x{} order)", a.rows, a.cols)
            }
        }
    }

    /// Convert any system model into canonical state-space representation.
    pub fn to_state_space(&self) -> Result<CanonicalStateSpace, Diagnostic> {
        match self {
            Self::FirstOrder { tau, gain, y0 } => {
                if *tau <= 0.0 {
                    return Err(Diagnostic::error(
                        format!("Time constant tau must be positive, got {tau}."),
                        None,
                    ));
                }
                if !tau.is_finite() || !gain.is_finite() || !y0.is_finite() {
                    return Err(Diagnostic::error(
                        "First-order model parameters must be finite numbers.",
                        None,
                    ));
                }
                // dx/dt = -1/tau * x + K/tau * u
                // y = 1 * x + 0 * u
                let a = DenseMatrix {
                    rows: 1,
                    cols: 1,
                    data: vec![-1.0 / tau],
                };
                let b = DenseMatrix {
                    rows: 1,
                    cols: 1,
                    data: vec![gain / tau],
                };
                let c = DenseMatrix {
                    rows: 1,
                    cols: 1,
                    data: vec![1.0],
                };
                Ok(CanonicalStateSpace {
                    a,
                    b,
                    c,
                    d: 0.0,
                    x0: vec![*y0],
                    state_names: vec!["y".to_string()],
                })
            }
            Self::SecondOrder {
                omega_n,
                zeta,
                gain,
                y0,
                v0,
            } => {
                if *omega_n <= 0.0 {
                    return Err(Diagnostic::error(
                        format!("Natural frequency omega_n must be positive, got {omega_n}."),
                        None,
                    ));
                }
                if *zeta < 0.0 {
                    return Err(Diagnostic::error(
                        format!("Damping ratio zeta must be non-negative, got {zeta}."),
                        None,
                    ));
                }
                if !omega_n.is_finite()
                    || !zeta.is_finite()
                    || !gain.is_finite()
                    || !y0.is_finite()
                    || !v0.is_finite()
                {
                    return Err(Diagnostic::error(
                        "Second-order model parameters must be finite numbers.",
                        None,
                    ));
                }
                // x1 = y, x2 = dy/dt
                // dx1/dt = x2
                // dx2/dt = -omega_n² * x1 - 2*zeta*omega_n * x2 + omega_n² * K * u
                let wn2 = omega_n * omega_n;
                let a = DenseMatrix {
                    rows: 2,
                    cols: 2,
                    data: vec![0.0, 1.0, -wn2, -2.0 * zeta * omega_n],
                };
                let b = DenseMatrix {
                    rows: 2,
                    cols: 1,
                    data: vec![0.0, wn2 * gain],
                };
                let c = DenseMatrix {
                    rows: 1,
                    cols: 2,
                    data: vec![1.0, 0.0],
                };
                Ok(CanonicalStateSpace {
                    a,
                    b,
                    c,
                    d: 0.0,
                    x0: vec![*y0, *v0],
                    state_names: vec![
                        "y (displacement)".to_string(),
                        "dy/dt (velocity)".to_string(),
                    ],
                })
            }
            Self::StateSpace { a, b, c, d, x0 } => {
                if a.rows != a.cols {
                    return Err(Diagnostic::error(
                        format!("State matrix A must be square, got {}x{}.", a.rows, a.cols),
                        None,
                    ));
                }
                let n = a.rows;
                if n == 0 {
                    return Err(Diagnostic::error(
                        "State dimension n must be at least 1.",
                        None,
                    ));
                }
                if b.rows != n || b.cols != 1 {
                    return Err(Diagnostic::error(
                        format!("Input matrix B must be {}x1, got {}x{}.", n, b.rows, b.cols),
                        None,
                    ));
                }
                if c.rows != 1 || c.cols != n {
                    return Err(Diagnostic::error(
                        format!(
                            "Output matrix C must be 1x{}, got {}x{}.",
                            n, c.rows, c.cols
                        ),
                        None,
                    ));
                }
                if x0.len() != n {
                    return Err(Diagnostic::error(
                        format!(
                            "Initial state vector x0 must have length {}, got {}.",
                            n,
                            x0.len()
                        ),
                        None,
                    ));
                }
                if a.data.iter().any(|v| !v.is_finite())
                    || b.data.iter().any(|v| !v.is_finite())
                    || c.data.iter().any(|v| !v.is_finite())
                    || !d.is_finite()
                    || x0.iter().any(|v| !v.is_finite())
                {
                    return Err(Diagnostic::error(
                        "State-space matrices and initial condition must contain only finite numbers.",
                        None,
                    ));
                }
                let state_names = (1..=n).map(|i| format!("x{i}")).collect();
                Ok(CanonicalStateSpace {
                    a: a.clone(),
                    b: b.clone(),
                    c: c.clone(),
                    d: *d,
                    x0: x0.clone(),
                    state_names,
                })
            }
        }
    }
}

/// Standardized linear state-space form used by the RK4 solver.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalStateSpace {
    pub a: DenseMatrix,
    pub b: DenseMatrix,
    pub c: DenseMatrix,
    pub d: f64,
    pub x0: Vec<f64>,
    pub state_names: Vec<String>,
}

/// Input excitation signal u(t).
#[derive(Debug, Clone, PartialEq)]
pub enum InputSignal {
    /// Step input: u(t) = amplitude for t >= t_start, else 0.
    Step { amplitude: f64, t_start: f64 },
    /// Impulse excitation: rectangular pulse of duration dt with total area equal to magnitude.
    Impulse { magnitude: f64, t_start: f64 },
    /// Ramp input: u(t) = slope * (t - t_start) for t >= t_start, else 0.
    Ramp { slope: f64, t_start: f64 },
    /// Sinusoidal excitation: u(t) = amplitude * sin(2*pi*frequency*t + phase_rad).
    Sine {
        amplitude: f64,
        frequency: f64,
        phase_rad: f64,
    },
    /// Zero input: u(t) = 0 for all t (autonomous/initial-condition response).
    Zero,
}

impl InputSignal {
    /// Evaluate input signal at time t given integration time step dt.
    pub fn eval(&self, t: f64, dt: f64) -> f64 {
        match *self {
            Self::Step { amplitude, t_start } => {
                if t >= t_start {
                    amplitude
                } else {
                    0.0
                }
            }
            Self::Impulse { magnitude, t_start } => {
                if t >= t_start && t < t_start + dt {
                    magnitude / dt
                } else {
                    0.0
                }
            }
            Self::Ramp { slope, t_start } => {
                if t >= t_start {
                    slope * (t - t_start)
                } else {
                    0.0
                }
            }
            Self::Sine {
                amplitude,
                frequency,
                phase_rad,
            } => amplitude * (2.0 * std::f64::consts::PI * frequency * t + phase_rad).sin(),
            Self::Zero => 0.0,
        }
    }

    /// User-facing label for plots and reports.
    pub fn label(&self) -> String {
        match self {
            Self::Step { amplitude, t_start } => {
                format!("Step (amp={amplitude}, t0={t_start})")
            }
            Self::Impulse { magnitude, t_start } => {
                format!("Impulse (mag={magnitude}, t0={t_start})")
            }
            Self::Ramp { slope, t_start } => {
                format!("Ramp (slope={slope}, t0={t_start})")
            }
            Self::Sine {
                amplitude,
                frequency,
                ..
            } => {
                format!("Sine (amp={amplitude}, f={frequency} Hz)")
            }
            Self::Zero => "Zero input (autonomous)".to_string(),
        }
    }
}

/// Simulation time parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationParams {
    pub t_final: f64,
    pub dt: f64,
}

impl SimulationParams {
    pub fn validate(&self) -> Result<usize, Diagnostic> {
        if !self.t_final.is_finite() || self.t_final <= 0.0 {
            return Err(Diagnostic::error(
                format!(
                    "Simulation duration t_final must be positive, got {}.",
                    self.t_final
                ),
                None,
            ));
        }
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err(Diagnostic::error(
                format!("Time step dt must be positive, got {}.", self.dt),
                None,
            ));
        }
        if self.dt > self.t_final {
            return Err(Diagnostic::error(
                format!(
                    "Time step dt ({}) cannot be greater than duration t_final ({}).",
                    self.dt, self.t_final
                ),
                None,
            ));
        }
        let steps = (self.t_final / self.dt).round() as usize + 1;
        if steps > MAX_SIMULATION_STEPS {
            return Err(Diagnostic::error(
                format!(
                    "Requested simulation produces {} steps, exceeding maximum {MAX_SIMULATION_STEPS}. Increase dt or decrease duration.",
                    steps
                ),
                None,
            ));
        }
        Ok(steps)
    }
}

/// Dynamic response and control performance metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseMetrics {
    /// Final or estimated steady-state output value.
    pub steady_state: f64,
    /// Peak output value observed during simulation.
    pub peak_value: f64,
    /// Time when the peak value occurred.
    pub peak_time: f64,
    /// Percent overshoot relative to steady-state (for step responses).
    pub overshoot_percent: Option<f64>,
    /// Rise time (10% to 90% of change from initial to steady-state).
    pub rise_time: Option<f64>,
    /// Settling time within 2% band of steady-state value.
    pub settling_time: Option<f64>,
    /// Whether all system poles have negative real parts (asymptotic stability).
    pub is_stable: bool,
    /// System poles / eigenvalues of A: vector of (real, imag).
    pub poles: Vec<(f64, f64)>,
}

/// Complete results of a simulation run.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationResult {
    pub model_name: String,
    pub input_label: String,
    pub time: Vec<f64>,
    pub y: Vec<f64>,
    pub u: Vec<f64>,
    /// State trajectories: length N vector of state vectors (each of length n).
    pub states: Vec<Vec<f64>>,
    pub state_names: Vec<String>,
    pub metrics: ResponseMetrics,
}

fn cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

/// Evaluate state derivative dx/dt = A*x + B*u.
fn eval_dx(a: &DenseMatrix, b: &DenseMatrix, x: &[f64], u: f64, out: &mut [f64]) {
    let n = a.rows;
    for i in 0..n {
        let mut sum = 0.0;
        let row_offset = i * n;
        for j in 0..n {
            sum += a.data[row_offset + j] * x[j];
        }
        sum += b.data[i] * u;
        out[i] = sum;
    }
}

/// Compute poles and analyze stability of the continuous-time system matrix A.
pub fn compute_poles_and_stability(a: &DenseMatrix) -> Result<(Vec<(f64, f64)>, bool), Diagnostic> {
    let (d, _) = eig_general(a, None)?;
    let mut poles = Vec::with_capacity(d.rows);
    let mut is_stable = true;
    for i in 0..d.rows {
        let lambda = d.data[i];
        poles.push((lambda.re, lambda.im));
        if lambda.re >= -1e-12 {
            is_stable = false;
        }
    }
    // Sort poles by real part ascending
    poles.sort_by(|p1, p2| {
        p1.0.partial_cmp(&p2.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| p1.1.partial_cmp(&p2.1).unwrap_or(std::cmp::Ordering::Equal))
    });
    Ok((poles, is_stable))
}

/// Calculate dynamic response metrics from output series y(t) and time vector t.
pub fn analyze_response(
    t: &[f64],
    y: &[f64],
    y0: f64,
    poles: Vec<(f64, f64)>,
    is_stable: bool,
) -> ResponseMetrics {
    let n = y.len();
    if n == 0 {
        return ResponseMetrics {
            steady_state: 0.0,
            peak_value: 0.0,
            peak_time: 0.0,
            overshoot_percent: None,
            rise_time: None,
            settling_time: None,
            is_stable,
            poles,
        };
    }

    // Steady state estimate: mean of final 5% samples
    let tail_count = ((n as f64) * 0.05).ceil() as usize;
    let tail_slice = &y[n.saturating_sub(tail_count.max(1))..];
    let steady_state = tail_slice.iter().sum::<f64>() / tail_slice.len() as f64;

    // Peak value and time
    let delta = steady_state - y0;
    let mut peak_val = y[0];
    let mut peak_time = t[0];

    if delta > 1e-6 {
        // Positive step: find maximum y
        let mut max_y = y[0];
        let mut max_t = t[0];
        for (&ti, &yi) in t.iter().zip(y.iter()) {
            if yi > max_y {
                max_y = yi;
                max_t = ti;
            }
        }
        peak_val = max_y;
        peak_time = max_t;
    } else if delta < -1e-6 {
        // Negative step: find minimum y
        let mut min_y = y[0];
        let mut min_t = t[0];
        for (&ti, &yi) in t.iter().zip(y.iter()) {
            if yi < min_y {
                min_y = yi;
                min_t = ti;
            }
        }
        peak_val = min_y;
        peak_time = min_t;
    } else {
        // Zero-input or no step change: find extremum furthest from 0
        let mut max_abs = y[0].abs();
        for (&ti, &yi) in t.iter().zip(y.iter()) {
            if yi.abs() > max_abs {
                max_abs = yi.abs();
                peak_val = yi;
                peak_time = ti;
            }
        }
    }

    // Overshoot for step response (when there is a significant step change from y0 to y_ss)
    let mut overshoot_percent = None;
    let mut rise_time = None;
    let mut settling_time = None;

    if delta.abs() > 1e-6 {
        // Overshoot
        if delta > 0.0 && peak_val > steady_state {
            overshoot_percent = Some(((peak_val - steady_state) / delta.abs()) * 100.0);
        } else if delta < 0.0 && peak_val < steady_state {
            overshoot_percent = Some(((steady_state - peak_val) / delta.abs()) * 100.0);
        } else {
            overshoot_percent = Some(0.0);
        }

        // Rise time (10% to 90% of change)
        let y10 = y0 + 0.10 * delta;
        let y90 = y0 + 0.90 * delta;
        let mut t10: Option<f64> = None;
        let mut t90: Option<f64> = None;

        for (idx, &yi) in y.iter().enumerate() {
            if delta > 0.0 {
                if t10.is_none() && yi >= y10 {
                    t10 = Some(t[idx]);
                }
                if t90.is_none() && yi >= y90 {
                    t90 = Some(t[idx]);
                }
            } else {
                if t10.is_none() && yi <= y10 {
                    t10 = Some(t[idx]);
                }
                if t90.is_none() && yi <= y90 {
                    t90 = Some(t[idx]);
                }
            }
        }
        if let (Some(t_start), Some(t_end)) = (t10, t90) {
            if t_end >= t_start {
                rise_time = Some(t_end - t_start);
            }
        }

        // Settling time (2% band around steady state)
        let band = 0.02 * delta.abs();
        let mut last_outside = None;
        for (idx, &yi) in y.iter().enumerate() {
            if (yi - steady_state).abs() > band {
                last_outside = Some(t[idx]);
            }
        }
        settling_time = last_outside;
    }

    ResponseMetrics {
        steady_state,
        peak_value: peak_val,
        peak_time,
        overshoot_percent,
        rise_time,
        settling_time,
        is_stable,
        poles,
    }
}

/// Core RK4 numerical integration of state-space system.
pub fn simulate_rk4(
    system: &CanonicalStateSpace,
    input: &InputSignal,
    params: &SimulationParams,
    cancel: Option<&AtomicBool>,
) -> Result<SimulationResult, Diagnostic> {
    let steps = params.validate()?;
    let (poles, is_stable) = compute_poles_and_stability(&system.a)?;

    let n = system.a.rows;
    let dt = params.dt;
    let mut time = Vec::with_capacity(steps);
    let mut y = Vec::with_capacity(steps);
    let mut u = Vec::with_capacity(steps);
    let mut states = Vec::with_capacity(steps);

    let mut x = system.x0.clone();
    let mut k1 = vec![0.0; n];
    let mut k2 = vec![0.0; n];
    let mut k3 = vec![0.0; n];
    let mut k4 = vec![0.0; n];
    let mut x_temp = vec![0.0; n];

    for step in 0..steps {
        if step % 256 == 0 && cancelled(cancel) {
            return Err(Diagnostic::error("Simulation cancelled by user.", None));
        }

        let t = (step as f64) * dt;
        let u_val = input.eval(t, dt);

        // Compute output y = C*x + D*u
        let mut y_val = system.d * u_val;
        for i in 0..n {
            y_val += system.c.data[i] * x[i];
        }

        if !y_val.is_finite() || x.iter().any(|val| !val.is_finite()) {
            return Err(Diagnostic::error(
                format!("Simulation diverged at t = {t:.4}s (non-finite state or output). System may be unstable."),
                None,
            ));
        }

        time.push(t);
        y.push(y_val);
        u.push(u_val);
        states.push(x.clone());

        if step + 1 == steps {
            break;
        }

        // RK4 Stage 1: k1 = f(t, x)
        eval_dx(&system.a, &system.b, &x, u_val, &mut k1);

        // RK4 Stage 2: k2 = f(t + dt/2, x + dt/2 * k1)
        let t_mid = t + 0.5 * dt;
        let u_mid1 = input.eval(t_mid, dt);
        for i in 0..n {
            x_temp[i] = x[i] + 0.5 * dt * k1[i];
        }
        eval_dx(&system.a, &system.b, &x_temp, u_mid1, &mut k2);

        // RK4 Stage 3: k3 = f(t + dt/2, x + dt/2 * k2)
        let u_mid2 = input.eval(t_mid, dt);
        for i in 0..n {
            x_temp[i] = x[i] + 0.5 * dt * k2[i];
        }
        eval_dx(&system.a, &system.b, &x_temp, u_mid2, &mut k3);

        // RK4 Stage 4: k4 = f(t + dt, x + dt * k3)
        let t_next = t + dt;
        let u_next = input.eval(t_next, dt);
        for i in 0..n {
            x_temp[i] = x[i] + dt * k3[i];
        }
        eval_dx(&system.a, &system.b, &x_temp, u_next, &mut k4);

        // State update: x = x + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        for i in 0..n {
            x[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
    }

    let y0 = y.first().copied().unwrap_or(0.0);
    let metrics = analyze_response(&time, &y, y0, poles, is_stable);

    Ok(SimulationResult {
        model_name: "Linear Continuous-Time System".to_string(),
        input_label: input.label(),
        time,
        y,
        u,
        states,
        state_names: system.state_names.clone(),
        metrics,
    })
}

/// Execute a simulation, write workspace variables, and generate figure plots
/// in the provided `Session`.
pub fn run_simulation(
    session: &mut Session,
    model: &SystemModel,
    input: &InputSignal,
    params: &SimulationParams,
    cancel: Option<&AtomicBool>,
) -> Result<SimulationResult, Diagnostic> {
    let canonical = model.to_state_space()?;
    let mut result = simulate_rk4(&canonical, input, params, cancel)?;
    result.model_name = model.description();

    // Injects workspace variables:
    // sim_t: N x 1 matrix
    // sim_y: N x 1 matrix
    // sim_u: N x 1 matrix
    // sim_x: N x n matrix
    let n_steps = result.time.len();
    let n_states = canonical.a.rows;

    session.env.set(
        "sim_t",
        Value::Matrix(DenseMatrix {
            rows: n_steps,
            cols: 1,
            data: result.time.clone(),
        }),
    );
    session.env.set(
        "sim_y",
        Value::Matrix(DenseMatrix {
            rows: n_steps,
            cols: 1,
            data: result.y.clone(),
        }),
    );
    session.env.set(
        "sim_u",
        Value::Matrix(DenseMatrix {
            rows: n_steps,
            cols: 1,
            data: result.u.clone(),
        }),
    );

    let mut state_flat = Vec::with_capacity(n_steps * n_states);
    for row in &result.states {
        state_flat.extend_from_slice(row);
    }
    session.env.set(
        "sim_x",
        Value::Matrix(DenseMatrix {
            rows: n_steps,
            cols: n_states,
            data: state_flat,
        }),
    );

    session.env.set(
        "sim_steady_state",
        Value::Matrix(DenseMatrix::scalar(result.metrics.steady_state)),
    );
    session.env.set(
        "sim_peak",
        Value::Matrix(DenseMatrix::scalar(result.metrics.peak_value)),
    );
    session.env.set(
        "sim_overshoot",
        Value::Matrix(DenseMatrix::scalar(
            result.metrics.overshoot_percent.unwrap_or(f64::NAN),
        )),
    );
    session.env.set(
        "sim_rise_time",
        Value::Matrix(DenseMatrix::scalar(
            result.metrics.rise_time.unwrap_or(f64::NAN),
        )),
    );
    session.env.set(
        "sim_settling_time",
        Value::Matrix(DenseMatrix::scalar(
            result.metrics.settling_time.unwrap_or(f64::NAN),
        )),
    );

    // Figure 1: Time Response (Output y and Input u)
    let handle1 = session
        .graphics
        .figures
        .iter()
        .map(|f| f.handle)
        .max()
        .unwrap_or(0)
        + 1;
    let mut fig1 = FigureState::fresh(handle1);
    fig1.title = Some(format!("Simulation — {}", result.model_name));
    fig1.xlabel = Some("Time (s)".to_string());
    fig1.ylabel = Some("Signal Amplitude".to_string());
    fig1.grid = true;
    fig1.items.push(PlotItem::Line {
        label: "Output y(t)".to_string(),
        x: result.time.clone(),
        y: result.y.clone(),
    });
    fig1.items.push(PlotItem::Line {
        label: format!("Input u(t) [{}]", result.input_label),
        x: result.time.clone(),
        y: result.u.clone(),
    });
    session.graphics.figures.push(fig1);
    session.graphics.current = Some(session.graphics.figures.len() - 1);

    // Figure 2: Phase Portrait (State Trajectory) if system order >= 2
    if n_states >= 2 {
        let handle2 = session
            .graphics
            .figures
            .iter()
            .map(|f| f.handle)
            .max()
            .unwrap_or(0)
            + 1;
        let mut fig2 = FigureState::fresh(handle2);
        let name_x = result
            .state_names
            .first()
            .cloned()
            .unwrap_or_else(|| "x1".to_string());
        let name_y = result
            .state_names
            .get(1)
            .cloned()
            .unwrap_or_else(|| "x2".to_string());
        fig2.title = Some(format!("Phase Portrait — {}", result.model_name));
        fig2.xlabel = Some(name_x);
        fig2.ylabel = Some(name_y);
        fig2.grid = true;

        let traj_x: Vec<f64> = result.states.iter().map(|s| s[0]).collect();
        let traj_y: Vec<f64> = result.states.iter().map(|s| s[1]).collect();

        fig2.items.push(PlotItem::Line {
            label: "Trajectory".to_string(),
            x: traj_x.clone(),
            y: traj_y.clone(),
        });
        fig2.items.push(PlotItem::Scatter {
            label: "Initial state (x0)".to_string(),
            x: vec![traj_x[0]],
            y: vec![traj_y[0]],
        });
        session.graphics.figures.push(fig2);
        session.graphics.current = Some(session.graphics.figures.len() - 1);
    }

    Ok(result)
}

fn guard_overwrite(path: &Path, overwrite: bool) -> Result<(), Diagnostic> {
    if path.exists() && !overwrite {
        return Err(Diagnostic::error(
            format!(
                "Refusing to overwrite existing file {} without confirmation (simulation).",
                path.display()
            ),
            None,
        ));
    }
    Ok(())
}

/// Format simulation trajectories as CSV content.
pub fn format_simulation_csv(result: &SimulationResult) -> String {
    let mut header = String::from("time,y,u");
    for name in &result.state_names {
        header.push(',');
        header.push_str(name);
    }
    header.push('\n');

    let mut out = header;
    for (idx, ((&t, &y), &u)) in result
        .time
        .iter()
        .zip(result.y.iter())
        .zip(result.u.iter())
        .enumerate()
    {
        out.push_str(&format!("{t:.6},{y:.6},{u:.6}"));
        if let Some(state_row) = result.states.get(idx) {
            for &x_val in state_row {
                out.push_str(&format!(",{x_val:.6}"));
            }
        }
        out.push('\n');
    }
    out
}

/// Export simulation time history and state trajectories to a CSV file.
pub fn export_simulation_csv(
    path: &Path,
    result: &SimulationResult,
    overwrite: bool,
) -> Result<(), Diagnostic> {
    guard_overwrite(path, overwrite)?;
    let content = format_simulation_csv(result);
    std::fs::write(path, content).map_err(|err| {
        Diagnostic::error(format!("Could not write {}: {}", path.display(), err), None)
    })
}
