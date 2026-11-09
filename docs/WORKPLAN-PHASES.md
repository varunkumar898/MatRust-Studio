# MatRust Studio Engineering Phases (docs/WORKPLAN-PHASES.md)

This document defines the detailed execution roadmap for MatRust Studio development across Phases **F1 through F14**, providing itemized accounting, dependency graphs, sizing, verification oracles, release trains, and governance rules.

---

## 1. Rules for Phase Items (P13)

1. **One item = one task = one PR**: Every discrete item maps to a single focused task and Pull Request. Items sized **XL** (DAE solver, Radau IIA, FMU export/import, HDL generation, GPU acceleration, Model Predictive Control, HIL, AUTOSAR, PLC code generation, ROS bridge, Gazebo integration, state machine editor) must be decomposed by a formal planning task (`PLAN-F<n>`) into subtasks of size **L** or smaller, accompanied by an Architecture Decision Record (ADR), prior to code implementation.
2. **Standardized status values**:
   - `TODO`: Planned item not yet started or implemented.
   - `EXISTS-VERIFIED`: Implemented in the codebase with named automated test verification cited.
   - `EXISTS-UNVERIFIED`: Implemented in the codebase or UI, pending formal automated test or human visual verification.
   - `DEFERRED`: Explicitly deferred by owner decision (tracked in `docs/ROADMAP.md`).
   - `NEEDS-HUMAN-HARDWARE`: Target requiring physical hardware (microcontroller board, robot, external hardware-in-the-loop simulator). The software interface, code generator, and simulated host loopback mock must be built and tested; on-target claims are disallowed without physical verification.
