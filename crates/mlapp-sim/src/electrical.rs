use crate::model::{BlockDiagram, BlockType};
use crate::netlist::resolve_electrical_nets;
use crate::solver::SimError;
use std::collections::HashMap;

/// Dense LU decomposition with partial pivoting ($PA = LU$).
/// Inverts and solves linear systems $A x = b$ in $O(N^3)$ decomposition
/// and $O(N^2)$ substitution with zero heap allocations during substitution.
#[derive(Debug, Clone)]
pub struct LuDecomposition {
    pub size: usize,
    pub lu: Vec<f64>,
    pub pivots: Vec<usize>,
}

impl LuDecomposition {
    /// Decomposes row-major square matrix `a` of dimension `size x size`.
    /// Returns `Err(SimError::Singular)` if a zero or near-zero pivot (< 1e-12) is encountered.
    pub fn decompose(a: &[f64], size: usize) -> Result<Self, SimError> {
        let n = size;
        let mut lu = a.to_vec();
        let mut pivots = vec![0; n];

        for i in 0..n {
            // Find pivot in column i at or below row i
            let mut max_val = lu[i * n + i].abs();
            let mut pivot_row = i;

            for (k, row) in lu.chunks_exact(n).enumerate().take(n).skip(i + 1) {
                let val = row[i].abs();
                if val > max_val {
                    max_val = val;
                    pivot_row = k;
                }
            }

            if max_val < 1e-12 {
                return Err(SimError::singular(format!(
                    "Zero pivot ({max_val}) encountered at MNA row/column index {i}. Why: The electrical circuit has no unique solution. Common causes include a floating subcircuit with no DC path to Ground, an ideal loop of voltage sources with zero series impedance, or an ungrounded inductive node cutset. How to fix: Connect Ground to every subcircuit, add small series resistances to voltage sources, or add bleeder resistors across floating nodes."
                )));
            }

            // Swap rows in LU if necessary (including sub-diagonal L columns 0..i)
            if pivot_row != i {
                for col in 0..n {
                    lu.swap(i * n + col, pivot_row * n + col);
                }
            }
            pivots[i] = pivot_row;

            let diag = lu[i * n + i];
            for j in (i + 1)..n {
                lu[j * n + i] /= diag;
                let mult = lu[j * n + i];
                for k in (i + 1)..n {
                    let sub = mult * lu[i * n + k];
                    lu[j * n + k] -= sub;
                }
            }
        }

        Ok(Self { size, lu, pivots })
    }

