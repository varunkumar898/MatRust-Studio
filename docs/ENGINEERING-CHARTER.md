# ENGINEERING CHARTER - MatRust Studio Simulation Platform

*Single source of truth for architecture, principles, workflow, and the end result.*

## 1. WHAT WE ARE BUILDING
A block-diagram and circuit simulator inside MatRust Studio (Rust workspace,
egui desktop app, `crates/mlapp-sim` for the core, `apps/desktop` for the
editor). One diagram can mix signal-flow blocks (sources, math, control) and
electrical networks (R, L, C, sources, diode, switch), be checked for errors
before running, be simulated with fixed- or variable-step solvers, be inspected
in scopes and spectra, be exported to CSV, and be turned into allocation-free
C for a microcontroller.

## 2. CORE PRINCIPLES (every change must respect them)
- **P1 Single source of truth**: ONE per-block-type spec table (keys, labels, units,
  defaults, ports, feedthrough per input port, validation) drives the editor,
  the validator, the docs and the code generator. No duplicated block lists.
- **P2 Pure evaluation, explicit state**: `eval(t, states, workspace)` never mutates
  anything. Discrete states change only in `update_discrete`, once per accepted
  major step. State layouts (continuous and discrete offsets) are compiled once.
- **P3 Determinism**: stable ordering (ties broken by block order in the file),
  inputs summed in port order, time as $t_{start} + k \cdot dt$ with an integer $k$,
  so results are reproducible bit for bit. Exact bit comparison wherever no
  `sin`/`cos`/`exp`/`powf` is involved; a relative tolerance (about $10^{-12}$) only for
  signals computed from transcendentals.
- **P4 Fail loudly and helpfully**: every error is a typed diagnostic with a code, a
  message in the form *what happened -> why -> how to fix*, and the SIDs of the
  blocks involved. Never fall back silently (no silent solver, no silent
  sample time, no silent delay).
- **P5 Compile once, run allocation-free**: validation and compilation happen before
  the run; the per-step path allocates nothing (tested with a counting allocator).
- **P6 Layering**: `apps/desktop` does UI only and never contains simulation math;
  `mlapp-sim` has no GUI dependency; `rustlab-stdlib` is NOT in `mlapp-sim`
  `[dependencies]` (it may be a dev-dependency for oracles).
- **P7 Backward compatibility**: diagram JSON has `schema_version` with migrations; old
  diagrams and `.slx` fixtures keep giving the same results.
- **P8 Tests first, with analytic oracles and stated tolerances**: Never assert exact
  step counts across operating systems; assert tolerances and invariants.
- **P9 Clean-room and honesty**: textbook methods only (references below); no
  MathWorks text, icons, layouts or naming; no use of the words Simulink or
  Simscape in UI strings or file names (library category: "Electrical").
  Never claim something you did not run. A draft is labelled unfinished; a
  failure is reported as a failure; the GUI is "unverified" unless a human
  checked it. The project is AI-assisted and says so in the docs.
- **P10 Small reviewable PRs with human gates between milestones**.

## 3. ARCHITECTURE
### Layers (dependencies point downward only):
- `apps/desktop`: egui editor: canvas, Properties panel, Problems panel, scopes,
  spectrum view, solver settings, examples menu
- `crates/mlapp-cli`: thin commands: check, run, codegen
- `mlapp-sim::codegen`: emits C from the compiled model
- `mlapp-sim::{engine, solver, electrical, measurements, diagnostics}`
- `mlapp-sim::model`: serde data model: BlockDiagram, Block, Line, PortKind,
  `schema_version`, spec table, SimulationConfig

### Pipeline (one public API used by GUI, CLI and codegen):
```
BlockDiagram (JSON or .slx)
  -> validate(&diagram, &config) -> Vec<Diagnostic>
  -> compile(&diagram, &config) -> Result<CompiledModel, Vec<Diagnostic>>
       (execution order, state layouts, electrical islands, MNA structure)
  -> CompiledModel::run(&mut Workspace) -> SimulationOutput {tout, signals}
  -> editor plots / CSV export / C-code oracle tests
```

### Diagnostics & Data Model
- `Diagnostic = { code, severity, message, block_sids, fix: Option<FixAction> }`.
- `Ports: PortKind { Signal (directed), Electrical (undirected terminal) }`.
- Electrical wires merge terminals into nets (union-find). Signal and electrical
  connect ONLY through bridge blocks. Every connected electrical circuit needs a
  Ground block.