3. **Dependency and storage discipline**: Any new external crate dependency requires an ADR and owner approval; prefer self-contained in-tree numerical algorithms. Heavy optional capabilities (FMU co-simulation, Python C-API bindings, GPU compute shaders, ROS client libraries) must reside behind optional cargo feature flags or standalone modules, never bloating the default base installer. Every phase must audit release executable size and package footprint against [docs/STORAGE-BUDGET.md](file:///<repo>/docs/STORAGE-BUDGET.md).
4. **Clean-room naming standards**: Use generic, descriptive terminology in all user interface labels, source code identifiers, documentation, and file names (e.g., *state machine editor*, *model diagnostics & checks*, *custom block extension API*). Never copy MathWorks trademarks, proprietary brand names, or copyrighted interface descriptors.
5. **Traceability and auditability**: Every merged item requires:
   - A named automated regression test in the test suite.
   - A dedicated generated documentation page in `docs/`.
   - An active entry in [docs/FEATURE-REGISTRY.md](file:///<repo>/docs/FEATURE-REGISTRY.md) (with GUI rendering status marked `UNVERIFIED` until human visual inspection).
   - An updated capability mapping line in [docs/CLAIMS.md](file:///<repo>/docs/CLAIMS.md) for any public-facing capability.
6. **Phase merge gate**: A phase is declared **MERGED** only when all constituent items are `MERGED`, `EXISTS-VERIFIED`, or `DEFERRED`, all stated entry and exit criteria pass, storage and performance budgets are satisfied, and its designated release-train reference example is integrated into the application's Examples catalog.

---

## 2. Release Trains & Acceptance Goal Gates

### 2.1 Release Train Schedule

| Release Train | Target Version | Included Scope | Milestone Goal Gates | Reference Milestone Example |
| :--- | :---: | :--- | :--- | :--- |
| **R1** | `v0.3` | Waves 0–2 + **F1** (Block Editor) + **F2** (Block Library) | **G1**, **G4** | Closed-loop DC motor velocity control diagram |
| **R2** | `v0.4` | **F3** (Hierarchy) + **F4** (Advanced Solvers) | **G3**, **G5** | Hierarchical switched buck converter with stiff bisection |
| **R3** | `v0.5` | **F5** (Multidomain basics) + **F7** (Data Inspector) + **F8** (Control) | **G2**, **G6**, **G7** | Thermal-electromechanical actuator with Bode/margin analysis |
| **R4** | `v0.6` | **F6** (State Machines) + **F9** (Optimization) + **F10** (Verification) | **G8** | Automated test suite with parameter sweep & statechart supervisory logic |
| **R5** | `v0.7` | **F11** (Embedded Codegen) + **F12** (Integration) + **F13** (Performance) | **G9**, **G10** | High-throughput multi-rate model with ISO C99 target generation & SIL |

### 2.2 Acceptance Goal Gates (G1..G10)

- **G1 (Motor Control from Blocks)**: Complete electromechanical motor-control model constructed purely from block diagram library components and simulated with continuous feedback.
- **G2 (Multidomain Co-Simulation)**: Unified simulation coupling electrical circuit, mechanical inertia/shaft, and thermal dissipation domains in a single time-domain run.
- **G3 (Nested Subsystems & Reusable Models)**: Multi-level hierarchical subsystem nesting with atomic boundary execution and reusable library block instantiation.
- **G4 (Continuous, Discrete & Hybrid)**: Deterministic synchronization of continuous ODE states and multi-rate discrete sample times with zero numerical drift.
- **G5 (Adaptive & Stiff Solvers)**: Adaptive error-controlled solvers and stiff/DAE numerical engines solving classical benchmark problems (Robertson, Van der Pol $\mu=1000$).
- **G6 (Stability & Frequency Analysis)**: Automated linear state-space extraction, numerical Jacobian linearization, Bode magnitude/phase frequency response, and stability margin computation.
- **G7 (Recording & Comparing Runs)**: High-capacity Data Inspector recording multi-run time histories with difference plotting, cursor measurements, and CSV/MAT export.
- **G8 (Automated Tests & Reports)**: Headless CLI test execution verifying block unit behavior, assertion monitoring, and automated markdown/HTML test reports.
- **G9 (Embedded C on Target)**: Standalone ANSI/ISO C99 code generation verified for zero dynamic memory allocation and identical numerical trajectory against interpreter (`NEEDS-HUMAN-HARDWARE`).
- **G10 (Documented Interoperability)**: Standardized FMI/FMU co-simulation container exchange and data bridge interoperability with verified golden test suites.

---

## 3. Phase Item Catalog (F1 through F14)

### Phase F1: Professional Block Editor (14 Items)

- **Goal**: Transform the block diagram editor into a high-productivity graphical modeling environment with fluid wire routing, visual inspection, alignment ergonomics, and keyboard navigation.
- **Entry Criteria**: Baseline block diagram canvas (M12) with multi-level undo/redo, cancellation, and basic port connectivity.
- **Exit Criteria**: Every editor action functions seamlessly via mouse and keyboard; geometric logic (snapping, routing, alignment, selection) is verified by unit tests; UI checklist is human-verified.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F1.01** | Snap-to-grid | `TODO` | - | M | - | Configurable magnetic grid pitch (8px/16px) with visual guide points. |
| **F1.02** | Automatic wire routing | `TODO` | F1.01 | L | - | Orthogonal Manhattan routing with obstacle avoidance around blocks. |
| **F1.03** | Multi-block selection | `TODO` | - | M | - | Box selection marquee and Shift/Ctrl click multi-item selection set. |
| **F1.04** | Copy, paste and duplicate blocks | `EXISTS-UNVERIFIED` | F1.03 | M | `test_block_diagram_duplicate` | Diagram block duplication verified; system clipboard copy/paste pending. |
| **F1.05** | Block alignment and distribution | `TODO` | F1.03 | S | - | Align edges/centers and distribute horizontal/vertical spacing evenly. |
| **F1.06** | Zoom-to-fit and minimap | `TODO` | - | M | - | Viewport auto-fit bounding box calculation and draggable corner minimap. |
| **F1.07** | Wire labels and annotations | `TODO` | F1.02 | S | - | Text annotations anchored to signal nets and freeform diagram text boxes. |
| **F1.08** | Signal-path highlighting | `TODO` | F1.03 | M | - | Dynamic trace highlighting upstream sources and downstream sinks. |
| **F1.09** | Block parameter validation | `EXISTS-VERIFIED` | - | S | `test_invalid_numeric_param_rejected_old_value_kept` | Property panel rejects non-numeric/out-of-bound inputs while preserving valid state. |
| **F1.10** | Unit and data-type visualization | `TODO` | F2.00 | M | - | Colored port badges displaying signal dimensions, types, and physical units. |
| **F1.11** | Searchable block library | `TODO` | - | M | - | Real-time fuzzy text search filter over categorized component palette. |
| **F1.12** | Keyboard shortcuts | `EXISTS-UNVERIFIED` | - | S | `test_m4_block_rotation_and_simulation_e_unsupported_in_editor` | 'R' key rotation exists; complete accelerator table pending interactive check. |
| **F1.13** | Model hierarchy navigation | `TODO` | F3.01 | L | - | Breadcrumb path bar and double-click drill-down into nested subsystems. |
| **F1.14** | Diagram auto-layout | `TODO` | F1.02, F1.05 | L | - | Sugiyama-style hierarchical layered DAG layout engine for clean routing. |

---

### Phase F2: Complete Block Library (23 Items + F2.00)

- **Prerequisite Task F2.00**: Vector signal width, tensor dimensions, and primitive data types in the block spec table and port model. Requires formal ADR and owner approval.
- **Goal**: Full mathematical, routing, continuous, discrete, and non-linear block library with strict feedthrough definitions, sample-time rules, and C codegen support.
- **Entry Criteria**: Spec table architecture (M3) and electrical port model (M4).
- **Exit Criteria**: Every block has a spec-table definition, generated docs page, feedthrough/sample-time contracts, oracle test, palette entry, and codegen flag.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F2.00** | Vector width & type model (Prereq) | `TODO` | - | L | - | Architecture Decision Record: typed tensor signal buses in port definitions. |
| **F2.01** | Ramp and random signal generators | `TODO` | F2.00 | M | - | Continuous slope ramp and seeded pseudo-random uniform/normal generators. |
| **F2.02** | Chirp and band-limited noise | `TODO` | F2.01 | M | - | Swept-frequency sinusoidal chirp and band-limited Gaussian white noise. |
| **F2.03** | Mux and Demux | `TODO` | F2.00 | M | - | Signal vector multiplexer (scalars -> vector) and vector split demultiplexer. |
| **F2.04** | Bus Creator and Bus Selector | `TODO` | F2.00, F2.03 | L | - | Named composite signal bus assembly and field extraction blocks. |
| **F2.05** | Vector concatenate | `TODO` | F2.00 | S | - | Multi-signal concatenation across specified tensor dimension. |
| **F2.06** | Selector and Assignment | `TODO` | F2.00 | M | - | Element indexing, vector slice selection, and sub-vector tensor assignment. |
| **F2.07** | Goto and From | `TODO` | - | M | - | Wireless signal connectivity tags with local, scoped, and global visibility. |
| **F2.08** | Data Store Memory/Read/Write | `TODO` | - | L | - | Global named state storage registers with read-before-write validation. |
| **F2.09** | Rate Transition | `TODO` | F2.00 | M | - | Deterministic buffer handling asynchronous sample rate transitions. |
| **F2.10** | Derivative | `TODO` | - | S | - | Numerical continuous differentiator with high-frequency low-pass filter pole. |
| **F2.11** | Advanced Integrator | `TODO` | - | M | - | Integrator with external reset port, state saturation limits, and IC input. |
| **F2.12** | State-Space | `EXISTS-VERIFIED` | - | M | `test_m3_transfer_function_analytical_step` | Continuous and discrete state-space formulation $\dot{x} = Ax + Bu, y = Cx + Du$. |
| **F2.13** | Zero-Pole-Gain | `TODO` | F2.12 | M | - | Factored transfer function block parameterized by zeros, poles, and scalar gain. |
| **F2.14** | Discrete filters | `TODO` | - | M | - | Direct-form I/II discrete IIR and FIR digital filter difference equations. |
| **F2.15** | Transport Delay | `TODO` | - | M | - | Time delay buffer $u(t - \tau)$ with cubic Hermite continuous interpolation. |
| **F2.16** | Memory | `EXISTS-VERIFIED` | - | S | `test_unit_delay_sample_time_under_dp` | Discrete 1-step unit delay $z^{-1}$ breaking continuous algebraic loops. |
| **F2.17** | Lookup Table 1D and 2D | `TODO` | - | L | - | Breakpoint table interpolation (linear/cubic spline) with extrapolation bounds. |
| **F2.18** | Dead Zone and Backlash | `TODO` | - | S | - | Non-linear symmetric/asymmetric deadband and mechanical backlash gap. |
| **F2.19** | Relay and hysteresis | `TODO` | - | S | - | Bi-stable switching relay with on/off hysteresis thresholds. |
| **F2.20** | Quantizer | `TODO` | - | S | - | Continuous signal discretization to discrete quantization interval steps. |
| **F2.21** | Multiport Switch | `EXISTS-VERIFIED` | - | M | `test_m3_control_eval_and_feedthrough` | Multi-input conditional data routing controlled by integer index or threshold. |
| **F2.22** | Function-Call Generator | `TODO` | - | M | - | Programmable periodic or event-driven execution trigger strobe. |
| **F2.23** | Enabled and Triggered subsystems | `TODO` | - | M | - | Boundary control ports enabling conditional subsystem execution blocks. |

---

### Phase F3: Hierarchical Modeling (16 Items)

- **Goal**: Hierarchical composition, reusable sub-models, conditional subsystem execution, masked component abstractions, and centralized data dictionaries.
- **Entry Criteria**: Block editor (F1) and core block library (F2).
- **Exit Criteria**: Hierarchical model with nested subsystems, masked blocks, and variant selections executes bit-identical to flattened equivalent; schema version bumped with automated migration.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F3.01** | Nested Subsystems | `TODO` | F1.13 | L | - | Arbitrary-depth child diagrams encapsulated within a parent subsystem node. |
| **F3.02** | Subsystem input/output interfaces | `TODO` | F3.01 | M | - | Formal boundary ports (`Inport`, `Outport`) mapping external signals to child nets. |
| **F3.03** | Virtual and atomic subsystems | `TODO` | F3.01 | L | - | Graph-flattened virtual execution vs isolated atomic topological ordering. |
| **F3.04** | Enabled Subsystems | `TODO` | F2.23, F3.03 | L | - | Conditional execution active only while enable signal exceeds zero. |
| **F3.05** | Triggered Subsystems | `TODO` | F2.23, F3.03 | L | - | Instantaneous one-step execution triggered on rising/falling signal edges. |
| **F3.06** | Function-Call Subsystems | `TODO` | F2.22, F3.03 | L | - | Software event-driven subsystem invoked directly by function-call strobe. |
| **F3.07** | Model Reference blocks | `TODO` | F3.01 | XL | - | Encapsulated execution of external standalone `.json` model files. |
| **F3.08** | Reusable model components | `TODO` | F3.07 | M | - | Multi-instance shared diagram components without duplicating source logic. |
| **F3.09** | Library linking | `TODO` | F3.08 | L | - | Referenced library blocks with automated propagation of upstream changes. |
| **F3.10** | Custom masked blocks | `TODO` | F3.01 | L | - | Parameter encapsulation dialogs, user prompts, and custom canvas icon rendering. |
| **F3.11** | Model workspace | `TODO` | - | M | - | Model-scoped parameter scope preventing global variable pollution. |
| **F3.12** | Shared Data Dictionary | `TODO` | F3.11 | L | - | Centralized project data repository for parameters, constants, and types. |
| **F3.13** | Bus objects | `TODO` | F2.04, F3.12 | M | - | Strongly-typed structural definitions validating bus contents at compile-time. |
| **F3.14** | Variant Subsystems | `TODO` | F3.03 | L | - | Mutually-exclusive diagram implementations switched by configuration flags. |
| **F3.15** | Variant parameters | `TODO` | F3.14 | M | - | Parameter values dynamically evaluated according to active variant condition. |
| **F3.16** | Model configuration profiles | `TODO` | - | M | - | Named configurations storing solver parameters, tolerances, and diagnostics. |

---

### Phase F4: Advanced Simulation Engine (18 Items)

- **Goal**: High-order stiff solvers (Radau, BDF), differential-algebraic equation (DAE) solver, microsecond zero-crossing bisection, and hybrid multi-rate scheduling.
- **Entry Criteria**: Adaptive continuous solvers (M2) and MNA circuit stepping (M5/M6).
- **Exit Criteria**: Stiff benchmarks (Robertson, Van der Pol $\mu=1000$) match analytical tolerances; bouncing ball bisection achieves $10^{-9}$ s event accuracy; operating point save/restore matches continuous run.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F4.01** | Adaptive solver error control | `EXISTS-VERIFIED` | - | M | `test_dormand_prince_tolerance_compliance` | Dormand-Prince DP45 variable-step error control with local truncation bounds. |
| **F4.02** | BDF stiff solver | `TODO` | F4.01 | L | - | Variable-order (orders 1..5) Backward Differentiation Formula solver. |
| **F4.03** | Radau IIA stiff solver | `TODO` | F4.01 | XL | - | 3-stage 5th-order fully implicit Runge-Kutta stiff and DAE integrator. |
| **F4.04** | DAE solver | `TODO` | F4.03 | XL | - | Semi-explicit index-1 and index-2 differential algebraic equation solver. |
| **F4.05** | Consistent initial conditions | `TODO` | F4.04 | M | - | Initial condition solver reconciling algebraic constraints at $t=0$. |
| **F4.06** | Zero-crossing detection | `TODO` | - | L | - | Continuous sign monitoring detecting non-smooth state and signal boundaries. |
| **F4.07** | Accurate event localization | `TODO` | F4.06 | L | - | Bisection root-finding to $10^{-9}$ s and step 1 Backward Euler restart (M6b). |
| **F4.08** | Switching event scheduling | `TODO` | F4.07 | M | - | Time-ordered priority queue sequencing discrete hits and zero crossings. |
| **F4.09** | Event logging and timestamps | `TODO` | F4.08 | S | - | Diagnostic trace recording exact event timestamps and triggering block IDs. |
| **F4.10** | State reset & solver reinitialization | `TODO` | F4.07 | M | - | Hard state discontinuity injection and adaptive step size re-initialization. |
| **F4.11** | Multi-rate scheduling | `TODO` | - | L | - | Fundamental GCD multi-tasking scheduler executing distinct sample periods. |
| **F4.12** | Asynchronous sample times | `TODO` | F4.11 | M | - | Aperiodic and interrupt-driven execution timestamps outside periodic grid. |
| **F4.13** | Rate Transition semantics | `TODO` | F2.09, F4.11 | M | - | Deterministic data integrity buffers between asynchronous clock domains. |
| **F4.14** | Hybrid continuous/discrete execution | `EXISTS-VERIFIED` | - | M | `test_unit_delay_sample_time_under_dp` | Synchronized integration of continuous ODE states with discrete sample hits. |
| **F4.15** | Algebraic loop solver | `EXISTS-UNVERIFIED` | - | L | `test_algebraic_loop_self_loop_gives_e_alg` | Loop detection (`E-ALG`) exists; Newton-Raphson iteration solver is planned. |
| **F4.16** | Save & restore operating points | `TODO` | - | M | - | Serialization of entire continuous/discrete simulator state for warm restarts. |
| **F4.17** | Pause, resume and restart | `EXISTS-UNVERIFIED` | - | S | `test_simulation_gain_parameter_change` | Thread-safe cancel flag exists; interactive pause/resume loop pending verification. |
| **F4.18** | Solver profiling and diagnostics | `TODO` | - | M | - | Detailed accounting of step attempts, rejections, Jacobian updates, and wall time. |

---

### Phase F5: Multidomain Physical Simulation (46 Items)

- **Goal**: Physics-based physical network simulation across mechanical, thermal, fluid, and power electronic domains with conservative energy balance and unit-consistent physical ports.
- **Entry Criteria**: F4 stiff solver and event localization.
- **Exit Criteria**: Every component features unit-consistent terminals and an analytical oracle test; energy conservation verified on lossless multi-domain couplings.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F5.01** | Mass | `TODO` | F4.01 | S | - | Translational mass element obeying Newton's second law $F = m \dot{v}$. |
| **F5.02** | Spring | `TODO` | F5.01 | S | - | Linear mechanical spring with restoring force $F = k (x_1 - x_2)$. |
| **F5.03** | Damper | `TODO` | F5.01 | S | - | Viscous translational damper with dissipative force $F = c (v_1 - v_2)$. |
| **F5.04** | Translational friction | `TODO` | F5.01 | M | - | Non-linear Coulomb, Stribeck, and static breakaway friction model. |
| **F5.05** | Inertia | `TODO` | F4.01 | S | - | Rotational moment of inertia obeying torque balance $\tau = J \dot{\omega}$. |
| **F5.06** | Rotational shaft | `TODO` | F5.05 | S | - | Torsional spring-damper compliance between rotating mechanical nodes. |
| **F5.07** | Gearbox | `TODO` | F5.05 | S | - | Kinematic gear constraint with ratio $N$ coupling torque and angular speed. |
| **F5.08** | Clutch | `TODO` | F5.05, F4.07 | M | - | Friction plate clutch with locked, slipping, and disengaged operating modes. |
| **F5.09** | Brake | `TODO` | F5.05, F4.07 | M | - | Mechanical dissipative brake with controllable friction clamping torque. |
| **F5.10** | Torque source | `TODO` | F5.05 | S | - | Ideal controllable torque actuator injecting angular momentum. |
| **F5.11** | Motion sensor | `TODO` | F5.01, F5.05 | S | - | Kinematic position, linear velocity, angle, and angular velocity sensor. |
| **F5.12** | Thermal mass | `TODO` | F4.01 | S | - | Lumped thermal energy storage element $Q = m c_p \dot{T}$. |
| **F5.13** | Thermal resistance | `TODO` | F5.12 | S | - | Conductive heat transfer element obeying Fourier's law $q = \Delta T / R_{th}$. |
| **F5.14** | Thermal capacitance | `TODO` | F5.12 | S | - | Generalized thermal storage capacitance buffering temperature transients. |
| **F5.15** | Heat source | `TODO` | F5.12 | S | - | Controlled heat flow or heat dissipation power injection element. |
| **F5.16** | Convection | `TODO` | F5.12 | S | - | Fluid surface convection obeying Newton's cooling law $q = h A \Delta T$. |
| **F5.17** | Radiation | `TODO` | F5.12 | M | - | Stefan-Boltzmann radiative heat transfer $q = \epsilon \sigma A (T_1^4 - T_2^4)$. |
| **F5.18** | Heat exchanger | `TODO` | F5.13, F5.16 | M | - | Dual-fluid counter-flow or cross-flow thermal exchange macro-model. |
| **F5.19** | Temperature sensor | `TODO` | F5.12 | S | - | Across-variable thermal sensor measuring absolute or differential temperature. |
| **F5.20** | Pressure source | `TODO` | F4.01 | S | - | Ideal hydraulic/pneumatic pressure source maintaining specified head. |
| **F5.21** | Flow source | `TODO` | F4.01 | S | - | Controlled volumetric flow rate fluid injection element. |
| **F5.22** | Pipe | `TODO` | F5.20 | M | - | Fluid conduit modeling laminar/turbulent wall friction and fluid inertia. |
| **F5.23** | Valve | `TODO` | F5.20, F4.07 | M | - | Orifice valve with variable discharge coefficient and flow restriction. |
| **F5.24** | Pump | `TODO` | F5.20, F5.05 | M | - | Positive displacement or centrifugal pump converting torque to fluid flow. |
| **F5.25** | Reservoir | `TODO` | F5.20 | S | - | Atmospheric or pressurized hydraulic fluid storage reservoir. |
| **F5.26** | Hydraulic cylinder | `TODO` | F5.20, F5.01 | M | - | Hydro-mechanical piston transducer converting fluid pressure to force. |
| **F5.27** | Pneumatic chamber | `TODO` | F5.20, F5.12 | M | - | Compressible gas chamber modeling pressure, volume, and temperature laws. |
| **F5.28** | Pressure and flow sensors | `TODO` | F5.20 | S | - | Through and across transducers measuring line pressure and volumetric flow. |
| **F5.29** | MOSFET | `TODO` | M6, F4.07 | M | - | Power MOSFET semiconductor model with gate threshold and on-resistance. |
| **F5.30** | IGBT | `TODO` | M6, F4.07 | M | - | Insulated-gate bipolar transistor model with forward drop and tail current. |
| **F5.31** | BJT | `TODO` | M6, F4.07 | M | - | Bipolar junction transistor modeling linear, saturated, and cutoff states. |
| **F5.32** | Transformer | `TODO` | M5 | M | - | Multi-winding magnetic transformer with leakage inductance and core loss. |
| **F5.33** | Three-phase source | `TODO` | M5 | M | - | Balanced and unbalanced three-phase sinusoidal AC voltage source. |
| **F5.34** | Three-phase load | `TODO` | M5 | M | - | Wye- and delta-connected balanced/unbalanced resistive-inductive AC loads. |
| **F5.35** | PWM generator | `TODO` | M3 | S | - | Carrier-based sinusoidal and space-vector pulse-width modulation generator. |
| **F5.36** | Gate driver | `TODO` | F5.29, F5.35 | S | - | Buffer amplifier translating logic signals to gate charge drive voltages. |
| **F5.37** | Battery model | `TODO` | M5 | M | - | Equivalent-circuit electrochemical battery with state-of-charge tracking. |
| **F5.38** | Battery management system | `TODO` | F5.37 | L | - | Cell balancing, thermal monitoring, and over-current protection logic. |
| **F5.39** | Motor drive | `TODO` | F5.29, F5.35 | L | - | Inverter bridge power stage driving PMSM and induction motor windings. |
| **F5.40** | Thermal losses & temperature | `TODO` | F5.12, F5.29 | M | - | Conduction and switching loss calculation coupled to thermal heat sink nodes. |
| **F5.41** | Electrical-mechanical coupling | `EXISTS-VERIFIED` | M5, M7 | M | `test_example_dc_motor` | DC motor back-EMF and electromagnetic torque transduction verified. |
| **F5.42** | Electrical-thermal coupling | `TODO` | F5.12, F5.40 | M | - | Temperature-dependent electrical resistance and semiconductor thermal loss. |
| **F5.43** | Hydraulic-mechanical coupling | `TODO` | F5.26 | M | - | Direct mechanical linkage between hydraulic pistons and mechanical loads. |
| **F5.44** | Electromagnetic actuator models | `EXISTS-VERIFIED` | M5, M7 | M | `test_example_voice_coil_actuator` | Voice coil Lorentz force and back-EMF transducer verified. |
| **F5.45** | Unit-consistent physical ports | `TODO` | F2.00 | M | - | Compile-time dimensionality and unit compatibility checking across pins. |
| **F5.46** | Physical network initialization | `TODO` | F4.05 | L | - | Automated steady-state DC equilibrium solve across all coupled domains. |

---

### Phase F6: State Machines (15 Items)

- **Goal**: Deterministic finite state machine and statechart modeling engine with visual animation, hierarchical states, temporal logic, and embedded C code generation.
- **Entry Criteria**: Hierarchical subsystem interfaces (F3) and event scheduler (F4).
- **Exit Criteria**: Documented execution priority and transition semantics; state machine simulation traces verified by tests; generated C matches simulation traces.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F6.01** | State diagram editor | `TODO` | F1.01 | XL | - | Interactive canvas editor for authoring states, transitions, and junctions. |
| **F6.02** | States and transitions | `TODO` | F6.01 | M | - | Formal representation of active states, directed transitions, and labels. |
| **F6.03** | Entry, during and exit actions | `TODO` | F6.02 | M | - | State lifecycle action hooks executing assignment and event logic. |
| **F6.04** | Guard conditions | `TODO` | F6.02 | S | - | Boolean guard expressions enabling or inhibiting transition firing. |
| **F6.05** | Events | `TODO` | F6.02 | M | - | Broadcast and directed event triggers initiating state transitions. |
| **F6.06** | Timers | `TODO` | F6.02 | S | - | Temporal logic timers evaluating elapsed time and tick counts in state. |
| **F6.07** | Hierarchical state machines | `TODO` | F6.02 | L | - | Nested superstates and sub-states with inherited transition properties. |
| **F6.08** | Parallel states | `TODO` | F6.07 | L | - | Orthogonal AND-state decomposition with deterministic execution ordering. |
| **F6.09** | State transition tables | `TODO` | F6.02 | M | - | Tabular matrix view for editing and inspecting state transitions. |
| **F6.10** | Truth tables | `TODO` | - | M | - | Combinatorial condition-action matrix evaluating decision branches. |
| **F6.11** | Flow charts | `TODO` | F6.02 | M | - | Visual flow-chart logic graphs with conditional junction branches. |
| **F6.12** | State animation | `TODO` | F6.01 | M | - | Visual live state highlighting and active transition glow during simulation. |
| **F6.13** | State transition debugging | `TODO` | F6.12 | M | - | Step-by-step statechart execution with transition breakpoints and inspect. |
| **F6.14** | State coverage | `TODO` | F6.02 | M | - | Execution coverage metrics reporting state, transition, and guard coverage. |
| **F6.15** | State-machine C code generation | `TODO` | F6.02, F11.01 | L | - | Clean, MISRA-compliant C switch-case or state-table dispatch code generation. |

---

### Phase F7: Data Inspector (18 Items)

- **Goal**: High-capacity signal visualization, multi-run comparison, waveform cursor metrics, and multi-format data export with strict memory capping.
- **Entry Criteria**: Measurements engine (M8) and desktop plotting panels.
- **Exit Criteria**: Runs recorded, compared, and exported; session files save/load round-trip; signal metric extraction passes analytical golden tests.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F7.01** | Live scopes | `TODO` | - | M | - | Real-time streaming time-domain oscilloscope canvas during execution. |
| **F7.02** | Multiple scope windows | `TODO` | F7.01 | M | - | Detachable multi-channel scope displays with independent axes. |
| **F7.03** | Signal logging | `TODO` | - | M | - | High-throughput structured signal logging into ring-buffer storage. |
| **F7.04** | Log selected signals without Scope | `TODO` | F7.03 | S | - | Context-menu badge marking any diagram wire for background logging. |
| **F7.05** | Simulation run history | `TODO` | F7.03 | M | - | Indexed session database cataloging previous simulation runs. |
| **F7.06** | Compare multiple runs | `TODO` | F7.05 | M | - | Multi-run overlay visualization with color-coded signal traces. |
| **F7.07** | Baseline comparison | `TODO` | F7.06 | M | - | Formal baseline run selection with automatic tolerance envelope overlay. |
| **F7.08** | Difference plots | `TODO` | F7.07 | S | - | Computed numerical difference $y_{test}(t) - y_{base}(t)$ signal plotting. |
| **F7.09** | Data cursors | `TODO` | F7.01 | S | - | Interactive dual vertical cursors calculating $\Delta t$, $\Delta y$, and slope. |
| **F7.10** | Peak measurement | `TODO` | F7.09 | S | - | Automated local maxima/minima extraction and peak-to-peak amplitude display. |
| **F7.11** | RMS measurement | `EXISTS-VERIFIED` | - | S | `test_pure_sine_wave_rms_mean_dc` | True RMS sliding-window algorithm verified with Neumaier summation. |
| **F7.12** | Overshoot & settling time | `TODO` | F7.05 | M | - | Automated transient metrics: rise time, peak overshoot %, settling time. |
| **F7.13** | XY plots | `TODO` | F7.01 | M | - | Phase plane and parametric state cross-plots ($x_1$ vs $x_2$). |
| **F7.14** | FFT plots | `EXISTS-UNVERIFIED` | - | M | `test_spectrum_fft_and_dft_and_window_functions` | Cooley-Tukey FFT engine verified; interactive UI frequency plot pending. |
| **F7.15** | Spectrogram | `EXISTS-UNVERIFIED` | - | M | `test_spectrum_fft_and_dft_and_window_functions` | Short-time Fourier transform engine verified; waterfall heat-map UI pending. |
| **F7.16** | Replay results | `TODO` | F7.05 | M | - | Time-scrubber bar replaying past simulation traces and canvas states. |
| **F7.17** | Save analysis sessions | `TODO` | - | S | - | Serialization of plotted signals, cursors, annotations, and run comparisons. |
| **F7.18** | Export MAT, CSV and XLSX | `EXISTS-VERIFIED` | - | M | `test_csv_export_format_and_escaping` | RFC 4180 CSV export with formula injection escaping and MAT v5 verified. |

---

### Phase F8: Control Engineering (21 Items)

- **Goal**: Linear control system design and synthesis tools, frequency-domain stability analysis, state estimation, and automated controller tuning.
- **Entry Criteria**: Reference control models (M10) and linear algebra math crates.
- **Exit Criteria**: Frequency responses and stability margins match textbook closed-form solutions; numerical linearization matches analytical Jacobians; LQR/Kalman match Riccati solutions.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F8.01** | State-space models | `EXISTS-VERIFIED` | - | M | `test_m3_transfer_function_analytical_step` | Continuous and discrete state-space formulation with analytical step response. |
| **F8.02** | Transfer-function composition | `EXISTS-VERIFIED` | - | M | `test_m3_transfer_function_analytical_step` | Canonical controllable companion form transfer function evaluation. |
| **F8.03** | Zero-pole-gain models | `TODO` | F8.02 | M | - | ZPK representation and polynomial conversion algorithms. |
| **F8.04** | Step response | `EXISTS-VERIFIED` | - | S | `test_simulation_first_order_step_response` | Automated transient time-domain unit step response calculation. |
| **F8.05** | Impulse response | `TODO` | F8.04 | S | - | Continuous and discrete impulse response computation. |
| **F8.06** | Bode | `EXISTS-VERIFIED` | - | M | `test_bode_first_order_system` | Frequency response magnitude and phase array calculation. |
| **F8.07** | Nyquist | `TODO` | F8.06 | M | - | Complex plane Nyquist polar plots $(-1, j0)$ with encirclement counting. |
| **F8.08** | Nichols | `TODO` | F8.06 | M | - | Nichols chart plotting open-loop gain vs phase with closed-loop M-circles. |
| **F8.09** | Root locus | `TODO` | F8.02 | L | - | Evans root locus trajectory plotting poles as scalar gain $K$ varies $0 \to \infty$. |
| **F8.10** | Gain and phase margins | `EXISTS-VERIFIED` | F8.06 | M | `test_bode_first_order_system` | Exact gain margin, phase margin, and crossover frequency extraction. |
| **F8.11** | Stability analysis | `TODO` | F8.01 | M | - | Eigenvalue analysis, Routh-Hurwitz, and Lyapunov matrix equation solving. |
| **F8.12** | Model linearization | `TODO` | F4.01 | L | - | Numerical Jacobian perturbation linearizing non-linear diagrams around $(x_0, u_0)$. |
| **F8.13** | Operating-point calculation | `TODO` | F8.12 | M | - | Non-linear steady-state equilibrium solve $\dot{x} = f(x_0, u_0) = 0$. |
| **F8.14** | PID auto-tuning | `EXISTS-UNVERIFIED` | - | L | `test_m3_pid_analytical_and_antiwindup` | PID block verified; automated relay-feedback frequency response tuning pending. |
| **F8.15** | LQR | `TODO` | F8.01 | L | - | Linear-Quadratic Regulator solving continuous algebraic Riccati equation. |
| **F8.16** | LQG | `TODO` | F8.15 | L | - | Linear-Quadratic-Gaussian optimal control coupling LQR and Kalman filter. |
| **F8.17** | Kalman filter | `TODO` | F8.01 | L | - | Discrete-time steady-state and time-varying optimal state estimator. |
| **F8.18** | State observer | `TODO` | F8.01 | M | - | Luenberger full-order and reduced-order deterministic state observers. |
| **F8.19** | Pole placement | `TODO` | F8.01 | M | - | Ackermann formula and robust state-feedback pole placement algorithms. |
| **F8.20** | Model predictive control | `TODO` | F8.01 | XL | - | Constrained receding-horizon quadratic programming optimal controller. |
| **F8.21** | System identification | `TODO` | F7.03 | L | - | Parameter estimation fitting ARX/state-space models to measured input-output data. |

---

### Phase F9: Optimization and Design Exploration (13 Items)

- **Goal**: Multi-parameter sensitivity analysis, Monte Carlo statistical runs, least-squares parameter estimation, and Pareto multi-objective optimization.
- **Entry Criteria**: Parameter sweep engine (M12).
- **Exit Criteria**: Seeded stochastic runs produce identical distributions; parameter estimation recovers true parameters from noisy synthetic data; Pareto front verified on test problems.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F9.01** | Multi-parameter sweep | `EXISTS-VERIFIED` | - | M | `test_parameter_sweep_gain_variation` | Multi-dimensional grid sweep engine verified across diagram components. |
| **F9.02** | Sensitivity analysis | `TODO` | F9.01 | M | - | Local and global parameter sensitivity indices $\partial y / \partial \theta$. |
| **F9.03** | Monte Carlo simulation | `TODO` | F9.01 | L | - | Seeded stochastic batch simulation across parameter probability distributions. |
| **F9.04** | Design of experiments | `TODO` | F9.01 | M | - | Full factorial, fractional factorial, and Latin hypercube sampling designs. |
| **F9.05** | Component tolerance analysis | `TODO` | F9.03 | M | - | Worst-case corner analysis and RSS statistical tolerance stack-up. |
| **F9.06** | Parameter estimation | `TODO` | F7.03 | L | - | Non-linear least-squares fitting model parameters to measured experimental data. |
| **F9.07** | Least-squares optimization | `TODO` | F9.06 | M | - | Levenberg-Marquardt and Gauss-Newton curve fitting algorithms. |
| **F9.08** | Nonlinear optimization | `TODO` | - | L | - | BFGS and Nelder-Mead simplex unconstrained/constrained optimization. |
| **F9.09** | Multi-objective optimization | `TODO` | F9.08 | L | - | Weighted-sum and $\epsilon$-constraint multi-criteria optimization. |
| **F9.10** | Pareto analysis | `TODO` | F9.09 | M | - | Non-dominated sorting extracting Pareto-optimal trade-off frontiers. |
| **F9.11** | Automatic controller tuning | `TODO` | F8.14, F9.08 | L | - | Numerical optimization tuning gains to minimize integral error (ITAE/ISE). |
| **F9.12** | Robustness analysis | `TODO` | F9.03 | M | - | Stability and performance margin testing across extreme parametric envelopes. |
| **F9.13** | Design constraint checking | `TODO` | - | S | - | Automated assertion checking verifying signals remain within safety envelopes. |

---

### Phase F10: Testing and Verification (18 Items)

- **Goal**: Model test harness management, back-to-back model vs generated code verification, structural coverage (MC/DC), and automated diagnostic repairs.
- **Entry Criteria**: Diagnostic engine (M9) and CLI execution test runner.
- **Exit Criteria**: Model test suite executes headless from CLI producing verified reports; structural coverage verified on test model; model-vs-C equivalence verified in CI.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F10.01** | Model test manager | `TODO` | - | L | - | Test suite hierarchy defining test cases, parameter inputs, and pass/fail criteria. |
| **F10.02** | Test harnesses | `TODO` | F3.01, F10.01 | M | - | Isolated test wrapper diagrams feeding test stimuli and evaluating outputs. |
| **F10.03** | Unit testing of individual blocks | `EXISTS-VERIFIED` | - | S | `test_m3_spec_table_definitions` | Comprehensive unit tests verifying execution across individual block types. |
| **F10.04** | Functional testing | `TODO` | F10.01 | M | - | Black-box behavioral validation against formal functional requirements. |
| **F10.05** | Regression testing | `TODO` | F10.01 | M | - | Automated regression testing asserting outputs match golden reference vectors. |
| **F10.06** | Baseline testing | `TODO` | F10.05 | S | - | Multi-version baseline comparison detecting unwanted numerical divergence. |
| **F10.07** | Model-versus-C comparison | `EXISTS-VERIFIED` | M11 | L | `test_codegen_circuit_mna_solver` | Back-to-back numerical comparison asserting simulation equals C executable output. |
| **F10.08** | Input/output assertions | `TODO` | - | S | - | Assertion blocks checking value bounds, rates of change, and conditions. |
| **F10.09** | Requirements traceability | `TODO` | F10.01 | M | - | Bi-directional linking connecting model blocks to functional requirement IDs. |
| **F10.10** | Block coverage | `TODO` | - | M | - | Execution coverage metric reporting percentage of blocks executed in test. |
| **F10.11** | Decision coverage | `TODO` | F10.10 | M | - | Branch coverage verifying both true and false paths evaluated for decisions. |
| **F10.12** | Condition coverage | `TODO` | F10.11 | M | - | Verifying each boolean condition within compound decisions evaluated both ways. |
| **F10.13** | MC/DC coverage | `TODO` | F10.12 | L | - | Modified Condition/Decision Coverage for safety-critical DO-178C verification. |
| **F10.14** | State-transition coverage | `TODO` | F6.14 | M | - | Verifying all valid statechart transitions exercised during test run. |
| **F10.15** | Automated test reports | `TODO` | F10.01 | M | - | Formatted HTML and Markdown test summary reports detailing passed/failed runs. |
| **F10.16** | CI-based simulation testing | `TODO` | F10.15 | M | - | Automated GitHub Actions CI workflow executing model tests and archiving logs. |
| **F10.17** | Model checks (diagnostics) | `EXISTS-VERIFIED` | - | M | `test_m9_floating_island_and_autofix` | Static and topological compile-time diagnostics (`E-FLOAT`, `E-VLOOP`, `E-ICUT`, etc.). |
| **F10.18** | Automated model repairs | `EXISTS-VERIFIED` | F10.17 | M | `test_m9_floating_island_and_autofix` | `apply_fix` engine with 1-click automated diagram repair and single-level undo. |

---

### Phase F11: Code Generation and Embedded Deployment (20 Items)

- **Notice**: All 20 items in Phase F11 are designated `TODO` (or `NEEDS-HUMAN-HARDWARE`). While foundational ANSI/ISO C99 code generation exists from Milestone M11 (`test_codegen_signal_model_allocation_free`), all broader language extensions, embedded targets, real-time operating systems, and standards compliance are future work.
- **Goal**: Safety-critical, MISRA C-oriented code generation targeting microcontrollers, fixed-point processors, hardware-in-the-loop simulators, and hardware description languages.
- **Entry Criteria**: ISO C99 allocation-free generator (M11) and complete block library.
- **Exit Criteria**: Generated code compiles cleanly with gcc/clang under `-Wall -Wextra -Werror` matching simulation outputs; RAM/stack usage reports verified; SIL runs in CI.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F11.01** | Broader block coverage | `TODO` | M11 | L | - | Expanding C99 generator to non-linear, lookup table, and state blocks. |
| **F11.02** | C++ generation | `TODO` | F11.01 | L | - | Modern C++14/17 class-encapsulated code generation with clean namespaces. |
| **F11.03** | Fixed-point generation | `TODO` | F11.01 | L | - | Word-length and fractional-scaling integer Q-format fixed-point codegen. |
| **F11.04** | Stack and RAM usage reports | `TODO` | F11.01 | M | - | Static static analysis reporting stack frames and static BSS/data memory usage. |
| **F11.05** | Execution-time analysis | `TODO` | F11.04 | M | - | Profiling worst-case execution time (WCET) per simulation step. |
| **F11.06** | RTOS task generation | `TODO` | F11.01 | L | - | FreeRTOS and Zephyr periodic task threads with queue communication. |
| **F11.07** | Bare-metal target support | `TODO` | F11.01 | M | - | Minimal main loop execution clocked by hardware SysTick timer interrupts. |
| **F11.08** | STM32 target | `NEEDS-HUMAN-HARDWARE` | F11.07 | L | - | STM32Cube HAL integration; software simulation tests executed in CI. |
| **F11.09** | ESP32 target | `NEEDS-HUMAN-HARDWARE` | F11.07 | L | - | ESP-IDF build system integration and FreeRTOS task deployment. |
| **F11.10** | ARM Cortex-M support | `TODO` | F11.07 | M | - | CMSIS-DSP intrinsic function acceleration for matrix and vector operations. |
| **F11.11** | Peripheral configuration | `TODO` | F11.07 | M | - | Memory-mapped register abstraction for on-chip microcontroller peripherals. |
| **F11.12** | UART, SPI and I2C interfaces | `NEEDS-HUMAN-HARDWARE` | F11.11 | M | - | Serial bus communication peripheral drivers with simulated mock loopback. |
| **F11.13** | GPIO, ADC and PWM interfaces | `NEEDS-HUMAN-HARDWARE` | F11.11 | M | - | Microcontroller I/O pin drivers and analog conversion interfaces. |
| **F11.14** | Software-in-the-loop (SIL) | `TODO` | F11.01 | L | - | Host-compiled generated binary co-simulated with test harness in CI. |
| **F11.15** | Processor-in-the-loop (PIL) | `NEEDS-HUMAN-HARDWARE` | F11.14 | XL | - | Embedded code executing on physical target board communicating via UART. |
| **F11.16** | Hardware-in-the-loop (HIL) | `NEEDS-HUMAN-HARDWARE` | F11.15 | XL | - | Hard real-time target executing plant dynamics connected to controller. |
| **F11.17** | MISRA C-oriented generation | `TODO` | F11.01 | L | - | Strict adherence to MISRA C:2012 guidelines; verified with cppcheck reports. |
| **F11.18** | AUTOSAR integration | `TODO` | F11.02 | XL | - | AUTOSAR Classic and Adaptive software component (SWC) XML interfaces. |
| **F11.19** | PLC code generation | `TODO` | F11.01 | XL | - | IEC 61131-3 Structured Text (ST) generation for industrial PLCs. |
| **F11.20** | HDL generation | `TODO` | F11.03 | XL | - | Synthesis-ready VHDL and Verilog RTL generation for FPGA deployment. |

---

### Phase F12: External Integration (13 Items)

- **Goal**: Seamless interoperability with industry-standard simulation protocols (FMI/FMU), SPICE netlists, Modelica models, Python runtimes, and robotics simulators.
- **Entry Criteria**: Hierarchical model interfaces (F3) and simulation engine (F4).
- **Exit Criteria**: FMU import/export validated against official public FMI reference test models; Python bridge verified; ROS/Gazebo bridges verified with simulated mock nodes.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F12.01** | FMI importer | `TODO` | F3.01 | L | - | Import of Functional Mock-up Units conforming to FMI 2.0/3.0 standards. |
| **F12.02** | FMU Model Exchange | `TODO` | F12.01 | L | - | Numerical integration of imported dynamic state-derivative equations. |
| **F12.03** | FMU Co-Simulation | `TODO` | F12.01 | L | - | Lock-step co-simulation with imported FMU containing internal solver. |
| **F12.04** | FMU exporter | `TODO` | F11.01, F12.01 | XL | - | Exporting MatRust Studio models as standardized `.fmu` co-simulation packages. |
| **F12.05** | SPICE integration | `TODO` | M5 | L | - | Direct import and parsing of SPICE circuit netlists into electrical subcircuits. |
| **F12.06** | Modelica interoperability | `TODO` | F5.45 | XL | - | Bi-directional translation of standard acausal physical Modelica components. |
| **F12.07** | Python bridge | `EXISTS-UNVERIFIED` | - | M | `crates/mlapp-py` | PyO3 Python C-API bridge exists; block diagram scripting interface pending. |
| **F12.08** | C/C++ custom block API | `TODO` | F11.01 | M | - | Native C ABI interface allowing users to write compiled custom block dynamics. |
| **F12.09** | Extension API for user blocks | `TODO` | F12.08 | M | - | Dynamic plugin architecture for third-party custom block library distribution. |
| **F12.10** | ROS/ROS 2 bridge | `NEEDS-HUMAN-HARDWARE` | - | XL | - | ROS 2 DDS middleware pub/sub node communication interface for robotics. |
| **F12.11** | Gazebo integration | `NEEDS-HUMAN-HARDWARE` | F12.10 | XL | - | Synchronized physics co-simulation with Gazebo multi-body dynamics engine. |
| **F12.12** | Network-based co-simulation | `TODO` | - | L | - | Distributed UDP/TCP socket synchronization for multi-computer co-simulation. |
| **F12.13** | Time synchronization | `TODO` | F12.12 | M | - | PTP/NTP network clock synchronization enforcing deterministic step barriers. |

---

### Phase F13: Performance and Scale (13 Items)

- **Goal**: High-performance scaling for massive multi-domain models through multithreaded evaluation, sparse matrix algorithms, vectorization, and caching.
- **Entry Criteria**: MNA circuit engine (M5/M6) and continuous execution engine.
- **Exit Criteria**: Comprehensive benchmark suite; all acceleration claims substantiated with before/after measurements; bit-exact deterministic results; memory limits enforced.

| ID | Title | Status | Depends-On | Size | Evidence Test | Notes |
| :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| **F13.01** | Incremental model compilation | `TODO` | - | L | - | Caching unchanged subsystem execution graphs to avoid full recompilation. |
| **F13.02** | Cached execution plans | `TODO` | F13.01 | M | - | Serialized topological ordering and pre-factored matrices reused across runs. |
| **F13.03** | Sparse physical network solvers | `EXISTS-UNVERIFIED` | - | L | `crates/rustlab-math/src/sparse.rs` | CSC sparse matrix arithmetic exists; sparse LU MNA solver integration pending. |
| **F13.04** | Sparse Jacobian algorithms | `TODO` | F13.03 | L | - | Graph coloring Curtis-Powell-Reid sparse Jacobian finite-difference estimation. |
| **F13.05** | Safe block-level parallelism | `TODO` | - | L | - | Concurrent evaluation of independent DAG branches using Rayon work-stealing. |
| **F13.06** | Parallel independent simulations | `TODO` | - | M | - | Multi-threaded batch simulation harness running independent model instances. |
| **F13.07** | Parallel parameter sweeps | `EXISTS-UNVERIFIED` | - | M | `test_parameter_sweep_gain_variation` | Parameter sweep engine exists; multi-core parallel distribution pending. |
| **F13.08** | CPU vectorization | `EXISTS-UNVERIFIED` | - | M | `crates/rustlab-math/src/ops.rs` | SIMD vectorization and matrixmultiply kernels active in core numerical library. |
| **F13.09** | GPU acceleration | `TODO` | - | XL | - | Compute shader kernels (wgpu/Vulkan) offloading parallel Monte Carlo sweeps. |
| **F13.10** | Streaming data storage | `TODO` | F7.03 | M | - | Chunked binary disk buffering avoiding RAM exhaustion during gigabyte runs. |
| **F13.11** | Simulation checkpointing | `TODO` | F4.16 | M | - | Periodic disk snapshots allowing instant rollback and failure recovery. |
| **F13.12** | Memory & execution profiling | `TODO` | - | M | - | Per-block allocation tracking and sub-microsecond wall-clock timing telemetry. |
| **F13.13** | Reproducible benchmarks | `EXISTS-VERIFIED` | - | S | `benches/host_benchmarks.rs` | Standardized Criterion benchmark harness verifying performance regressions. |

---

### Phase F14: MATLAB Ecosystem Expansion (13 Domain Groups)

Phase F14 expands language and toolbox compatibility across the broader technical computing ecosystem. Specific items will be expanded via individual planning tasks (`PLAN-F14`):
1. **Programming Language**: Classdef OOP, packages, events/listeners, debugger hooks, execution profiler.
2. **Numerical Computing**: Stiff ODE solvers (`ode15s`, `ode23t`), root finding (`fzero`, `fsolve`), adaptive quadrature (`integral`).
3. **Optimization Toolbox**: `fminsearch`, `fminunc`, `lsqnonlin`, simplex `linprog`, quadratic programming `quadprog`.
4. **Data Engineering**: Tables, timetables, XLSX reader (calamine), JSON encoder/decoder, relational joins.
5. **Graphics & Visualization**: 3D surfaces, contour plots, vector fields, volumetric slices, animations (overlaps P4).
6. **Signal Processing**: Butterworth, Chebyshev, elliptic filter design, FIR windowing, `pwelch`, spectrogram, resampling.
7. **Audio Processing**: Audio file I/O (WAV/FLAC), sound synthesis, acoustic filtering, time-pitch scaling.
8. **Image Processing**: Spatial filtering, edge detection (Sobel, Canny), morphological operations, Otsu thresholding.
9. **Machine Learning**: Decision trees, random forests, k-means clustering, PCA dimensionality reduction, MLP neural networks.
10. **Robotics & Automation**: SE(3) transformations, Euler/quaternion kinematics, Denavit-Hartenberg parameters, trajectory planning.
11. **Parallel Computing**: Multi-core `parfor` scheduling, parallel batch execution, shared-memory worker pools.
12. **App Development**: Standalone desktop deployment, application packaging, UI component layout builder.
13. **Hardware Connectivity**: Serial port communication, TCP/UDP sockets, ring-buffered data acquisition streams.

---

## 4. Phase Accounting & Audit Summary

- **Total Phase Items (F1 through F13)**: **248 items** (exact match with charter count).
- **EXISTS-VERIFIED**: **20 items** (backed by named automated tests in the repository).
- **EXISTS-UNVERIFIED**: **12 items** (backend implemented; full UI or integration check pending).
- **TODO / NEEDS-HUMAN-HARDWARE**: **216 items** (future developmental roadmap).
