# MatRust Studio Master Workplan (docs/WORKPLAN.md)

This document establishes the overarching development waves, planning task protocols, and strategic execution sequencing for MatRust Studio.

The waves and phases below are **planned work**, not completed work. Completed milestones are recorded separately in "Completed" and are never labelled with the wave labels.

> **Provenance note (A2 correction):** the Wave 0-2 task text below is reconstructed from the owner's correction because the verbatim bootstrap draft was not present in the repository or its history. The owner is asked to confirm the task wording and TEST lists; nothing here is implemented in this session.

---

## 1. Development Waves

### Wave 0: Hygiene and Layout Foundations (H1, L1..L4)

- **H1 (auto) - Repository hygiene and truth audit**: no personal paths or secrets in tracked files; every documented count matches a real command output; `docs/test-manifest.txt` matches `cargo test -- --list`; every capability claim in `README.md` is backed by `docs/CLAIMS.md` and a named test. `auto` (scripts only).
  - TEST: `scripts/check_docs.py` (privacy and link checks); manifest check; `scripts/verify.ps1` from a clean tree.
- **L1 - Layout audit (read-only)**: review the desktop layout against contracts C1-C12, using real-window captures at the supported sizes. No code changes.
  - TEST: a completed C1-C12 table in `docs/layout-audit.md`; captured images referenced by path and size.
- **L2 - Layout probe and contract checks**: add a read-only layout probe, a `check_layout` entry point, a minimum-size table, the C1 container check, the L-C size sweep and the L-E mutation check. Findings only; no panel fixes.
  - TEST: probe unit tests; `check_layout` returns the documented report for each capture; min-size table rows exercised.
- **L3 - Responsive layout fixes (auto where measurable)**: fix the L2 findings so no panel overlaps or clips at 900x600, 1280x800 and 1920x1080; keep the standard dock preset intact.
  - TEST: three-size capture set with a per-panel no-overlap/no-clip assertion; existing GUI tests stay green. `auto` for the geometry assertions.
- **L4 - Layout regression gate (auto)**: make L3 measurable in CI with stored snapshots and a failing check when a panel leaves its minimum rect.
  - TEST: CI job that fails on a layout snapshot change without an approved update; the gate runs on every pull request.

### Wave 1: Plotting and Visualization (P1..P5)

- **P1 - Subplot rendering**: render an m-by-n grid of axes with per-cell placement, spanning cells, shared axis support and deterministic output ordering. Every subplot draws into its own rect; no axes may bleed into a neighbour.
  - TEST: 1x1, 2x1, 2x2 and one spanning layout at 1280x800; byte-equal golden output; empty-subplot case; `subplot(2,2,3)` places the axes in the expected cell.
- **P2 - Independent axes properties**: each subplot owns its limits, scales (linear and log), tick locations, tick labels, grid, title, axis labels and legend. Changing one subplot must not change another.
  - TEST: two subplots with different `xlim`, `ylim` and scale keep independent state; a serialization round-trip preserves each axes; a regression test proves no cross-talk.
- **P3 - Per-subplot interaction**: select, hover, zoom and pan the subplot under the pointer; a readout shows the cursor value; linked axes are opt-in. Interaction state is per axes, not per figure.
  - TEST: a pointer hit-test maps a click to the correct axes; zooming one subplot leaves the other limits unchanged; a linked pair moves together; the readout matches the data value.
- **P4 - 3D plotting**: line, surface and scatter plots with view angles, colour maps, depth ordering and labelled axes.
  - TEST: a known surface (for example z = x^2 + y^2) sampled at fixed grid points; a projection golden; axis labels and colour-map range asserted; an empty-data case.
- **P5 - Simulation result visualization and time-series analysis**: plot simulation outputs as multi-signal time series, with cursors, min/max/mean/RMS readouts and export.
  - TEST: run a known model and assert the plotted series equals the simulation's time and output vectors; measurement readouts match analytic values; CSV and MAT export round-trip.

### Wave 2: Repository Engineering Hygiene (S1..S4)

- **S1 - Build-size hygiene (auto)**: measure release executable and installer size, record the numbers, and fail a release when a documented budget is exceeded. Release profile only; state the profile, target and thread count with every number.
  - TEST: a size report script writes the numbers to a dated file; the budget check fails on a deliberate size regression; the report matches `docs/STORAGE-BUDGET.md`.
- **S2 - Dependency audit (auto)**: check licence allow-list, duplicate crate versions, unused dependencies and optional-feature gating on every change.
  - TEST: `cargo deny check licenses` saved; a duplicate-crate report (`cargo tree -d`); the feature-gated heavy crates are absent from the default build.
