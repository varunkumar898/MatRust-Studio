//! Simulation and modeling diagnostics: errors, warnings, suggested fixes,
//! and proactive compile-time/topological circuit validation.

use crate::model::{Block, BlockDiagram, BlockRef, BlockType, Line, PortKind, PortRef};
use crate::netlist::resolve_electrical_nets;
use crate::solver::{SimError, SimulationConfig};
use crate::spec::validate_block_params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity of a simulation or validation diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }
}

/// Suggested action to resolve a diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixAction {
    Message(String),
    AddGroundToNet { net_id: usize, position: (f32, f32) },
    InsertSeriesResistor { branch_id: BlockRef, value: f64 },
    InsertParallelResistor { node_id: usize, value: f64 },
    ConnectPortToDefaultZero { block: BlockRef, port: usize },
}

impl PartialEq for FixAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Message(a), Self::Message(b)) => a == b,
            (
                Self::AddGroundToNet {
                    net_id: n1,
                    position: (x1, y1),
                },
                Self::AddGroundToNet {
                    net_id: n2,
                    position: (x2, y2),
                },
            ) => n1 == n2 && x1.to_bits() == x2.to_bits() && y1.to_bits() == y2.to_bits(),
            (
                Self::InsertSeriesResistor {
                    branch_id: b1,
                    value: v1,
                },
                Self::InsertSeriesResistor {
                    branch_id: b2,
                    value: v2,
                },
            ) => b1 == b2 && v1.to_bits() == v2.to_bits(),
            (
                Self::InsertParallelResistor {
                    node_id: n1,
                    value: v1,
                },
                Self::InsertParallelResistor {
                    node_id: n2,
                    value: v2,
                },
            ) => n1 == n2 && v1.to_bits() == v2.to_bits(),
            (
                Self::ConnectPortToDefaultZero {
                    block: b1,
                    port: p1,
                },
                Self::ConnectPortToDefaultZero {
                    block: b2,
                    port: p2,
                },
            ) => b1 == b2 && p1 == p2,
            _ => false,
        }
    }
}

impl Eq for FixAction {}

/// Canonical metadata describing a diagnostic code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    pub code: &'static str,
    pub severity: DiagnosticSeverity,
    pub title: &'static str,
    pub explanation: &'static str,
    pub typical_fix: &'static str,
}