- Electrical solving: companion-model MNA with the model's fixed step $h$;
  trapezoidal by default, Backward Euler for the first step and after any switching
  event; electrical blocks require a fixed-step solver (variable step + electrical
  = `E-SOLVER` for now). Switch controls and controlled sources are sampled at the
  START of each step and held. A path `sensor -> ... -> controlled source or switch`
  in the same step must pass through a state block (`UnitDelay`, `Integrator`, `Zero-Order Hold`,
  strictly proper transfer function); otherwise the engine inserts a
  one-step delay and reports warning `W-DELAY`, never silently.
- Sign convention: passive sign convention, documented.

### Allowed References (cite ONLY these):
- Dommel 1969 (electromagnetic transients)
- Ho, Ruehli, Brennan 1975 (modified nodal approach)
- Vlach and Singhal 1983 (Computer Methods for Circuit Analysis and Design)
- Dormand and Prince 1980
- Hairer, Norsett, Wanner (ODE I)
- Mohan, Undeland, Robbins (Power Electronics)

## 4. MILESTONES ROADMAP
- **M0**: Codebase audit and plan (completed)
- **M1**: Execution engine (topological order, Tarjan SCC E-ALG, pure eval, workspace) (completed, PR #29)
- **M2**: Solvers (Euler, RK4, Dormand-Prince 5(4), E-STEP/E-NAN/E-SAMPLE/E-PARAM, UnitDelay SampleTime rule, schema_version 2, editor dropdown) (completed, PR #30)
- **M3**: Sources and control blocks (spec table driven; Sawtooth, Triangle, Pulse Generator, Repeating Sequence, Clock, Sine Wave units; Transfer Function, Discrete Transfer Function, PID with anti-windup, ZOH, Relational/Logical Operator, Switch, Abs, Math Function)
- **M4**: Electrical data model and editor (PortKind, terminals, nets via union-find, block rotation; Resistor, Capacitor, Inductor, DC/AC Voltage Source, DC Current Source, Ground, Diode, Ideal Switch; Electrical library tab)
- **M5**: MNA, linear circuits (Modified Nodal Analysis, companion models, dense self-contained LU with partial pivoting, constant-matrix fast path, E-SINGULAR)
- **M6**: Nonlinear and switching (Piecewise linear Diode, Ideal Switch, Thyristor, chatter detection W-CHATTER, event localization via bisection)
- **M7**: Bridges and hybrid coupling (Voltage/Current Sensors, Controlled Sources, sampling/hold rules, W-DELAY diagnostic)
- **M8**: Measurements (RMS, Mean, Power Measurement, FFT radix-2/DFT, THD, spectrum bar plot, CSV export)
- **M9**: Error finding and developer support (validate() pre-check, Problems panel with one-click fix, mlapp-cli check, canonical diagnostics E-ALG, E-STEP, E-NAN, E-SAMPLE, E-PARAM, E-GND, E-FLOAT, E-VLOOP, E-ICUT, E-SINGULAR, E-PORT, E-MULTI, E-SOLVER, E-CHATTER, W-DELAY, W-CHATTER, W-STEP, I-SLOW)
- **M10**: Examples and docs (rc_charge, rlc_underdamped, half_wave_rectifier, buck_open_loop, buck_closed_loop_pi, rl_load_power, thd_square_wave, pid_first_order; theory docs)
- **M11**: C code generation (C99, allocation-free, static MNA assembly, host test harness, gcc/clang parity)
- **M12**: Workflow polish (Background simulation worker thread with Cancel, undo/redo, parameter sweep, inline validation text, release notes)

## 5. DEFINITION OF DONE FOR THE WHOLE PROJECT
A user can: draw a mixed signal and circuit diagram; edit every parameter with
validation; pick Euler, RK4 or variable-step ode45; run it; see all scopes and
spectra; read a Problems panel that names the block and offers a fix for every
detectable mistake; export CSV; sweep a parameter; run `mlapp-cli check` on a
model in CI; open eight worked examples; and generate C that compiles with
gcc and clang and matches the Rust solver to $10^{-9}$. Every claim in the README is
backed by a named test; CI is green on Windows, Linux and macOS; docs explain the
design, the theory and every block; the project states clearly that it is
AI-assisted, alpha, and not affiliated with MathWorks.