- **S3 - Architecture rules as tests (auto)**: encode dependency direction and safety rules as executable checks: core numerical crates must not depend on GUI, plotting, machine-learning or hardware crates; library crates deny `unwrap`, `expect`, `panic`, `todo` and `unimplemented`; no `unsafe` without a written justification.
  - TEST: an architecture test target that fails when a forbidden edge or lint is added; the check runs in CI and locally.
- **S4 - Budget gates (auto)**: make the S1-S3 checks release gates alongside the test-count, documentation-coverage, memory and performance budgets; a regression fails the gate with the measured value.
  - TEST: a CI workflow step for each gate; a negative test proves each gate fails when its threshold is crossed; the gates run from a clean build.

### Backlog
Phases F1..F16: see [docs/WORKPLAN-PHASES.md](WORKPLAN-PHASES.md)

---

## 2. Completed Work (M1..M12)

These milestones are **complete and merged**. SHAs are the merge or squash commits on `origin/main`, read from git in this session; PR numbers are from the commit subjects. Claims are limited to [docs/CLAIMS.md](CLAIMS.md) and named tests. No completed milestone reuses a wave label.

| Milestone | Delivered | Merge commit | PR | Evidence |
| :--- | :--- | :--- | :---: | :--- |
| M1 | Pure signal execution engine: pure `eval()`, topological order, algebraic-loop detection (`E-ALG`) | `45709131edc94dc21b0020d5512b536dd5121f92` | #29 | CLAIMS §1 |
| M2 | Numerical solvers: Euler, RK4, Dormand-Prince 5(4); solver diagnostics | `75fe6810a5518ea5c20c3e46326a7acfd139942c` | #30 | CLAIMS §1 |
| M3 | Sources and linear/nonlinear control blocks; spec table; PID with anti-windup | `bcfcb749de5264aa951c2fee457ecd032679b837` | #32 | CLAIMS §1 |
| M4 | Electrical data model: PortKind, union-find nets, editor integration | `c7f872011fcf8cc8c74b4d8d7de85b62a2d4cbcc` | #35 | CLAIMS §2 |
| M5 | Modified Nodal Analysis, companion models, dense LU with partial pivoting, `E-SINGULAR` | `115a66c635236fae362425927c38d9fad6fdf111` | #36 | CLAIMS §2 |
| M6 | Piecewise-linear diode and ideal switch, within-step iteration, `W-CHATTER`/`E-CHATTER` | `2dd7b5e3860931ce74a3a03aa120cbe523e4bce5` | #37 | CLAIMS §2 |
| M7 | Voltage/current sensors, controlled sources, cross-domain `W-DELAY`, op-amp macro-model | `25c3fa4c1c1268aef547c0e6c195f422fcc55d1e` | #38 | CLAIMS §3 |
| M8 | RMS/mean, power, FFT/DFT, THD, RFC 4180 CSV export | `4684866bb2db7d31dcca15b83ca848bf346e062c` | #39 | CLAIMS §4 |
| M9 | Diagnostics `E-FLOAT`, `E-VLOOP`, `E-ICUT`, `E-PORT`, `W-STEP`; `apply_fix`; CLI `check` | `8afd8b555a8605e1df20ef39e11674facc7d76be` | #40 | CLAIMS §5 |
| M10 | Eight worked example models with analytic or physical oracles | `ffe6ef19d1287396163262e9335aef89b1c059ed` | #41 | CLAIMS §6 |
| M11 | ISO C99 allocation-free code generation and generated MNA solver | `ccbf804ea84598012555a4c98c19144b01bc6a9f` | #42 | `test_codegen_signal_model_allocation_free`, `test_codegen_circuit_mna_solver` |
| M12 | Parameter sweep engine; simulation cancel flag; editor undo/redo stacks | `ccbf804ea84598012555a4c98c19144b01bc6a9f` | #42 | `test_parameter_sweep_gain_variation`, `test_session_cancellation_flag`; undo/redo has no automated test (`EXISTS-UNVERIFIED`) |

The interactive desktop editor remains **GUI: unverified** per CLAIMS §7.

---

## 3. Planning Tasks Protocol (PLAN-F1 .. PLAN-F16)

Before implementation work begins on any phase, a formal planning task **PLAN-F<n>** must be submitted and approved by the project owner. Each planning task defines:
1. **Itemized Work Breakdown**: Decomposing each phase item into concrete tasks, identifying explicit dependencies and required Architecture Decision Records (ADRs).
2. **Sizing & Subtask Decomposition**: Categorizing tasks by complexity (**S**, **M**, **L**, **XL**). Any task sized **XL** must be split into subtasks of size **L** or smaller prior to implementation. Planning tasks may merge small sibling tasks (**S**) into cohesive single PRs.
3. **Acceptance Criteria & Oracles**: Formulating exact known-answer numerical tests, closed-form analytical solutions, or textbook benchmark comparisons before writing production code.
4. **Touch List & Blast Radius Estimate**: Documenting all crates, source modules, and public APIs expected to be touched.
5. **Storage & Performance Budget Impact**: Estimating the impact on release executable size, installer footprint, and memory ceilings against [docs/STORAGE-BUDGET.md](STORAGE-BUDGET.md).
6. **Integrity Invariant**: Planning tasks may reorder tasks internally or combine small items, but may **never drop an item**. Only the project owner can designate an item as `DEFERRED`.