pub const DIAGNOSTIC_REGISTRY: &[DiagnosticInfo] = &[
    DiagnosticInfo {
        code: "E-ALG",
        severity: DiagnosticSeverity::Error,
        title: "Algebraic Loop",
        explanation: "Direct feedthrough algebraic loop without a state block",
        typical_fix: "Insert an Integrator, Unit Delay, or Zero-Order Hold to break the instantaneous algebraic loop",
    },
    DiagnosticInfo {
        code: "E-STEP",
        severity: DiagnosticSeverity::Error,
        title: "Step Size Failure",
        explanation: "Adaptive solver step size shrank below minimum allowed step size without achieving error tolerances",
        typical_fix: "Loosen error tolerances (RelTol / AbsTol) or check for state discontinuities",
    },
    DiagnosticInfo {
        code: "E-NAN",
        severity: DiagnosticSeverity::Error,
        title: "NaN or Infinite State",
        explanation: "Numerical integration encountered non-finite (NaN or Inf) values in state derivatives or outputs",
        typical_fix: "Check for division by zero in transfer functions or excessive integration step size",
    },
    DiagnosticInfo {
        code: "E-SAMPLE",
        severity: DiagnosticSeverity::Error,
        title: "Missing Sample Time",
        explanation: "Discrete block has no sample time specified when used with a variable-step solver",
        typical_fix: "Configure an explicit positive SampleTime parameter on the discrete block",
    },
    DiagnosticInfo {
        code: "E-PARAM",
        severity: DiagnosticSeverity::Error,
        title: "Invalid Parameter",
        explanation: "Component parameter has an invalid, negative, or unparseable value",
        typical_fix: "Verify that component values (resistance, capacitance, inductance, frequency) are strictly positive",
    },
    DiagnosticInfo {
        code: "E-GND",
        severity: DiagnosticSeverity::Error,
        title: "Missing Ground",
        explanation: "Electrical circuit island has no Ground reference (Node 0)",
        typical_fix: "Add a Ground block connected to this electrical island",
    },
    DiagnosticInfo {
        code: "E-FLOAT",
        severity: DiagnosticSeverity::Error,
        title: "Floating Net / Island",
        explanation: "Net or electrical island has no DC path to ground reference",
        typical_fix: "Add a Ground block to the net or provide a DC conductance path",
    },
    DiagnosticInfo {
        code: "E-VLOOP",
        severity: DiagnosticSeverity::Error,
        title: "Voltage Source Loop",
        explanation: "Closed loop formed entirely by ideal voltage sources and switches without series impedance violates KVL",
        typical_fix: "Add a small series resistor or internal source resistance to break the ideal voltage source loop",
    },
    DiagnosticInfo {
        code: "E-ICUT",
        severity: DiagnosticSeverity::Error,
        title: "Current Source Cut-Set",
        explanation: "Cut-set of ideal current sources with no parallel discharge path violates KCL",
        typical_fix: "Add a large parallel resistor across the current source",
    },
    DiagnosticInfo {
        code: "E-SINGULAR",
        severity: DiagnosticSeverity::Error,
        title: "Singular Circuit Matrix",
        explanation: "MNA conductance or capacitance matrix is singular, preventing LU factorization",
        typical_fix: "Check for floating subcircuits, parallel voltage sources, or series current sources",
    },
    DiagnosticInfo {
        code: "E-PORT",
        severity: DiagnosticSeverity::Error,
        title: "Unconnected Input Port",
        explanation: "Block input port is not connected to any driving signal line (strict error in schema version 4)",
        typical_fix: "Connect the input port to a driving block or connect a Constant zero source",
    },
    DiagnosticInfo {
        code: "W-PORT",
        severity: DiagnosticSeverity::Warning,
        title: "Unconnected Input Port",
        explanation: "Block input port is unconnected; evaluating to default 0.0 (schema version < 4)",
        typical_fix: "Connect the input port to a driving block",
    },
    DiagnosticInfo {
        code: "E-MULTI",
        severity: DiagnosticSeverity::Error,
        title: "Multiple Port Inputs",
        explanation: "Multiple output lines drive the same input port without a Summer or Mux (strict error in schema version 4)",
        typical_fix: "Insert a Sum or Mux block to combine signals before feeding the input port",
    },
    DiagnosticInfo {
        code: "W-MULTI",
        severity: DiagnosticSeverity::Warning,
        title: "Multiple Port Inputs",
        explanation: "Multiple lines connect to the same input port (schema version < 4)",
        typical_fix: "Insert a Sum or Mux block to explicitly combine signals",
    },
    DiagnosticInfo {
        code: "E-SOLVER",
        severity: DiagnosticSeverity::Error,
        title: "Incompatible Solver",
        explanation: "Electrical domain circuit blocks require a fixed-step solver (Euler or RK4)",
        typical_fix: "Set solver type to Fixed-Step Euler or Runge-Kutta 4 in SimulationConfig",
    },
    DiagnosticInfo {
        code: "E-CHATTER",
        severity: DiagnosticSeverity::Error,
        title: "Switching Limit Cycle",
        explanation: "Switching device limit cycle persisted for more than 100 consecutive steps",
        typical_fix: "Add snubber RC circuit across diodes/switches or decrease solver step size",
    },
    DiagnosticInfo {
        code: "W-DELAY",
        severity: DiagnosticSeverity::Warning,
        title: "Algebraic Hybrid Loop",
        explanation: "Direct sensor-to-source feedback loop without state; resolved by inserting 1-step latency",
        typical_fix: "Insert a UnitDelay or ZeroOrderHold block to explicitly decouple the loop",
    },
    DiagnosticInfo {
        code: "W-WINDOW",
        severity: DiagnosticSeverity::Warning,
        title: "Non-Integer Window",
        explanation: "Measurement window length is not an integer multiple of the sample time",
        typical_fix: "Adjust SampleTime or FundamentalFrequency so that 1 / (f * Ts) is an exact integer",
    },
    DiagnosticInfo {
        code: "W-CHATTER",
        severity: DiagnosticSeverity::Warning,
        title: "Switch Chattering",
        explanation: "Switching state limit cycle detected within simulation step",
        typical_fix: "Add snubber capacitance or verify switching threshold hysteresis",
    },
    DiagnosticInfo {
        code: "W-STEP",
        severity: DiagnosticSeverity::Warning,
        title: "Excessive Step Size",
        explanation: "Solver step size exceeds 2*tau_min of circuit time constants, causing oscillations",
        typical_fix: "Reduce solver step size h <= tau_min / 5",
    },
    DiagnosticInfo {
        code: "W-WIRE",
        severity: DiagnosticSeverity::Warning,
        title: "Redundant Wire",
        explanation: "Self-loop wire or duplicate wire connects the same terminal pair",
        typical_fix: "Remove redundant or self-loop wire",
    },
    DiagnosticInfo {
        code: "W-OPEN",
        severity: DiagnosticSeverity::Warning,
        title: "Unconnected Terminal",
        explanation: "Electrical component terminal has no attached wires",
        typical_fix: "Connect the terminal to the circuit or remove the unused block",
    },
    DiagnosticInfo {
        code: "W-THD-LIMIT",
        severity: DiagnosticSeverity::Warning,
        title: "Harmonic Order Clamped",
        explanation: "Requested MaxHarmonic exceeds Nyquist limit (Nw - 1) / 2",
        typical_fix: "Decrease MaxHarmonic or decrease SampleTime to increase window sample count",
    },
    DiagnosticInfo {
        code: "I-SLOW",
        severity: DiagnosticSeverity::Info,
        title: "Fast-Path Fallback",
        explanation: "Circuit exceeds 10 switches or 64 MB pre-factored table; using runtime LU solver",
        typical_fix: "Reduce switch count to <= 10 if pre-factored execution is required",
    },
    DiagnosticInfo {
        code: "I-THD-NOFUND",
        severity: DiagnosticSeverity::Info,
        title: "Zero Fundamental",
        explanation: "Fundamental component amplitude is zero; THD is mathematically undefined",
        typical_fix: "Ensure signal contains non-zero fundamental frequency amplitude",
    },
    DiagnosticInfo {
        code: "E-DOMAIN",
        severity: DiagnosticSeverity::Error,
        title: "Domain Mismatch",
        explanation: "Direct wire connection between continuous signal port and electrical terminal",
        typical_fix: "Use a Sensor or Controlled Source block to interface between domains",
    },
    DiagnosticInfo {
        code: "E-UNSUPPORTED",
        severity: DiagnosticSeverity::Error,
        title: "Unsupported Block",
        explanation: "Block type is not supported in the simulation engine",
        typical_fix: "Replace with a supported standard library block",
    },
];

