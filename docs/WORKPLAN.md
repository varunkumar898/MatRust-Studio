# MatRust Studio Master Workplan (docs/WORKPLAN.md)

This document establishes the overarching development waves, planning task protocols, and strategic execution sequencing for MatRust Studio.

---

## 1. Development Waves

### Wave 0: Foundations & Architecture Setup
- **W0.1**: Architectural Charter, Claims Matrix, and Zero-Copilot / Clean-Room Protocol definition.
- **W0.2**: Safe CI build pipelines, multi-platform test harnesses, and strict zero-warning compiler enforcement.
- **W0.3**: Modular workspace structure, license audits, and strict dependency boundary decoupling.

### Wave 1: Core Platform Engines (P1..P5)
- **P1 (Core Language & Mathematics)**: Pure Rust AST parser and evaluator, column-major multidimensional arrays, 1-based indexing, numeric types, and BLAS/LAPACK linear algebra.
- **P2 (Data Analysis & Visualization)**: Table data types, RFC 4180 CSV / XLSX format I/O, and deterministic SVG plot-description model.
- **P3 (Engineering Simulation Engine)**: Graphical block diagram data model, topological sorting, algebraic loop detection, and continuous/discrete ODE stepping.
- **P4 (Signal & Image Processing)**: Radix-2 FFT/DFT transforms, digital filtering, time-frequency windowing, and 2D image arrays.
- **P5 (Data Inspector & Diagnostics)**: Proactive static compilation and topological diagnostics (`E-FLOAT`, `E-VLOOP`, `E-ICUT`, `E-PORT`), true RMS, and RFC 4180 CSV export.

### Wave 2: Specialized Simulation & Domain Engines (S1..S4)
- **S1 (Modified Nodal Analysis - M5)**: Ho-Ruehli-Brennan MNA circuit formulation, Dommel companion models, dense LU factorization with partial row pivoting, and cold-start Backward Euler initialization.
- **S2 (Switching & Non-Linear Circuits - M6)**: Piecewise-linear semiconductor models (diodes, switches), fixed-point within-step switching iteration, limit cycle chattering control (`W-CHATTER` / `E-CHATTER`), and compile-time matrix pre-factorization.
- **S3 (Hybrid Bridges & Multi-Domain Coupling - M7)**: Continuous-to-electrical and electrical-to-continuous transducers (voltage/current sensors, controlled sources) with ZOH input sampling and end-of-step measurement readouts.
- **S4 (Code Generation & Workflow Polish - M11 & M12)**: ANSI/ISO C99 allocation-free code generator, MNA C circuit solver, multi-dimensional parameter sensitivity sweeps, multi-level undo/redo, and simulation thread cancellation.

### Backlog
Phases F1..F14: see docs/WORKPLAN-PHASES.md

---

## 2. Planning Tasks Protocol (PLAN-F1 .. PLAN-F14)