### Planning Task Catalog

- **PLAN-F1**: Planning for Phase F1 (Professional Block Editor)
- **PLAN-F2**: Planning for Phase F2 (Complete Block Library) - Includes prerequisite task **F2.00** (Tensor signal dimensions and typing model)
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
- **PLAN-F15**: Planning for Phase F15 (Octave Conformance) - Emits one task per batch of at most 15 functions after F15.01
- **PLAN-F16**: Planning for Phase F16 (Julia Bridge and Export)

---

## 4. Strategic Execution Sequence & Dependency Justification

The planned execution sequence across waves and phases is strictly ordered as follows:

$$\text{Wave 0} \longrightarrow \text{Wave 1 (P1..P5)} \longrightarrow \text{Wave 2 (S1..S4)} \longrightarrow \mathbf{F15.00} \longrightarrow \mathbf{F15.01} \longrightarrow \mathbf{F1} \longrightarrow \mathbf{F2} \longrightarrow \mathbf{F3} \longrightarrow \mathbf{F4} \longrightarrow \mathbf{F5} \longrightarrow \mathbf{F7} \longrightarrow \mathbf{F8} \longrightarrow \mathbf{F6} \longrightarrow \mathbf{F9} \longrightarrow \mathbf{F10} \longrightarrow \mathbf{F11} \longrightarrow \mathbf{F12} \longrightarrow \mathbf{F16} \longrightarrow \mathbf{F13} \longrightarrow \mathbf{F14}$$

F15.02..F15.12 (the function batches) are interleaved with the sequence by PLAN-F15, using the real category counts from F15.01.

### Dependency Justifications for Sequencing

1. **Waves 0-2 $\to$ F15.00, F15.01**: The differential harness and the gap report depend on repository hygiene, plotting and build hygiene being measurable first; F15.01's real counts then decide the F15 batch order.
2. **F15.00, F15.01 $\to$ F1, F2**: The gap report informs which core arithmetic and linear-algebra functions the block editor and library need first, without displacing the F-sequence.
3. **F1, F2 $\to$ F3, F4**: Hierarchical composition (**F3**) and advanced numerical engines (**F4**) require fully established block interfaces, signal widths, and basic continuous/discrete blocks from **F1** and **F2**.
4. **F4 $\to$ F5**: Physical multi-domain networks (**F5**) (mechanical, thermal, hydraulic, electrical) generate stiff differential-algebraic equations (DAEs) and frequent non-smooth switching events; they strictly require the stiff solvers (Radau/BDF) and $10^{-9}$ s zero-crossing localization built in **F4**.
5. **F5 $\to$ F7, F8**: The Data Inspector (**F7**) and Control Engineering tools (**F8**) require rich multi-domain plants and physical signals (**F5**) to analyze time-domain responses, calculate stability margins, and display multi-channel waveforms.
6. **F7, F8 $\to$ F6**: State Machines (**F6**) are sequenced *after* **F7** and **F8** because statecharts act as supervisory controllers that govern multi-domain plant models; developing state machines requires live signal scopes (**F7**) and linear stability tools (**F8**) to monitor and tune supervisory transitions.
7. **F6 $\to$ F9, F10**: Optimization design exploration (**F9**) and formal verification harnesses (**F10**) require complete physical plants and supervisory state logic to run automated Monte Carlo sweeps, fault-injection tests, and MC/DC structural coverage analysis.
8. **F10 $\to$ F11**: Embedded code generation (**F11**) requires the complete, formally verified block and state machine catalog (**F1..F10**) so that embedded target outputs match verified simulation baselines.
9. **F11 $\to$ F12**: External co-simulation integration (**F12**) (FMI/FMU, ROS, Gazebo) leverages the generated C models and compiled execution plans.
10. **F12 $\to$ F16**: The Julia bridge reuses the external-process bridge pattern established by the Python bridge in F12.07 and the Octave bridge; F16 stays opt-in and outside the base installer.
11. **F13 $\to$ F14**: Broad ecosystem and toolbox parity (**F14**) builds upon the mature high-performance simulation engine; the graphics group reuses the Wave 1 plot model.