pub fn lookup_diagnostic(code: &str) -> Option<&'static DiagnosticInfo> {
    DIAGNOSTIC_REGISTRY.iter().find(|d| d.code == code)
}

/// A structured diagnostic message adhering to Charter Principle P4:
/// "what happened -> why -> how to fix".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub block_sids: Vec<String>,
    pub fix: Option<Box<FixAction>>,
}

impl Diagnostic {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        block_sids: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            block_sids,
            fix: None,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        block_sids: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            block_sids,
            fix: None,
        }
    }

    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        block_sids: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            block_sids,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: FixAction) -> Self {
        self.fix = Some(Box::new(fix));
        self
    }

    pub fn title(&self) -> &str {
        lookup_diagnostic(&self.code)
            .map(|i| i.title)
            .unwrap_or(self.code.as_str())
    }

    pub fn explanation(&self) -> &str {
        lookup_diagnostic(&self.code)
            .map(|i| i.explanation)
            .unwrap_or("")
    }

    pub fn typical_fix(&self) -> Option<&str> {
        lookup_diagnostic(&self.code).map(|i| i.typical_fix)
    }
}

/// Run all static compile-time and topological diagnostics on a block diagram.
pub fn check_diagram(diagram: &BlockDiagram, config: Option<&SimulationConfig>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // 1. Parameter validations
    for b in &diagram.blocks {
        let b_diags = validate_block_params(&b.block_type, &b.sid, &b.parameters);
        diags.extend(b_diags);
    }

    // 2. Port connection validations (E-PORT/W-PORT, E-MULTI/W-MULTI)
    let is_schema4 = diagram.schema_version >= 4;
    for b in &diagram.blocks {
        let n_in = b.input_ports();
        if n_in == 0 {
            continue;
        }

        // Count incoming lines per input port
        let mut port_lines: HashMap<usize, usize> = HashMap::new();
        for line in &diagram.lines {
            if line.dst.block_sid == b.sid || line.dst.block_sid == b.name {
                let p = if line.dst.port_index > 0 {
                    line.dst.port_index
                } else {
                    1
                };
                *port_lines.entry(p).or_default() += 1;
            }
        }

        for p in 1..=n_in {
            let count = port_lines.get(&p).copied().unwrap_or(0);
            if count == 0 {
                let msg = format!(
                    "Input port {} of block '{}' (sid '{}') is unconnected -> {} -> connect input port or supply a constant input",
                    p,
                    b.name,
                    b.sid,
                    if is_schema4 {
                        "strict schema_version >= 4 requires all input ports to be connected"
                    } else {
                        "unconnected port defaults to 0.0 in schema_version < 4"
                    }
                );
                let fix = FixAction::ConnectPortToDefaultZero {
                    block: BlockRef::new(b.sid.clone(), b.name.clone()),
                    port: p,
                };
                if is_schema4 {
                    diags.push(Diagnostic::error("E-PORT", msg, vec![b.sid.clone()]).with_fix(fix));
                } else {
                    diags.push(
                        Diagnostic::warning("W-PORT", msg, vec![b.sid.clone()]).with_fix(fix),
                    );
                }
            } else if count > 1 {
                let msg = format!(
                    "Input port {} of block '{}' (sid '{}') has {} incoming lines -> multiple lines driving one input port is forbidden -> insert a Sum or Mux block",
                    p, b.name, b.sid, count
                );
                if is_schema4 {
                    diags.push(Diagnostic::error("E-MULTI", msg, vec![b.sid.clone()]));
                } else {
                    diags.push(Diagnostic::warning("W-MULTI", msg, vec![b.sid.clone()]));
                }
            }
        }
    }

    // 3. Electrical domain and netlist checks
    let has_elec = diagram.blocks.iter().any(|b| b.block_type.is_electrical());
    if has_elec {
        // Variable-step solver incompatibility
        if let Some(cfg) = config {
            if cfg.solver.is_variable_step() {
                diags.push(Diagnostic::error(
                    "E-SOLVER",
                    format!(
                        "SimulationConfig specifies variable-step solver '{:?}' with electrical domain blocks -> electrical circuits currently require a fixed-step solver (Euler or RK4) -> select FixedStepEuler or FixedStepRk4",
                        cfg.solver
                    ),
                    Vec::new(),
                ));
            }
        }

        let net_res = resolve_electrical_nets(diagram);
        diags.extend(net_res.diagnostics);

        if let Some(topo) = &net_res.topology {
            // Check for floating nets / islands without Ground (E-FLOAT)
            let has_ground = diagram
                .blocks
                .iter()
                .any(|b| b.block_type == BlockType::Ground);
            if !has_ground {
                let elec_sids: Vec<String> = diagram
                    .blocks
                    .iter()
                    .filter(|b| b.block_type.is_electrical())
                    .map(|b| b.sid.clone())
                    .collect();
                if !elec_sids.is_empty() {
                    let first_pos = diagram
                        .blocks
                        .iter()
                        .find(|b| b.block_type.is_electrical())
                        .and_then(|b| b.position.as_ref())
                        .map(|p| (p[0] as f32, (p[1] + 80.0) as f32))
                        .unwrap_or((100.0, 100.0));
                    diags.push(
                        Diagnostic::error(
                            "E-FLOAT",
                            format!(
                                "Electrical circuit containing blocks {:?} has no DC path to ground -> every electrical circuit requires a Ground block (Node 0) -> add a Ground block",
                                elec_sids
                            ),
                            elec_sids,
                        )
                        .with_fix(FixAction::AddGroundToNet {
                            net_id: 1,
                            position: first_pos,
                        }),
                    );
                }
            }

            // Topological KVL check: E-VLOOP
            // Detect cycles of ideal voltage sources and closed switches without series resistance.
            let mut v_branches = Vec::new();
            for b in &diagram.blocks {
                let is_v_branch = matches!(
                    b.block_type,
                    BlockType::DcVoltageSource
                        | BlockType::AcVoltageSource
                        | BlockType::VoltageControlledVoltageSource
                        | BlockType::CurrentSensor
                );
                if is_v_branch {
                    let term0 = (b.sid.clone(), 0);
                    let term1 = (b.sid.clone(), 1);
                    if let (Some(&n0), Some(&n1)) = (
                        topo.terminal_to_net.get(&term0),
                        topo.terminal_to_net.get(&term1),
                    ) {
                        v_branches.push((b.sid.clone(), b.name.clone(), n0, n1));
                    }
                }
            }

            // Check for zero-impedance loop
            let mut parent: HashMap<usize, usize> = HashMap::new();
            fn find_root(parent: &mut HashMap<usize, usize>, mut i: usize) -> usize {
                let mut root = i;
                while let Some(&p) = parent.get(&root) {
                    root = p;
                }
                while let Some(&p) = parent.get(&i) {
                    parent.insert(i, root);
                    i = p;
                }
                root
            }

            let mut loop_branch = None;
            for (sid, name, u, v) in &v_branches {
                if u == v {
                    // Self-loop voltage source
                    loop_branch = Some((sid.clone(), name.clone()));
                    break;
                }
                let ru = find_root(&mut parent, *u);
                let rv = find_root(&mut parent, *v);
                if ru == rv {
                    // Cycle of ideal voltage branches!
                    loop_branch = Some((sid.clone(), name.clone()));
                    break;
                } else {
                    parent.insert(ru, rv);
                }
            }

            if let Some((v_sid, v_name)) = loop_branch {
                diags.push(
                    Diagnostic::error(
                        "E-VLOOP",
                        format!(
                            "Ideal voltage source loop detected involving block '{}' (sid '{}') without series impedance -> violates Kirchhoff's Voltage Law (KVL) -> insert a small series resistor",
                            v_name, v_sid
                        ),
                        vec![v_sid.clone()],
                    )
                    .with_fix(FixAction::InsertSeriesResistor {
                        branch_id: BlockRef::new(v_sid, v_name),
                        value: 1e-3,
                    }),
                );
            }

            // Topological KCL check: E-ICUT
            // Detect current sources with no parallel discharge path (isolated terminal or cut-set)
            for b in &diagram.blocks {
                let is_i_source = matches!(
                    b.block_type,
                    BlockType::DcCurrentSource | BlockType::VoltageControlledCurrentSource
                );
                if is_i_source {
                    let term0 = (b.sid.clone(), 0);
                    let term1 = (b.sid.clone(), 1);
                    let n0 = topo.terminal_to_net.get(&term0).copied();
                    let n1 = topo.terminal_to_net.get(&term1).copied();

                    // If either terminal is connected to an open net where only this current source is present
                    for &net_opt in &[n0, n1] {
                        if let Some(net) = net_opt {
                            let attached_terms: Vec<_> = topo
                                .terminal_to_net
                                .iter()
                                .filter(|(_, &n)| n == net)
                                .map(|(t, _)| t)
                                .collect();
                            if attached_terms.len() <= 1 {
                                // Current source open terminal!
                                diags.push(
                                    Diagnostic::error(
                                        "E-ICUT",
                                        format!(
                                            "Current source '{}' (sid '{}') is connected to open node {} with no return path -> violates Kirchhoff's Current Law (KCL) -> add parallel load resistance",
                                            b.name, b.sid, net
                                        ),
                                        vec![b.sid.clone()],
                                    )
                                    .with_fix(FixAction::InsertParallelResistor {
                                        node_id: net,
                                        value: 1e6,
                                    }),
                                );
                                break;
                            }
                        }
                    }
                }
            }

            // Step size heuristic check: W-STEP
            // tau_min = min(R*C, L/R). If h > 2 * tau_min, warn W-STEP.
            if let Some(cfg) = config {
                let h = cfg.step_size;
                let mut tau_min = f64::INFINITY;
                let mut step_sids = Vec::new();

                let mut resistors = Vec::new();
                let mut capacitors = Vec::new();
                let mut inductors = Vec::new();

                for b in &diagram.blocks {
                    match b.block_type {
                        BlockType::Resistor => {
                            if let Some(r) = b
                                .get_param("Resistance")
                                .and_then(|s| s.parse::<f64>().ok())
                            {
                                if r > 0.0 {
                                    let n0 = topo.terminal_to_net.get(&(b.sid.clone(), 0)).copied();
                                    let n1 = topo.terminal_to_net.get(&(b.sid.clone(), 1)).copied();
                                    resistors.push((b.sid.clone(), r, n0, n1));
                                }
                            }
                        }
                        BlockType::Capacitor => {
                            if let Some(c) = b
                                .get_param("Capacitance")
                                .and_then(|s| s.parse::<f64>().ok())
                            {
                                if c > 0.0 {
                                    let n0 = topo.terminal_to_net.get(&(b.sid.clone(), 0)).copied();
                                    let n1 = topo.terminal_to_net.get(&(b.sid.clone(), 1)).copied();
                                    capacitors.push((b.sid.clone(), c, n0, n1));
                                }
                            }
                        }
                        BlockType::Inductor => {
                            if let Some(l) = b
                                .get_param("Inductance")
                                .and_then(|s| s.parse::<f64>().ok())
                            {
                                if l > 0.0 {
                                    let n0 = topo.terminal_to_net.get(&(b.sid.clone(), 0)).copied();
                                    let n1 = topo.terminal_to_net.get(&(b.sid.clone(), 1)).copied();
                                    inductors.push((b.sid.clone(), l, n0, n1));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Check RC pairs sharing a net
                for (r_sid, r, rn0, rn1) in &resistors {
                    for (c_sid, c, cn0, cn1) in &capacitors {
                        let connected = (rn0.is_some() && (*rn0 == *cn0 || *rn0 == *cn1))
                            || (rn1.is_some() && (*rn1 == *cn0 || *rn1 == *cn1));
                        if connected {
                            let tau = r * c;
                            if tau < tau_min {
                                tau_min = tau;
                                step_sids = vec![r_sid.clone(), c_sid.clone()];
                            }
                        }
                    }
                    for (l_sid, l, ln0, ln1) in &inductors {
                        let connected = (rn0.is_some() && (*rn0 == *ln0 || *rn0 == *ln1))
                            || (rn1.is_some() && (*rn1 == *ln0 || *rn1 == *ln1));
                        if connected {
                            let tau = l / r;
                            if tau < tau_min {
                                tau_min = tau;
                                step_sids = vec![r_sid.clone(), l_sid.clone()];
                            }
                        }
                    }
                }

                if tau_min.is_finite() && h > 2.0 * tau_min {
                    diags.push(Diagnostic::warning(
                        "W-STEP",
                        format!(
                            "Step size h ({h:.6e} s) exceeds 2*tau_min ({tau_min:.6e} s); expect numerical oscillation or damping error with Backward Euler / Trapezoidal. Recommended step <= {:.6e} s.",
                            tau_min / 5.0
                        ),
                        step_sids,
                    ));
                }
            }
        }
    }

    // 4. Discrete blocks & measurement blocks checks
    for b in &diagram.blocks {
        match b.block_type {
            BlockType::Rms | BlockType::Mean | BlockType::PowerMeasurement | BlockType::Thd => {
                let f = b
                    .get_param("FundamentalFrequency")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(50.0);
                let ts = b
                    .get_param("SampleTime")
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| config.map(|c| c.step_size))
                    .unwrap_or(1e-3);

                if f <= 0.0 || ts <= 0.0 {
                    diags.push(Diagnostic::error(
                        "E-PARAM",
                        format!("Measurement block '{}' FundamentalFrequency and SampleTime must be > 0", b.name),
                        vec![b.sid.clone()],
                    ));
                } else {
                    let ratio = (1.0 / f) / ts;
                    let nw = ratio.round() as usize;
                    if nw < 4 {
                        diags.push(Diagnostic::error(
                            "E-PARAM",
                            format!(
                                "Measurement block '{}' window size Nw = {} is too small (< 4) -> increase sampling rate or decrease fundamental frequency",
                                b.name, nw
                            ),
                            vec![b.sid.clone()],
                        ));
                    } else if (ratio - nw as f64).abs() > 1e-6 * (nw as f64) {
                        diags.push(Diagnostic::warning(
                            "W-WINDOW",
                            format!(
                                "Measurement block '{}' window duration T = {} s is not an integer multiple of sample time Ts = {} s (ratio = {:.4}, rounded to {}) -> ripple expected",
                                b.name, 1.0 / f, ts, ratio, nw
                            ),
                            vec![b.sid.clone()],
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    diags
}

/// Applies an automated fix action atomically to a block diagram.
pub fn apply_fix(diagram: &mut BlockDiagram, fix: &FixAction) -> Result<(), SimError> {
    match fix {
        FixAction::Message(_) => Ok(()),
        FixAction::AddGroundToNet { net_id, position } => {
            let net_res = resolve_electrical_nets(diagram);
            let topo_opt = net_res.topology;

            // Find a terminal belonging to net_id
            let target_term = topo_opt
                .as_ref()
                .and_then(|topo| {
                    topo.terminal_to_net
                        .iter()
                        .find(|(_, &n)| n == *net_id)
                        .map(|(t, _)| t.clone())
                        .or_else(|| topo.terminal_to_net.keys().next().cloned())
                })
                .or_else(|| {
                    diagram
                        .blocks
                        .iter()
                        .find(|b| b.block_type.is_electrical() && b.block_type != BlockType::Ground)
                        .map(|b| (b.sid.clone(), 0))
                });

            let gnd_sid = format!("gnd_autofix_{}", diagram.blocks.len() + 1);
            let mut gnd = Block::new(&gnd_sid, "Ground", BlockType::Ground);
            gnd.position = Some([
                position.0 as f64,
                position.1 as f64,
                (position.0 + 50.0) as f64,
                (position.1 + 50.0) as f64,
            ]);
            diagram.blocks.push(gnd);

            if let Some((target_sid, term_idx)) = target_term {
                let line = Line {
                    src: PortRef {
                        block_sid: gnd_sid,
                        port_index: 0,
                        port_kind: Some(PortKind::Electrical),
                    },
                    dst: PortRef {
                        block_sid: target_sid,
                        port_index: term_idx,
                        port_kind: Some(PortKind::Electrical),
                    },
                    name: None,
                };
                diagram.lines.push(line);
            }
            Ok(())
        }
        FixAction::InsertSeriesResistor { branch_id, value } => {
            let res_sid = format!("res_series_{}", diagram.blocks.len() + 1);
            let mut res = Block::new(
                &res_sid,
                format!("R_series_{}", branch_id.name),
                BlockType::Resistor,
            );
            res.parameters
                .insert("Resistance".to_string(), value.to_string());

            if let Some(target) = diagram.find_block(&branch_id.sid) {
                if let Some(pos) = &target.position {
                    res.position = Some([pos[0] + 80.0, pos[1], pos[2] + 80.0, pos[3]]);
                }
            }
            diagram.blocks.push(res);

            // Re-route one line connected to terminal 0 of the target branch block
            let mut rewired = false;
            for line in &mut diagram.lines {
                if line.src.block_sid == branch_id.sid && line.src.port_index == 0 {
                    line.src.block_sid = res_sid.clone();
                    line.src.port_index = 1;
                    rewired = true;
                    break;
                } else if line.dst.block_sid == branch_id.sid && line.dst.port_index == 0 {
                    line.dst.block_sid = res_sid.clone();
                    line.dst.port_index = 1;
                    rewired = true;
                    break;
                }
            }

            // Connect resistor terminal 0 to target branch terminal 0
            diagram.lines.push(Line {
                src: PortRef {
                    block_sid: res_sid,
                    port_index: 0,
                    port_kind: Some(PortKind::Electrical),
                },
                dst: PortRef {
                    block_sid: branch_id.sid.clone(),
                    port_index: 0,
                    port_kind: Some(PortKind::Electrical),
                },
                name: None,
            });

            let _ = rewired;
            Ok(())
        }
        FixAction::InsertParallelResistor { node_id, value } => {
            let net_res = resolve_electrical_nets(diagram);
            let topo_opt = net_res.topology;
            let target_term = topo_opt.as_ref().and_then(|topo| {
                topo.terminal_to_net
                    .iter()
                    .find(|(_, &n)| n == *node_id)
                    .map(|(t, _)| t.clone())
                    .or_else(|| topo.terminal_to_net.keys().next().cloned())
            });

            let res_sid = format!("res_par_{}", diagram.blocks.len() + 1);
            let mut res = Block::new(
                &res_sid,
                format!("R_parallel_{}", node_id),
                BlockType::Resistor,
            );
            res.parameters
                .insert("Resistance".to_string(), value.to_string());
            diagram.blocks.push(res);

            if let Some((target_sid, term_idx)) = target_term {
                // Connect terminal 0 to node
                diagram.lines.push(Line {
                    src: PortRef {
                        block_sid: res_sid.clone(),
                        port_index: 0,
                        port_kind: Some(PortKind::Electrical),
                    },
                    dst: PortRef {
                        block_sid: target_sid,
                        port_index: term_idx,
                        port_kind: Some(PortKind::Electrical),
                    },
                    name: None,
                });

                // Connect terminal 1 to Ground if ground exists
                let gnd = diagram
                    .blocks
                    .iter()
                    .find(|b| b.block_type == BlockType::Ground);
                if let Some(g) = gnd {
                    diagram.lines.push(Line {
                        src: PortRef {
                            block_sid: res_sid,
                            port_index: 1,
                            port_kind: Some(PortKind::Electrical),
                        },
                        dst: PortRef {
                            block_sid: g.sid.clone(),
                            port_index: 0,
                            port_kind: Some(PortKind::Electrical),
                        },
                        name: None,
                    });
                }
            }
            Ok(())
        }
        FixAction::ConnectPortToDefaultZero { block, port } => {
            let const_sid = format!("const_zero_{}", diagram.blocks.len() + 1);
            let mut c = Block::new(&const_sid, "Zero", BlockType::Constant);
            c.parameters.insert("Value".to_string(), "0".to_string());
            if let Some(target) = diagram.find_block(&block.sid) {
                if let Some(pos) = &target.position {
                    c.position = Some([pos[0] - 100.0, pos[1], pos[2] - 100.0, pos[3]]);
                }
            }
            diagram.blocks.push(c);

            diagram.lines.push(Line {
                src: PortRef {
                    block_sid: const_sid,
                    port_index: 1,
                    port_kind: Some(PortKind::Signal),
                },
                dst: PortRef {
                    block_sid: block.sid.clone(),
                    port_index: *port,
                    port_kind: Some(PortKind::Signal),
                },
                name: None,
            });
            Ok(())
        }
    }
}