    /// Solves $A x = b$ into pre-allocated buffer `x` using forward and back substitution.
    /// Performs ZERO heap allocations.
    pub fn solve(&self, b: &[f64], x: &mut [f64]) {
        let n = self.size;
        x.copy_from_slice(b);

        // Apply permutation vector P to b
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

/// Piecewise-linear diode element.
#[derive(Debug, Clone)]
pub struct DiodeElement {
    pub sid: String,
    pub name: String,
    pub p_net: usize,
    pub n_net: usize,
    pub vf: f64,
    pub ron: f64,
    pub roff: f64,
    pub gon: f64,
    pub goff: f64,
    pub is_on: bool,
    pub vd: f64,
    pub id: f64,
}

/// Ideal switch element with signal control input.
#[derive(Debug, Clone)]
pub struct IdealSwitchElement {
    pub sid: String,
    pub name: String,
    pub p_net: usize,
    pub n_net: usize,
    pub ron: f64,
    pub roff: f64,
    pub gon: f64,
    pub goff: f64,
    pub threshold: f64,
    pub ctrl_val: f64,
    pub ctrl_driver_idx: Option<usize>,
    pub is_on: bool,
    pub vs: f64,
    pub is: f64,
}

/// Linear and switching circuit MNA solver with constant-matrix fast path.
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
    pub diodes: Vec<DiodeElement>,
    pub switches: Vec<IdealSwitchElement>,

    sid_to_terminal_nets: HashMap<String, (usize, usize)>,

    // Pre-factorized LU decomposition tables for all 2^N_s topologies (where N_s <= 5)
    // Table indices correspond to switch mask m in 0..(1 << (diodes.len() + switches.len()))
    lu_init_table: Vec<Option<LuDecomposition>>,
    lu_be_table: Vec<Option<LuDecomposition>>,
    lu_trap_table: Vec<Option<LuDecomposition>>,

    // Pre-allocated runtime buffers (Zero heap allocation during step())
    rhs: Vec<f64>,
    sol: Vec<f64>,
    pub node_voltages: Vec<f64>, // index 0 is ground (0.0 V), index 1..num_nodes are non-ground nodes
    vsrc_currents: Vec<f64>,
    visited_masks: [usize; 100],
}

impl LinearCircuitSolver {
    /// Compiles circuit blocks from diagram into an MNA system.
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
        let mut diodes = Vec::new();
        let mut switches = Vec::new();
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
                BlockType::Diode => {
                    let vf = b
                        .get_param("ForwardVoltage")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.7);
                    let ron = b
                        .get_param("OnResistance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.01);
                    let roff = b
                        .get_param("OffResistance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1e6);
                    diodes.push(DiodeElement {
                        sid: b.sid.clone(),
                        name: b.name.clone(),
                        p_net,
                        n_net,
                        vf,
                        ron,
                        roff,
                        gon: 1.0 / ron,
                        goff: 1.0 / roff,
                        is_on: false,
                        vd: 0.0,
                        id: 0.0,
                    });
                }
                BlockType::IdealSwitch => {
                    let ron = b
                        .get_param("OnResistance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.01);
                    let roff = b
                        .get_param("OffResistance")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1e6);
                    let threshold = b
                        .get_param("Threshold")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.5);

                    // Find driving signal block from lines
                    let ctrl_driver_idx = diagram
                        .lines
                        .iter()
                        .find(|l| {
                            l.dst.block_sid == b.sid
                                && l.dst.port_kind != Some(crate::model::PortKind::Electrical)
                        })
                        .and_then(|l| {
                            diagram
                                .blocks
                                .iter()
                                .position(|blk| blk.sid == l.src.block_sid)
                        });

                    switches.push(IdealSwitchElement {
                        sid: b.sid.clone(),
                        name: b.name.clone(),
                        p_net,
                        n_net,
                        ron,
                        roff,
                        gon: 1.0 / ron,
                        goff: 1.0 / roff,
                        threshold,
                        ctrl_val: 0.0,
                        ctrl_driver_idx,
                        is_on: false,
                        vs: 0.0,
                        is: 0.0,
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

        let total_switches = diodes.len() + switches.len();
        if total_switches > 10 {
            return Err(SimError::unsupported(format!(
                "Circuit contains {total_switches} switches/diodes (maximum supported topologies: 2^10 = 1024)."
            )));
        }

        let total_topologies = 1usize << total_switches;
        let mut lu_init_table = Vec::with_capacity(total_topologies);
        let mut lu_be_table = Vec::with_capacity(total_topologies);
        let mut lu_trap_table = Vec::with_capacity(total_topologies);

        // Pre-factorize all 2^N_s topologies at compile time for constant-matrix fast path
        for mask in 0..total_topologies {
            // Helper to assemble matrix for given mask and integration rule
            let mut a_be = vec![0.0; system_size * system_size];
            let mut a_trap = vec![0.0; system_size * system_size];
            let mut a_init = vec![0.0; system_size * system_size];

            // 1. Resistors
            for r in &resistors {
                let g = 1.0 / r.resistance;
                stamp_conductance(&mut a_be, system_size, r.p_net, r.n_net, g);
                stamp_conductance(&mut a_trap, system_size, r.p_net, r.n_net, g);
                stamp_conductance(&mut a_init, system_size, r.p_net, r.n_net, g);
            }

            // 2. Capacitors
            for c in &capacitors {
                stamp_conductance(&mut a_be, system_size, c.p_net, c.n_net, c.capacitance / dt);
                stamp_conductance(
                    &mut a_trap,
                    system_size,
                    c.p_net,
                    c.n_net,
                    2.0 * c.capacitance / dt,
                );
                stamp_conductance(&mut a_init, system_size, c.p_net, c.n_net, 1e9);
            }

            // 3. Inductors
            for l in &inductors {
                stamp_conductance(&mut a_be, system_size, l.p_net, l.n_net, dt / l.inductance);
                stamp_conductance(
                    &mut a_trap,
                    system_size,
                    l.p_net,
                    l.n_net,
                    dt / (2.0 * l.inductance),
                );
                stamp_conductance(&mut a_init, system_size, l.p_net, l.n_net, 1e-9);
            }

            // 4. Voltage sources
            for vsrc in &dc_vsources {
                stamp_vsource(
                    &mut a_be,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
                stamp_vsource(
                    &mut a_trap,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
                stamp_vsource(
                    &mut a_init,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
            }
            for vsrc in &ac_vsources {
                stamp_vsource(
                    &mut a_be,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
                stamp_vsource(
                    &mut a_trap,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
                stamp_vsource(
                    &mut a_init,
                    system_size,
                    num_nodes,
                    vsrc.p_net,
                    vsrc.n_net,
                    vsrc.vsrc_idx,
                );
            }

            // 5. Diode conductances for this topology mask
            for (i, d) in diodes.iter().enumerate() {
                let is_on = (mask & (1 << i)) != 0;
                let g = if is_on { d.gon } else { d.goff };
                stamp_conductance(&mut a_be, system_size, d.p_net, d.n_net, g);
                stamp_conductance(&mut a_trap, system_size, d.p_net, d.n_net, g);
                stamp_conductance(&mut a_init, system_size, d.p_net, d.n_net, g);
            }

            // 6. Ideal switch conductances for this topology mask
            let n_diodes = diodes.len();
            for (j, s) in switches.iter().enumerate() {
                let is_on = (mask & (1 << (n_diodes + j))) != 0;
                let g = if is_on { s.gon } else { s.goff };
                stamp_conductance(&mut a_be, system_size, s.p_net, s.n_net, g);
                stamp_conductance(&mut a_trap, system_size, s.p_net, s.n_net, g);
                stamp_conductance(&mut a_init, system_size, s.p_net, s.n_net, g);
            }

            let be_res = LuDecomposition::decompose(&a_be, system_size);
            if mask == 0 {
                if let Err(e) = be_res {
                    return Err(e);
                }
            }
            lu_be_table.push(be_res.ok());
            lu_init_table.push(LuDecomposition::decompose(&a_init, system_size).ok());
            lu_trap_table.push(LuDecomposition::decompose(&a_trap, system_size).ok());
        }

        // If the all-off initial mask (mask 0) cannot be factored in init solve, circuit is singular
        if lu_init_table[0].is_none() {
            return Err(SimError::singular(
                "Initial circuit topology matrix is singular (no DC path to ground or redundant ideal sources)."
                    .to_string(),
            ));
        }

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
            diodes,
            switches,
            sid_to_terminal_nets,
            lu_init_table,
            lu_be_table,
            lu_trap_table,
            rhs: vec![0.0; system_size],
            sol: vec![0.0; system_size],
            node_voltages: vec![0.0; num_nodes + 1],
            vsrc_currents: vec![0.0; num_vsrc],
            visited_masks: [0; 100],
        }))
    }

    /// Computes the integer bitmask corresponding to current diode and switch states.
    #[inline]
    fn current_mask(&self) -> usize {
        let mut mask = 0usize;
        for (i, d) in self.diodes.iter().enumerate() {
            if d.is_on {
                mask |= 1 << i;
            }
        }
        let n_d = self.diodes.len();
        for (j, s) in self.switches.iter().enumerate() {
            if s.is_on {
                mask |= 1 << (n_d + j);
            }
        }
        mask
    }

    /// Sets the control input signal value for an IdealSwitch block.
    pub fn set_switch_control(&mut self, block_sid: &str, ctrl_value: f64) {
        if let Some(s) = self.switches.iter_mut().find(|s| s.sid == block_sid) {
            s.ctrl_val = ctrl_value;
        }
    }

    /// Returns the driver block index in `diagram.blocks` for an IdealSwitch block, if wired.
    pub fn get_switch_driver_idx(&self, block_sid: &str) -> Option<usize> {
        self.switches
            .iter()
            .find(|s| s.sid == block_sid)
            .and_then(|s| s.ctrl_driver_idx)
    }

    /// Returns the state of a switching element (1.0 for ON, 0.0 for OFF).
    pub fn element_state(&self, block_sid: &str) -> Option<f64> {
        if let Some(d) = self.diodes.iter().find(|d| d.sid == block_sid) {
            return Some(if d.is_on { 1.0 } else { 0.0 });
        }
        if let Some(s) = self.switches.iter().find(|s| s.sid == block_sid) {
            return Some(if s.is_on { 1.0 } else { 0.0 });
        }
        None
    }

    /// Solves the circuit operating point at t = t0 to establish initial node voltages, branch voltages,
    /// and branch currents consistent with component initial conditions (V0, I0) and source voltages.
    /// Performs ZERO heap allocations.
    pub fn init_solve(&mut self, t0: f64) -> Result<(), SimError> {
        let n = self.num_nodes;

        // 1. Update IdealSwitch states from control values
        for s in &mut self.switches {
            s.is_on = s.ctrl_val >= s.threshold;
        }

        let mut visited_count = 0usize;
        let n_diodes = self.diodes.len();

        // Fixed-point iteration loop to find self-consistent diode states at t0
        for _iter in 0..100 {
            let mask = self.current_mask();

            for &prev_mask in &self.visited_masks[..visited_count] {
                if prev_mask == mask {
                    return Err(SimError::chatter(format!(
                        "Switch limit cycle / chattering detected during DC operating point solve at t = {t0:.6} s; state mask {mask:#b} repeated in sequence."
                    )));
                }
            }

            if visited_count < self.visited_masks.len() {
                self.visited_masks[visited_count] = mask;
                visited_count += 1;
            }

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

            // Diode companion current source: when ON, I_eq = G_on * Vf from p to n
            for (i, d) in self.diodes.iter().enumerate() {
                let is_on = (mask & (1 << i)) != 0;
                if is_on {
                    let i_eq = d.gon * d.vf;
                    if d.p_net > 0 {
                        self.rhs[d.p_net - 1] += i_eq;
                    }
                    if d.n_net > 0 {
                        self.rhs[d.n_net - 1] -= i_eq;
                    }
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

            let lu = self
                .lu_init_table
                .get(mask)
                .and_then(|opt| opt.as_ref())
                .ok_or_else(|| {
                    SimError::singular(format!(
                        "Singular matrix encountered for switch state configuration {mask:#b} during DC operating point solve at t = {t0:.6} s."
                    ))
                })?;

            lu.solve(&self.rhs, &mut self.sol);

            self.node_voltages[0] = 0.0;
            for i in 1..=n {
                self.node_voltages[i] = self.sol[i - 1];
            }

            if n_diodes == 0 {
                break;
            }

            let mut converged = true;
            for (i, d) in self.diodes.iter_mut().enumerate() {
                let vp = self.node_voltages[d.p_net];
                let vn = self.node_voltages[d.n_net];
                let vd = vp - vn;
                let currently_on = (mask & (1 << i)) != 0;
                let id = if currently_on {
                    d.gon * (vd - d.vf)
                } else {
                    d.goff * vd
                };

                d.vd = vd;
                d.id = id;

                let next_on = if currently_on { id > 0.0 } else { vd >= d.vf };

                if next_on != currently_on {
                    converged = false;
                    d.is_on = next_on;
                }
            }

            if converged {
                break;
            }
        }

        if visited_count >= 100 {
            return Err(SimError::chatter(format!(
                "Exceeded maximum 100 fixed-point switch iterations during DC operating point solve at t = {t0:.6} s without convergence."
            )));
        }

        for (k, v) in self.vsrc_currents.iter_mut().enumerate() {
            *v = self.sol[n + k];
        }

        for d in &mut self.diodes {
            let vp = self.node_voltages[d.p_net];
            let vn = self.node_voltages[d.n_net];
            d.vd = vp - vn;
            d.id = if d.is_on {
                d.gon * (d.vd - d.vf)
            } else {
                d.goff * d.vd
            };
        }

        for s in &mut self.switches {
            let vp = self.node_voltages[s.p_net];
            let vn = self.node_voltages[s.n_net];
            s.vs = vp - vn;
            s.is = if s.is_on { s.gon * s.vs } else { s.goff * s.vs };
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

    /// Advances the linear and switching circuit by one fixed time step dt.
    /// Uses Backward Euler on step 1 to prevent cold-start trapezoidal ringing,
    /// then Trapezoidal rule for all subsequent steps.
    /// Performs within-step fixed-point iteration for piecewise-linear switches.
    /// Performs ZERO heap allocations.
    pub fn step(&mut self, t: f64, dt: f64, is_first_step: bool) -> Result<(), SimError> {
        let n = self.num_nodes;

        // 1. Update IdealSwitch states from their control values
        for s in &mut self.switches {
            s.is_on = s.ctrl_val >= s.threshold;
        }

        let mut visited_count = 0usize;
        let n_diodes = self.diodes.len();

        // 2. Fixed-point iteration loop
        for _iter in 0..100 {
            let mask = self.current_mask();

            // Check for limit cycle (chattering)
            for &prev_mask in &self.visited_masks[..visited_count] {
                if prev_mask == mask {
                    return Err(SimError::chatter(format!(
                        "Switch limit cycle / chattering detected at t = {t:.6} s; state mask {mask:#b} repeated in sequence."
                    )));
                }
            }

            if visited_count < self.visited_masks.len() {
                self.visited_masks[visited_count] = mask;
                visited_count += 1;
            }

            self.rhs.fill(0.0);

            // Independent DC current sources
            for isrc in &self.dc_isources {
                let i_val = isrc.current;
                if isrc.p_net > 0 {
                    self.rhs[isrc.p_net - 1] += i_val;
                }
                if isrc.n_net > 0 {
                    self.rhs[isrc.n_net - 1] -= i_val;
                }
            }

            // Companion model Norton equivalent history currents
            for c in &self.capacitors {
                let i_eq = if is_first_step {
                    -(c.capacitance / dt) * c.v0
                } else {
                    let g = 2.0 * c.capacitance / dt;
                    -g * c.vc - c.ic
                };

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

                if l.p_net > 0 {
                    self.rhs[l.p_net - 1] -= i_eq;
                }
                if l.n_net > 0 {
                    self.rhs[l.n_net - 1] += i_eq;
                }
            }

            // Diode companion current sources: when ON, I_eq = G_on * Vf from p to n
            for (i, d) in self.diodes.iter().enumerate() {
                let is_on = (mask & (1 << i)) != 0;
                if is_on {
                    let i_eq = d.gon * d.vf;
                    if d.p_net > 0 {
                        self.rhs[d.p_net - 1] += i_eq;
                    }
                    if d.n_net > 0 {
                        self.rhs[d.n_net - 1] -= i_eq;
                    }
                }
            }

            // Independent voltage sources
            for vsrc in &self.dc_vsources {
                self.rhs[n + vsrc.vsrc_idx] = vsrc.voltage;
            }

            for vsrc in &self.ac_vsources {
                let v_val = vsrc.amplitude
                    * (2.0 * std::f64::consts::PI * vsrc.frequency * t + vsrc.phase_rad).sin()
                    + vsrc.offset;
                self.rhs[n + vsrc.vsrc_idx] = v_val;
            }

            // Solve Ax = b via pre-factorized LU table
            let lu = if is_first_step {
                self.lu_be_table.get(mask).and_then(|opt| opt.as_ref())
            } else {
                self.lu_trap_table.get(mask).and_then(|opt| opt.as_ref())
            };

            match lu {
                Some(decomp) => decomp.solve(&self.rhs, &mut self.sol),
                None => {
                    return Err(SimError::singular(format!(
                        "Singular matrix encountered for switch state configuration {mask:#b} at t = {t:.6} s."
                    )));
                }
            }

            self.node_voltages[0] = 0.0;
            for i in 1..=n {
                self.node_voltages[i] = self.sol[i - 1];
            }

            if n_diodes == 0 {
                break;
            }

            // Evaluate next diode states
            let mut converged = true;
            for (i, d) in self.diodes.iter_mut().enumerate() {
                let vp = self.node_voltages[d.p_net];
                let vn = self.node_voltages[d.n_net];
                let vd = vp - vn;
                let currently_on = (mask & (1 << i)) != 0;
                let id = if currently_on {
                    d.gon * (vd - d.vf)
                } else {
                    d.goff * vd
                };

                d.vd = vd;
                d.id = id;

                let next_on = if currently_on { id > 0.0 } else { vd >= d.vf };

                if next_on != currently_on {
                    converged = false;
                    d.is_on = next_on;
                }
            }

            if converged {
                break;
            }
        }

        if visited_count >= 100 {
            return Err(SimError::chatter(format!(
                "Exceeded maximum 100 fixed-point switch iterations at t = {t:.6} s without convergence."
            )));
        }

        // Converged! Update source currents, element states, and reactive history
        for (k, v) in self.vsrc_currents.iter_mut().enumerate() {
            *v = self.sol[n + k];
        }

        for d in &mut self.diodes {
            let vp = self.node_voltages[d.p_net];
            let vn = self.node_voltages[d.n_net];
            d.vd = vp - vn;
            d.id = if d.is_on {
                d.gon * (d.vd - d.vf)
            } else {
                d.goff * d.vd
            };
        }

        for s in &mut self.switches {
            let vp = self.node_voltages[s.p_net];
            let vn = self.node_voltages[s.n_net];
            s.vs = vp - vn;
            s.is = if s.is_on { s.gon * s.vs } else { s.goff * s.vs };
        }

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

        // Diode
        if let Some(d) = self.diodes.iter().find(|d| d.sid == block_sid) {
            return Some(d.id);
        }

        // Ideal Switch
        if let Some(s) = self.switches.iter().find(|s| s.sid == block_sid) {
            return Some(s.is);
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
    if p > 0 {
        a[(p - 1) * size + col] += 1.0;
        a[col * size + (p - 1)] += 1.0;
    }
    if n > 0 {
        a[(n - 1) * size + col] -= 1.0;
        a[col * size + (n - 1)] -= 1.0;
    }
}
