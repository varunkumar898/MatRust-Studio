//! Modified Nodal Analysis (MNA) solver for linear electrical circuits.
//!
//! References:
//! - Ho, Ruehli, Brennan (1975): "The Modified Nodal Approach to Network Analysis"
//! - Dommel (1969): "Digital Computer Solution of Electromagnetic Transients in Single-and Multiphase Networks"
//! - Vlach & Singhal (1983): "Computer Methods for Circuit Analysis and Design"

use crate::model::{BlockDiagram, BlockType};
use crate::netlist::resolve_electrical_nets;
use crate::solver::SimError;
use std::collections::HashMap;

/// Dense LU decomposition with partial pivoting: P * A = L * U.
/// Stored in row-major order with zero per-step heap allocations during solve.
#[derive(Debug, Clone, PartialEq)]
pub struct LuDecomposition {
    pub size: usize,
    pub lu: Vec<f64>,
    pub pivots: Vec<usize>,
}

impl LuDecomposition {
    /// Performs Gaussian elimination with partial pivoting.
    /// Returns `Err(SimError::Singular)` if a zero pivot (< 1e-12) is encountered.
    pub fn decompose(matrix: &[f64], size: usize) -> Result<Self, SimError> {
        let mut lu = matrix.to_vec();
        let mut pivots = vec![0; size];

        for k in 0..size {
            // Find pivot row with maximum absolute value in column k
            let mut max_val = 0.0;
            let mut pivot_row = k;
            for i in k..size {
                let val = lu[i * size + k].abs();
                if val > max_val {
                    max_val = val;
                    pivot_row = i;
                }
            }

            if max_val < 1e-12 {
                return Err(SimError::singular(format!(
                    "Zero pivot ({max_val}) encountered at MNA row/column index {k}. Why: The electrical circuit has no unique solution. Common causes include a floating subcircuit with no DC path to Ground, an ideal loop of voltage sources with zero series impedance, or an ungrounded inductive node cutset. How to fix: Connect Ground to every subcircuit, add small series resistances to voltage sources, or add bleeder resistors across floating nodes."
                )));
            }

            pivots[k] = pivot_row;

            // Swap rows k and pivot_row if needed
            if pivot_row != k {
                for j in 0..size {
                    lu.swap(k * size + j, pivot_row * size + j);
                }
            }

            // Compute multipliers and update submatrix
            let pivot = lu[k * size + k];
            for i in (k + 1)..size {
                let factor = lu[i * size + k] / pivot;
                lu[i * size + k] = factor;
                for j in (k + 1)..size {
                    lu[i * size + j] -= factor * lu[k * size + j];
                }
            }
        }

        Ok(Self { size, lu, pivots })
    }

    /// Solves A * x = b using forward and back substitution.
    /// Performs ZERO heap allocations.
    pub fn solve(&self, b: &[f64], x: &mut [f64]) {
        let n = self.size;
        debug_assert_eq!(b.len(), n);
        debug_assert_eq!(x.len(), n);

        // Apply pivot permutation P to b into x
        x.copy_from_slice(b);
        for k in 0..n {
            let p = self.pivots[k];
            if p != k {
                x.swap(k, p);
            }
        }

        // Forward substitution: L * y = P * b (L has 1 on diagonal)
        for i in 0..n {
            let mut sum = x[i];
            for (j, &xj) in x.iter().enumerate().take(i) {
                sum -= self.lu[i * n + j] * xj;
            }
            x[i] = sum;
        }

        // Back substitution: U * x = y
        for i in (0..n).rev() {
            let mut sum = x[i];
            for (j, &xj) in x.iter().enumerate().take(n).skip(i + 1) {
                sum -= self.lu[i * n + j] * xj;
            }
            x[i] = sum / self.lu[i * n + i];
        }
    }
}

/// Linear resistor element.
#[derive(Debug, Clone)]
struct ResistorElement {
    pub sid: String,
    pub p_net: usize,
    pub n_net: usize,
    pub resistance: f64,
}

