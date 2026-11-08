# Release Notes: Milestones M11 & M12 - C Code Generation, Workflow Polish, and Final Engine Completion

## 1. Executive Summary

Milestones M11 and M12 represent the completion of the core numerical simulation engine and workflow capabilities for **MatRust Studio / RustLab** (Milestones M0 through M12).

- **Milestone M11 (ISO C99 Code Generation)**:
  - Generates standalone, allocation-free (zero `malloc`, `free`, `calloc`, `realloc`) C99 code for combined signal-flow block diagrams and Modified Nodal Analysis (MNA) electrical circuits.
  - Statically bounded execution buffers for all continuous, discrete, and algebraic signals.
  - Generates an in-place dense LU linear solver with partial pivoting for MNA networks.
  - Generates header (`.h`), implementation (`.c`), and optional standalone runner (`_main.c`) exporting CSV trajectories.
  - New CLI subcommand: `mlapp-cli codegen <model.json> [--out-dir <dir>] [--name <model_name>]`.

- **Milestone M12 (Workflow Polish & Engine Finalization)**:
  - **Parameter Sweep Engine**: Multi-run parameter sensitivity engine running series of simulations across discrete block parameter variations with consolidated tabular/CSV results and signal extraction.
  - New CLI subcommand: `mlapp-cli sweep <model.json> --block <sid> --param <name> --values <v1,v2,v3> [--out-dir <dir>]`.
  - **Desktop Editor Workflow Polish**:
    - Multi-level undo/redo stack (`undo_stack`, `redo_stack`, `push_undo_state`, `undo`, `redo`) in `BlockDiagramEditor` preserving all block parameters, lines, and geometries.
    - Asynchronous simulation cancellation flag (`sim_cancel_flag`, `cancel_simulation`) enabling graceful user aborts without corrupting editor state or leaking execution threads.
  - Strict preservation of the desktop editor UI/UX layout and styling.

---

## 2. Milestone M11: ISO C99 Code Generation Architecture

### 2.1 Design Principles
1. **Zero Dynamic Allocation**: The generated C99 code contains strictly 0 calls to `malloc`, `free`, `calloc`, or `realloc`. All state structures and temporary work buffers are allocated as part of a fixed-size state struct `[model_name]_state_t` on the stack or in static memory.
2. **Deterministic Execution**: Execution steps run in $O(1)$ memory with predictable timing, suitable for bare-metal microcontrollers, DSPs, and real-time hardware-in-the-loop (HIL) simulators.
3. **MNA Matrix Assembly & Solve**: Linear circuits assemble companion conductance matrix $G$ and source vector $I$ statically. An in-place LU solver with row pivoting resolves node voltages and branch currents at each time step.
4. **ANSI/ISO C99 Compliance**: Clean, portable code with no compiler-specific extensions, fully verifiable with GCC, Clang, or MSVC.

### 2.2 CLI Usage
```bash
# Generate C99 source and header files
mlapp-cli codegen examples/models/buck_converter.json --out-dir ./generated_c --name buck_ctrl

# Output files:
#   ./generated_c/buck_ctrl.h
#   ./generated_c/buck_ctrl.c
#   ./generated_c/buck_ctrl_main.c
```

---

## 3. Milestone M12: Parameter Sweep and Workflow Polish

### 3.1 Parameter Sweep Engine
The sweep engine allows users and automated CI pipelines to perform sensitivity analysis and component sizing directly against JSON block diagram models.
```bash
# Sweep resistor values in a voltage divider
mlapp-cli sweep examples/models/divider.json --block 2 --param Resistance --values 100,500,1000,5000 --out-dir ./sweep_results
```
- Each run evaluates the system under the exact same numerical tolerances and initial conditions.
- Final values and complete signal time histories are recorded per run.
- Results are exportable to CSV and JSON formats.

### 3.2 Editor Undo/Redo & Simulation Cancellation
- `BlockDiagramEditor` tracks snapshot checkpoints before destructive edits (block deletions, parameter modifications, wire removals).
- Undo and redo methods restore exact state without re-triggering unnecessary layout recalculations.
- Simulation threads can be cooperatively cancelled via an atomic boolean flag (`Arc<AtomicBool>`), terminating lengthy loops while keeping previously simulated data accessible.

---

## 4. Verification and Test Suite

All additions are covered by dedicated automated test suites:
- `crates/mlapp-sim/tests/codegen_tests.rs`:
  - `test_codegen_signal_model_allocation_free`: Validates zero dynamic allocations in feedback control diagrams.
  - `test_codegen_circuit_mna_solver`: Validates MNA matrix stamping and LU code generation for electrical networks.
  - `test_codegen_validation_failure_on_error_diagram`: Ensures codegen rejects models with algebraic loops (`E-ALG`).
  - `test_codegen_gcc_compilation_if_available`: Compiles generated C code using GCC and asserts trajectory match.
- `crates/mlapp-sim/tests/sweep_tests.rs`:
  - `test_parameter_sweep_gain_variation`: Verifies proportional gain sweep outputs.
  - `test_parameter_sweep_resistor_divider`: Compares resistor sweep outputs against analytical voltage divider equation.
  - `test_parameter_sweep_invalid_target_error`: Verifies proper error reporting when sweeping nonexistent components.
- `crates/mlapp-cli/tests/cli_tests.rs`:
  - `test_cli_codegen_command`: CLI codegen argument parsing and file generation.
  - `test_cli_sweep_command`: CLI sweep execution and output verification.

---

## 5. Summary of Completed Milestones (M0 - M12)

| Milestone | Capability | Status |
| :--- | :--- | :--- |
| **M0** | Core Simulation Shell & Model Representation | Completed |
| **M1** | Linear ODE Solvers (RK4, Euler, Dormand-Prince RK45) | Completed |
| **M2** | Hybrid Multi-Rate & Discrete Time Systems | Completed |
| **M3** | Simulink Standard Block Library & Transfer Functions | Completed |
| **M4** | Modified Nodal Analysis (MNA) Linear Circuit Engine | Completed |
| **M5** | Bridge Interfaces, Sensors, and Controlled Sources | Completed |
| **M6** | Nonlinear Circuits, Diodes, Ideal Switches, Chatter Limits | Completed |
| **M7** | Variable-Step Solvers with Adaptive Step-Size & Zero-Crossing | Completed |
| **M8** | Measurement Blocks (RMS, Mean, Power, THD, DPF, Radix-2 FFT) | Completed |
| **M9** | Static Diagnostics Engine (26 error codes, Auto-Fix, CLI `check`) | Completed |
| **M10** | Reference Examples, Physical Oracles & Documentation Suite | Completed |
| **M11** | Zero-Allocation ISO C99 Code Generation | Completed |
| **M12** | Workflow Polish, Parameter Sweeps, Undo/Redo & Release | Completed |