Before implementation work begins on any phase, a formal planning task **PLAN-F<n>** must be submitted and approved by the project owner. Each planning task defines:
1. **Itemized Work Breakdown**: Decomposing each phase item into concrete tasks, identifying explicit dependencies and required Architecture Decision Records (ADRs).
2. **Sizing & Subtask Decomposition**: Categorizing tasks by complexity (**S**, **M**, **L**, **XL**). Any task sized **XL** must be split into subtasks of size **L** or smaller prior to implementation. Planning tasks may merge small sibling tasks (**S**) into cohesive single PRs.
3. **Acceptance Criteria & Oracles**: Formulating exact known-answer numerical tests, closed-form analytical solutions, or textbook benchmark comparisons before writing production code.
4. **Touch List & Blast Radius Estimate**: Documenting all crates, source modules, and public APIs expected to be touched.
5. **Storage & Performance Budget Impact**: Estimating the impact on release executable size, installer footprint, and memory ceilings against [docs/STORAGE-BUDGET.md](file:///<repo>/docs/STORAGE-BUDGET.md).
6. **Integrity Invariant**: Planning tasks may reorder tasks internally or combine small items, but may **never drop an item**. Only the project owner can designate an item as `DEFERRED`.

### Planning Task Catalog

- **PLAN-F1**: Planning for Phase F1 (Professional Block Editor)
- **PLAN-F2**: Planning for Phase F2 (Complete Block Library) — Includes prerequisite task **F2.00** (Tensor signal dimensions and typing model)
- **PLAN-F3**: Planning for Phase F3 (Hierarchical Modeling)
- **PLAN-F4**: Planning for Phase F4 (Advanced Simulation Engine)
- **PLAN-F5**: Planning for Phase F5 (Multidomain Physical Simulation)
- **PLAN-F6**: Planning for Phase F6 (State Machines)
- **PLAN-F7**: Planning for Phase F7 (Data Inspector)
- **PLAN-F8**: Planning for Phase F8 (Control Engineering)
- **PLAN-F9**: Planning for Phase F9 (Optimization and Design Exploration)
- **PLAN-F10**: Planning for Phase F10 (Testing and Verification)
- **PLAN-F11**: Planning for Phase F11 (Code Generation and Embedded Deployment)
- **PLAN-F12**: Planning for Phase F12 (External Integration)
- **PLAN-F13**: Planning for Phase F13 (Performance and Scale)
- **PLAN-F14**: Planning for Phase F14 (MATLAB Ecosystem Expansion)

---

## 3. Strategic Execution Sequence & Dependency Justification

The planned execution sequence across waves and phases is strictly ordered as follows:

$$\text{Wave 0} \longrightarrow \text{Wave 1 (P1..P5)} \longrightarrow \text{Wave 2 (S1..S4)} \longrightarrow \mathbf{F1} \longrightarrow \mathbf{F2} \longrightarrow \mathbf{F3} \longrightarrow \mathbf{F4} \longrightarrow \mathbf{F5} \longrightarrow \mathbf{F7} \longrightarrow \mathbf{F8} \longrightarrow \mathbf{F6} \longrightarrow \mathbf{F9} \longrightarrow \mathbf{F10} \longrightarrow \mathbf{F11} \longrightarrow \mathbf{F12} \longrightarrow \mathbf{F13} \longrightarrow \mathbf{F14}$$

### Dependency Justifications for Sequencing

1. **Waves 0–2 $\to$ F1, F2**: Professional block editing (**F1**) and core block expansion (**F2**) build directly upon the existing graphical block diagram canvas and simulation execution foundation completed in Waves 0–2 (M1–M12).
2. **F1, F2 $\to$ F3, F4**: Hierarchical composition (**F3**) and advanced numerical engines (**F4**) require fully established block interfaces, signal widths, and basic continuous/discrete blocks from **F1** and **F2**.
3. **F4 $\to$ F5**: Physical multi-domain networks (**F5**) (mechanical, thermal, hydraulic, electrical) generate stiff differential-algebraic equations (DAEs) and frequent non-smooth switching events; they strictly require the stiff solvers (Radau/BDF) and $10^{-9}$ s zero-crossing localization built in **F4**.
4. **F5 $\to$ F7, F8**: The Data Inspector (**F7**) and Control Engineering tools (**F8**) require rich multi-domain plants and physical signals (**F5**) to analyze time-domain responses, calculate Bode stability margins, and display multi-channel waveforms.
5. **F7, F8 $\to$ F6**: State Machines (**F6**) are sequenced *after* **F7** and **F8** because statecharts act as supervisory controllers that govern multi-domain plant models; developing state machines requires live signal scopes (**F7**) and linear stability tools (**F8**) to monitor and tune supervisory transitions.
6. **F6 $\to$ F9, F10**: Optimization design exploration (**F9**) and formal verification harnesses (**F10**) require complete physical plants and supervisory state logic to run automated Monte Carlo sweeps, fault-injection tests, and MC/DC structural coverage analysis.
7. **F10 $\to$ F11**: Embedded code generation (**F11**) requires the complete, formally verified block and state machine catalog (**F1..F10**) so that embedded target outputs match verified simulation baselines.
8. **F11 $\to$ F12, F13**: External co-simulation integration (**F12**) (FMI/FMU, ROS, Gazebo) and performance scaling (**F13**) (parallel sweeps, sparse matrix solvers) leverage the generated C models and compiled execution plans.
9. **F13 $\to$ F14**: Broad MATLAB ecosystem and toolbox parity (**F14**) builds upon the mature high-performance simulation engine.