/// Linear capacitor element with companion model states.
#[derive(Debug, Clone)]
struct CapacitorElement {
    pub sid: String,
    pub p_net: usize,
    pub n_net: usize,
    pub capacitance: f64,
    pub v0: f64,
    pub vc: f64,
    pub ic: f64,
}

/// Linear inductor element with companion model states.
#[derive(Debug, Clone)]
struct InductorElement {
    pub sid: String,
    pub p_net: usize,
    pub n_net: usize,
    pub inductance: f64,
    pub i0: f64,
    pub vl: f64,
    pub il: f64,
}

/// DC voltage source.
#[derive(Debug, Clone)]
struct DcVoltageSourceElement {
    pub sid: String,
    pub p_net: usize,
    pub n_net: usize,
    pub voltage: f64,
    pub vsrc_idx: usize,
}

/// AC sinusoidal voltage source: v(t) = Amp * sin(2*pi*freq*t + phase_rad) + offset.
#[derive(Debug, Clone)]
struct AcVoltageSourceElement {
    pub sid: String,
    pub p_net: usize,
    pub n_net: usize,
    pub amplitude: f64,
    pub frequency: f64,
    pub phase_rad: f64,
    pub offset: f64,
    pub vsrc_idx: usize,
}

/// DC current source (injects current into positive terminal, out of negative).
#[derive(Debug, Clone)]
struct DcCurrentSourceElement {
    pub sid: String,
    pub p_net: usize,
    pub n_net: usize,
    pub current: f64,
}

/// Linear circuit MNA solver with constant-matrix fast path.
#[derive(Debug, Clone)]
pub struct LinearCircuitSolver {
    pub num_nodes: usize,
    pub num_vsrc: usize,
    pub system_size: usize,

    resistors: Vec<ResistorElement>,
    capacitors: Vec<CapacitorElement>,
    inductors: Vec<InductorElement>,
    dc_vsources: Vec<DcVoltageSourceElement>,
    ac_vsources: Vec<AcVoltageSourceElement>,
    dc_isources: Vec<DcCurrentSourceElement>,

    sid_to_terminal_nets: HashMap<String, (usize, usize)>,

    // Pre-factorized LU matrices for constant step size
    lu_init: LuDecomposition,
    lu_be: LuDecomposition,
    lu_trap: LuDecomposition,

    // Pre-allocated runtime buffers (Zero heap allocation during step())
    rhs: Vec<f64>,
    sol: Vec<f64>,
    pub node_voltages: Vec<f64>, // index 0 is ground (0.0 V), index 1..num_nodes are non-ground nodes
    vsrc_currents: Vec<f64>,
}

