# Milestone M7 Design Note: Sensors, Controlled Sources & Hybrid Coupling

## 1. What and Why
Milestone M7 bridges the signal-flow domain and the electrical network domain within the MatRust Studio simulation engine (`crates/mlapp-sim`).

Prior to M7, electrical circuits operated with independent sources (DC/AC voltage and current sources) and passive/semiconductor components (R, L, C, Diode, Ideal Switch), with signals only entering electrical circuits as switch control gates. M7 introduces bidirectional coupling:
1. **Electrical-to-Signal (Sensors)**: `VoltageSensor` and `CurrentSensor` observe circuit states and produce standard scalar signals for control and monitoring blocks.
2. **Signal-to-Electrical (Controlled Sources)**: `VoltageControlledVoltageSource` (VCVS) and `VoltageControlledCurrentSource` (VCCS) convert control signals into dynamic voltage and current excitations.

This enables closed-loop control of physical circuits, operational amplifier macro-models, and hybrid cyber-physical simulations.

---

## 2. Component Specifications & MNA Mathematical Formulation

### 2.1 Voltage Sensor (`VoltageSensor`)
- **Ports**:
  - Electrical Terminal 0: Positive terminal $p$ (label `+`).
  - Electrical Terminal 1: Negative terminal $n$ (label `-`).
  - Signal Output Port 0: Measured voltage $v$ (label `V`).
- **Physical Model**:
  - Ideal voltmeter with infinite input impedance: $G_{in} = 0$, $I_{in} = 0$.
  - Draws zero current from the monitored branch; does not perturb node potentials.
- **MNA Stamping**:
  - No entries in conductance matrix $G$ or source vector $J$.
  - Measurement at step $k$:
    $$v(t_k) = V_p(t_k) - V_n(t_k)$$
    where $V_0 = 0$ if connected to reference Ground (node 0).

### 2.2 Current Sensor (`CurrentSensor`)
- **Ports**:
  - Electrical Terminal 0: Anode/Inlet terminal $p$ (label `in`).
  - Electrical Terminal 1: Cathode/Outlet terminal $n$ (label `out`).
  - Signal Output Port 0: Measured current $i$ (label `I`).
- **Physical Model**:
  - Ideal ammeter with zero internal resistance: $R_{internal} = 0$, $V_p - V_n = 0$.
- **MNA Stamping**:
  - Stamped exactly as an independent 0 V voltage source ($E = 0$).
  - Introduces one auxiliary branch current variable $i_m$ in the MNA unknowns vector $x$:
    $$\begin{bmatrix} G & B \\ C & D \end{bmatrix} \begin{bmatrix} v \\ i \end{bmatrix} = \begin{bmatrix} J \\ E \end{bmatrix}$$
  - Terminal $p$ stamps $+1$ in row $m$, column $p$; terminal $n$ stamps $-1$ in row $m$, column $n$.
  - Column $m$ stamps $+1$ in node row $p$, and $-1$ in node row $n$.
  - RHS entry $E_m = 0.0$.
- **Measurement at step $k$**:
  $$i(t_k) = x_m(t_k)$$

### 2.3 Voltage-Controlled Voltage Source (`VoltageControlledVoltageSource` / `ControlledVoltageSource`)
- **Ports**:
  - Electrical Terminal 0: Positive terminal $p$ (label `+`).
  - Electrical Terminal 1: Negative terminal $n$ (label `-`).
  - Signal Input Port 0: Control input $u_{ctrl}$ (label `in`).
- **Parameters**:
  - `Gain`: $K \in \mathbb{R}$ (dimensionless or V/V, finite, default `1.0`).
- **MNA Stamping**:
  - Stamped as a branch voltage source with branch index $e$.
  - Equation: $V_p - V_n = K \cdot u_{ctrl}(t_k)$.
  - Conductance submatrices $B$ and $C$ stamped with $+1 / -1$ at terminals $p$ and $n$.
  - RHS vector updated dynamically at start of step $k$:
    $$E_e(t_k) = K \cdot u_{ctrl}(t_k)$$

### 2.4 Voltage-Controlled Current Source (`VoltageControlledCurrentSource` / `ControlledCurrentSource`)
- **Ports**:
  - Electrical Terminal 0: Positive terminal $p$ (label `+`).
  - Electrical Terminal 1: Negative terminal $n$ (label `-`).
  - Signal Input Port 0: Control input $u_{ctrl}$ (label `in`).
- **Parameters**:
  - `Transconductance` (or `Gain`): $G_m \in \mathbb{R}$ (A/V or S, finite, default `1.0`).