impl LinearCircuitSolver {
    /// Compiles linear circuit blocks from diagram into an MNA system.
    /// Returns `Ok(None)` if no electrical blocks are present.
    pub fn compile(diagram: &BlockDiagram, dt: f64) -> Result<Option<Self>, SimError> {
        let electrical_blocks: Vec<_> = diagram
            .blocks
            .iter()
            .filter(|b| b.block_type.is_electrical())
            .collect();

        if electrical_blocks.is_empty() {
            return Ok(None);
        }

        // Check for unsupported nonlinear blocks prior to M6
        for b in &electrical_blocks {
            if matches!(b.block_type, BlockType::Diode | BlockType::IdealSwitch) {
                return Err(SimError::unsupported(format!(
                    "Nonlinear electrical block '{}' ({:?}) arrives in Milestone M6; Milestone M5 supports linear circuits (Resistor, Capacitor, Inductor, DC/AC Voltage Source, DC Current Source, Ground).",
                    b.name, b.block_type
                )));
            }
        }

        // Resolve electrical nets
        let net_result = resolve_electrical_nets(diagram);
        for diag in &net_result.diagnostics {
            if diag.severity == crate::diagnostics::DiagnosticSeverity::Error {
                return Err(SimError::parameter(format!(
                    "Electrical topology error ({}): {}",
                    diag.code, diag.message
                )));
            }
        }

        let topology = net_result.topology.ok_or_else(|| {
            SimError::parameter(
                "Failed to construct valid electrical topology due to netlist diagnostics."
                    .to_string(),
            )
        })?;

        // Ground is node 0. Count non-ground nodes.
        let mut max_net = 0;
        for &net in topology.net_ids.iter() {
            if net > max_net {
                max_net = net;
            }
        }
        let num_nodes = max_net; // 1..num_nodes

        // Inventory components and assign voltage source branch current indices
        let mut resistors = Vec::new();
        let mut capacitors = Vec::new();
        let mut inductors = Vec::new();
        let mut dc_vsources = Vec::new();
        let mut ac_vsources = Vec::new();
        let mut dc_isources = Vec::new();
        let mut sid_to_terminal_nets = HashMap::new();

        let mut vsrc_idx = 0;

        for b in &electrical_blocks {
            let p_net = *topology
                .terminal_to_net
                .get(&(b.sid.clone(), 0))
                .unwrap_or(&0);
            let n_net = *topology
                .terminal_to_net
                .get(&(b.sid.clone(), 1))
                .unwrap_or(&0);
            sid_to_terminal_nets.insert(b.sid.clone(), (p_net, n_net));

            match b.block_type {
                BlockType::Resistor => {
                    let r = b
                        .get_param("Resistance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1000.0);
                    resistors.push(ResistorElement {
                        sid: b.sid.clone(),
                        p_net,
                        n_net,
                        resistance: r,
                    });
                }
                BlockType::Capacitor => {
                    let c = b
                        .get_param("Capacitance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1e-6);
                    let v0 = b
                        .get_param("InitialVoltage")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    capacitors.push(CapacitorElement {
                        sid: b.sid.clone(),
                        p_net,
                        n_net,
                        capacitance: c,
                        v0,
                        vc: v0,
                        ic: 0.0,
                    });
                }
                BlockType::Inductor => {
                    let l = b
                        .get_param("Inductance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1e-3);
                    let i0 = b
                        .get_param("InitialCurrent")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    inductors.push(InductorElement {
                        sid: b.sid.clone(),
                        p_net,
                        n_net,
                        inductance: l,
                        i0,
                        vl: 0.0,
                        il: i0,
                    });
                }
                BlockType::DcVoltageSource => {
                    let v = b
                        .get_param("Voltage")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(10.0);
                    dc_vsources.push(DcVoltageSourceElement {
                        sid: b.sid.clone(),
                        p_net,
                        n_net,
                        voltage: v,
                        vsrc_idx,
                    });
                    vsrc_idx += 1;
                }
                BlockType::AcVoltageSource => {
                    let amp = b
                        .get_param("Amplitude")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(120.0);
                    let freq = b
                        .get_param("Frequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(60.0);
                    let phase = b
                        .get_param("Phase")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let bias = b
                        .get_param("Bias")
                        .or_else(|| b.get_param("Offset"))
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    ac_vsources.push(AcVoltageSourceElement {
                        sid: b.sid.clone(),
                        p_net,
                        n_net,
                        amplitude: amp,
                        frequency: freq,
                        phase_rad: phase.to_radians(),
                        offset: bias,
                        vsrc_idx,
                    });
                    vsrc_idx += 1;
                }
                BlockType::DcCurrentSource => {
                    let i = b
                        .get_param("Current")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    dc_isources.push(DcCurrentSourceElement {
                        sid: b.sid.clone(),
                        p_net,
                        n_net,
                        current: i,
                    });
                }
                BlockType::Ground => {
                    // Datum node 0 already handled in netlist resolution
                }
                _ => {}
            }
        }

        let num_vsrc = vsrc_idx;
        let system_size = num_nodes + num_vsrc;

        if system_size == 0 {
            return Err(SimError::singular(
                "Circuit system size is zero (no non-ground nodes or sources).".to_string(),
            ));
        }

        // Helper to assemble MNA matrix A for a given integration rule (is_be: true for Backward Euler, false for Trapezoidal)
        let assemble_matrix = |is_be: bool| -> Vec<f64> {
            let mut a = vec![0.0; system_size * system_size];

            // 1. Stamp resistors: G = 1 / R
            for r in &resistors {
                let g = 1.0 / r.resistance;
                stamp_conductance(&mut a, system_size, r.p_net, r.n_net, g);
            }

            // 2. Stamp capacitors: G_eq = C/dt (BE) or 2C/dt (Trap)
            for c in &capacitors {
                let g = if is_be {
                    c.capacitance / dt
                } else {
                    2.0 * c.capacitance / dt
                };
                stamp_conductance(&mut a, system_size, c.p_net, c.n_net, g);
            }

            // 3. Stamp inductors: G_eq = dt/L (BE) or dt/(2L) (Trap)
            for l in &inductors {
                let g = if is_be {
                    dt / l.inductance
                } else {
                    dt / (2.0 * l.inductance)
                };
                stamp_conductance(&mut a, system_size, l.p_net, l.n_net, g);
            }

            // 4. Stamp DC and AC voltage source connections: B and B^T
            for vsrc in &dc_vsources {
                stamp_vsource(
                    &mut a,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
            }
            for vsrc in &ac_vsources {
                stamp_vsource(
                    &mut a,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
            }

            a
        };

        let assemble_matrix_init = || -> Vec<f64> {
            let mut a = vec![0.0; system_size * system_size];

            for r in &resistors {
                let g = 1.0 / r.resistance;
                stamp_conductance(&mut a, system_size, r.p_net, r.n_net, g);
            }

            for c in &capacitors {
                stamp_conductance(&mut a, system_size, c.p_net, c.n_net, 1e9);
            }

            for l in &inductors {
                stamp_conductance(&mut a, system_size, l.p_net, l.n_net, 1e-9);
            }

            for vsrc in &dc_vsources {
                stamp_vsource(
                    &mut a,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
            }
            for vsrc in &ac_vsources {
                stamp_vsource(
                    &mut a,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
            }

            a
        };

        let a_init = assemble_matrix_init();
        let a_be = assemble_matrix(true);
        let a_trap = assemble_matrix(false);

        let lu_init = LuDecomposition::decompose(&a_init, system_size)?;
        let lu_be = LuDecomposition::decompose(&a_be, system_size)?;
        let lu_trap = LuDecomposition::decompose(&a_trap, system_size)?;

        Ok(Some(Self {
            num_nodes,
            num_vsrc,
            system_size,
            resistors,
            capacitors,
            inductors,
            dc_vsources,
            ac_vsources,
            dc_isources,
            sid_to_terminal_nets,
            lu_init,
            lu_be,
            lu_trap,
            rhs: vec![0.0; system_size],
            sol: vec![0.0; system_size],
            node_voltages: vec![0.0; num_nodes + 1],
            vsrc_currents: vec![0.0; num_vsrc],
        }))
    }

    /// Solves the circuit at t = t0 to establish initial node voltages, branch voltages,
    /// and branch currents consistent with component initial conditions (V0, I0) and source voltages.
    /// Performs ZERO heap allocations.
    pub fn init_solve(&mut self, t0: f64) -> Result<(), SimError> {
        let n = self.num_nodes;
        self.rhs.fill(0.0);

        for isrc in &self.dc_isources {
            let i_val = isrc.current;
            if isrc.p_net > 0 {
                self.rhs[isrc.p_net - 1] += i_val;
            }
            if isrc.n_net > 0 {
                self.rhs[isrc.n_net - 1] -= i_val;
            }
        }

        for c in &self.capacitors {
            let g = 1e9;
            let i_eq = -g * c.v0;
            if c.p_net > 0 {
                self.rhs[c.p_net - 1] -= i_eq;
            }
            if c.n_net > 0 {
                self.rhs[c.n_net - 1] += i_eq;
            }
        }

        for l in &self.inductors {
            let i_eq = l.i0;
            if l.p_net > 0 {
                self.rhs[l.p_net - 1] -= i_eq;
            }
            if l.n_net > 0 {
                self.rhs[l.n_net - 1] += i_eq;
            }
        }

        for vsrc in &self.dc_vsources {
            self.rhs[n + vsrc.vsrc_idx] = vsrc.voltage;
        }

        for vsrc in &self.ac_vsources {
            let v_val = vsrc.amplitude
                * (2.0 * std::f64::consts::PI * vsrc.frequency * t0 + vsrc.phase_rad).sin()
                + vsrc.offset;
            self.rhs[n + vsrc.vsrc_idx] = v_val;
        }

        self.lu_init.solve(&self.rhs, &mut self.sol);

        self.node_voltages[0] = 0.0;
        for i in 1..=n {
            self.node_voltages[i] = self.sol[i - 1];
        }
        for (k, v) in self.vsrc_currents.iter_mut().enumerate() {
            *v = self.sol[n + k];
        }

        for c in &mut self.capacitors {
            let vp = self.node_voltages[c.p_net];
            let vn = self.node_voltages[c.n_net];
            c.vc = vp - vn;
            c.ic = 1e9 * (c.vc - c.v0);
        }

        for l in &mut self.inductors {
            let vp = self.node_voltages[l.p_net];
            let vn = self.node_voltages[l.n_net];
            l.vl = vp - vn;
            l.il = l.i0 + 1e-9 * l.vl;
        }

        Ok(())
    }

    /// Advances the linear circuit by one fixed time step dt.
    /// Uses Backward Euler on step 1 to prevent cold-start trapezoidal ringing,
    /// then Trapezoidal rule for all subsequent steps.
    /// Performs ZERO heap allocations.
    pub fn step(&mut self, t: f64, dt: f64, is_first_step: bool) -> Result<(), SimError> {
        let n = self.num_nodes;
        self.rhs.fill(0.0);

        // 1. Independent DC current sources
        for isrc in &self.dc_isources {
            let i_val = isrc.current;
            if isrc.p_net > 0 {
                self.rhs[isrc.p_net - 1] += i_val;
            }
            if isrc.n_net > 0 {
                self.rhs[isrc.n_net - 1] -= i_val;
            }
        }

        // 2. Companion model Norton equivalent history currents
        for c in &self.capacitors {
            let i_eq = if is_first_step {
                -(c.capacitance / dt) * c.v0
            } else {
                let g = 2.0 * c.capacitance / dt;
                -g * c.vc - c.ic
            };

            // Current source in parallel with G flows from p to n
            if c.p_net > 0 {
                self.rhs[c.p_net - 1] -= i_eq;
            }
            if c.n_net > 0 {
                self.rhs[c.n_net - 1] += i_eq;
            }
        }

        for l in &self.inductors {
            let i_eq = if is_first_step {
                l.i0
            } else {
                let g = dt / (2.0 * l.inductance);
                l.il + g * l.vl
            };

            // Current source in parallel with G flows from p to n
            if l.p_net > 0 {
                self.rhs[l.p_net - 1] -= i_eq;
            }
            if l.n_net > 0 {
                self.rhs[l.n_net - 1] += i_eq;
            }
        }

        // 3. Independent voltage sources
        for vsrc in &self.dc_vsources {
            self.rhs[n + vsrc.vsrc_idx] = vsrc.voltage;
        }

        for vsrc in &self.ac_vsources {
            let v_val = vsrc.amplitude
                * (2.0 * std::f64::consts::PI * vsrc.frequency * t + vsrc.phase_rad).sin()
                + vsrc.offset;
            self.rhs[n + vsrc.vsrc_idx] = v_val;
        }

        // 4. Solve Ax = z via pre-factorized LU (Zero heap allocation)
        if is_first_step {
            self.lu_be.solve(&self.rhs, &mut self.sol);
        } else {
            self.lu_trap.solve(&self.rhs, &mut self.sol);
        }

        // 5. Store node voltages and source currents
        self.node_voltages[0] = 0.0;
        for i in 1..=n {
            self.node_voltages[i] = self.sol[i - 1];
        }
        for (k, v) in self.vsrc_currents.iter_mut().enumerate() {
            *v = self.sol[n + k];
        }

        // 6. Update reactive element history states
        for c in &mut self.capacitors {
            let vp = self.node_voltages[c.p_net];
            let vn = self.node_voltages[c.n_net];
            let v_new = vp - vn;
            let i_new = if is_first_step {
                let g = c.capacitance / dt;
                let i_eq = -g * c.v0;
                g * v_new + i_eq
            } else {
                let g = 2.0 * c.capacitance / dt;
                let i_eq = -g * c.vc - c.ic;
                g * v_new + i_eq
            };
            c.vc = v_new;
            c.ic = i_new;
        }

        for l in &mut self.inductors {
            let vp = self.node_voltages[l.p_net];
            let vn = self.node_voltages[l.n_net];
            let v_new = vp - vn;
            let i_new = if is_first_step {
                let g = dt / l.inductance;
                let i_eq = l.i0;
                g * v_new + i_eq
            } else {
                let g = dt / (2.0 * l.inductance);
                let i_eq = l.il + g * l.vl;
                g * v_new + i_eq
            };
            l.vl = v_new;
            l.il = i_new;
        }

        Ok(())
    }

    /// Returns the potential of node `net` in Volts (Ground is Net 0 = 0.0 V).
    pub fn node_voltage(&self, net: usize) -> f64 {
        if net < self.node_voltages.len() {
            self.node_voltages[net]
        } else {
            0.0
        }
    }

    /// Returns the branch voltage across a block's terminals (v_p - v_n) in Volts.
    pub fn branch_voltage(&self, block_sid: &str) -> Option<f64> {
        let &(p_net, n_net) = self.sid_to_terminal_nets.get(block_sid)?;
        Some(self.node_voltage(p_net) - self.node_voltage(n_net))
    }

    /// Returns the branch current flowing from positive to negative terminal in Amperes.
    pub fn branch_current(&self, block_sid: &str) -> Option<f64> {
        // Resistors: i = (vp - vn) / R
        if let Some(r) = self.resistors.iter().find(|r| r.sid == block_sid) {
            let v = self.node_voltage(r.p_net) - self.node_voltage(r.n_net);
            return Some(v / r.resistance);
        }

        // Capacitors: stored ic
        if let Some(c) = self.capacitors.iter().find(|c| c.sid == block_sid) {
            return Some(c.ic);
        }

        // Inductors: stored il
        if let Some(l) = self.inductors.iter().find(|l| l.sid == block_sid) {
            return Some(l.il);
        }

        // DC Voltage source: current flowing through source
        if let Some(vsrc) = self.dc_vsources.iter().find(|v| v.sid == block_sid) {
            return Some(self.vsrc_currents[vsrc.vsrc_idx]);
        }

        // AC Voltage source: current flowing through source
        if let Some(vsrc) = self.ac_vsources.iter().find(|v| v.sid == block_sid) {
            return Some(self.vsrc_currents[vsrc.vsrc_idx]);
        }

        // DC Current source
        if let Some(isrc) = self.dc_isources.iter().find(|i| i.sid == block_sid) {
            return Some(isrc.current);
        }

        None
    }
}

/// Helper: Stamps conductance G into row-major matrix A.
fn stamp_conductance(a: &mut [f64], size: usize, p: usize, n: usize, g: f64) {
    if p > 0 {
        a[(p - 1) * size + (p - 1)] += g;
    }
    if n > 0 {
        a[(n - 1) * size + (n - 1)] += g;
    }
    if p > 0 && n > 0 {
        a[(p - 1) * size + (n - 1)] -= g;
        a[(n - 1) * size + (p - 1)] -= g;
    }
}

/// Helper: Stamps independent voltage source connections (+1 / -1) into MNA submatrices B and B^T.
fn stamp_vsource(
    a: &mut [f64],
    size: usize,
    num_nodes: usize,
    p: usize,
    n: usize,
    vsrc_idx: usize,
) {
    let col = num_nodes + vsrc_idx;
    let row = num_nodes + vsrc_idx;

    if p > 0 {
        a[(p - 1) * size + col] += 1.0;
        a[row * size + (p - 1)] += 1.0;
    }
    if n > 0 {
        a[(n - 1) * size + col] -= 1.0;
        a[row * size + (n - 1)] -= 1.0;
    }
}