- **MNA Stamping**:
  - Does NOT add an auxiliary row to the MNA matrix.
  - Injected current leaves terminal $n$ and enters terminal $p$:
    $$I_{out}(t_k) = G_m \cdot u_{ctrl}(t_k)$$
  - Directly stamped into the RHS current injection vector $J$ at start of step $k$:
    $$J_p \mathrel{+}= I_{out}(t_k), \quad J_n \mathrel{-}= I_{out}(t_k)$$

---

## 3. Multi-Domain Coupling & Sampling Rules (Decision D4)

1. **One-Step Hold (Zero-Order Hold)**:
   - At step $k$ ($t = t_k$), the control input $u_{ctrl}(t_k)$ to all controlled sources and switches is evaluated and sampled at the **start** of the step.
   - It remains constant during the MNA linear/nonlinear solve for that step.
2. **Sensor Signal Propagation**:
   - The MNA solution $x(t_k)$ is solved.
   - Sensor output signals $v(t_k)$ and $i(t_k)$ are written into the signal registry.
   - Downstream signal blocks (scopes, controllers, math blocks) receive the updated sensor signals.
3. **Algebraic Loop Detection & `W-DELAY`**:
   - If a diagram contains a direct signal connection from a sensor output to a controlled source (or switch) that affects the same circuit, and this path does not pass through a state-holding block (`Integrator`, `UnitDelay`, `ZeroOrderHold`, or strictly proper `TransferFunction`), an instantaneous cross-domain loop exists.
   - In accordance with Charter Decision D4, the engine detects this path during topological analysis, emits diagnostic warning `W-DELAY`, and inserts a 1-step unit delay ($z^{-1}$) so the simulation proceeds deterministically without algebraic non-convergence.

---

## 4. Scope IN / OUT

- **IN**:
  - 4 new block types: `VoltageSensor`, `CurrentSensor`, `VoltageControlledVoltageSource`, `ControlledCurrentSource` (with aliases).
  - Spec table entries in `spec.rs` (ports, parameters, feedthrough, pin coordinates).
  - MNA stamps and RHS dynamic evaluation in `electrical.rs`.
  - Signal-electrical coupling and ZOH sampling in `solver.rs`.
  - Direct algebraic loop detection emitting warning `W-DELAY`.
  - Desktop editor integration: palette, icons, pin geometry.
  - Full test suite in `m7_sensors_sources_tests.rs` with analytical oracles (ideal op-amp inverting amplifier, current-controlled filter, ammeter insertion parity).
  - Zero heap allocation verification across 10,000 steps in `alloc_test.rs`.
- **OUT**:
  - Nonlinear flux/charge dependent controlled sources (deferred to future expansion).
  - Thermal domain ports (out of project scope).

---

## 5. File Layout

1. `crates/mlapp-sim/src/model.rs`:
   - Add `BlockType::VoltageSensor`, `BlockType::CurrentSensor`, `BlockType::VoltageControlledVoltageSource`, `BlockType::VoltageControlledCurrentSource`.
2. `crates/mlapp-sim/src/spec.rs`:
   - Spec table entries, ports, parameters, pin offsets.
3. `crates/mlapp-sim/src/electrical.rs`:
   - Sensor tracking structs and controlled source structs.
   - In-place RHS vector updating with zero allocations.
4. `crates/mlapp-sim/src/solver.rs`:
   - Step execution loop routing sensor readings to signal registry and feeding control inputs to controlled sources.
   - Detection of direct sensor-to-source loops emitting `W-DELAY`.
5. `crates/mlapp-sim/src/diagnostics.rs` & `docs/spec/error-catalog.md`:
   - Catalog entry and diagnostic definition for `W-DELAY`.
6. `apps/desktop/src/block_diagram_editor.rs`:
   - Electrical library palette entries and pin rendering.
7. `crates/mlapp-sim/tests/m7_sensors_sources_tests.rs`:
   - Integration tests, known-answer oracles, boundary cases, `W-DELAY` tests.

---

## 6. Risks & Mitigations

| Risk | Mitigation |
| :--- | :--- |
| Current sensor auxiliary rows increase MNA matrix size | Pre-factored LU decomposition handles increased $N$; memory stays well below 64 MB for realistic circuits. |
| Memory allocations during step when reading sensors or updating RHS | Pre-allocated slices and vectors; zero heap allocations verified in `alloc_test.rs`. |
| Closed-loop feedback instability due to direct feedthrough | Enforce ZOH 1-step hold and emit `W-DELAY` diagnostic when a direct algebraic loop is detected. |
