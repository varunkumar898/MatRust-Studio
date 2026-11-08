# Engineering Charter Claims & Verification Matrix

*This document maps every capability and architectural claim made in `docs/ENGINEERING-CHARTER.md` to its exact verifying test, documentation, or benchmark file.*

> **Status Note**: All simulation engine, MNA solver, block execution, diagnostic checks, CLI, and analytical oracle capabilities are verified by automated tests running in CI. The interactive graphical desktop editor is marked **"GUI: unverified"** pending manual interactive verification.

---

## 1. Core Architecture & Signal Execution Engine (M1 - M3)

| Charter Claim | Verifying Test / Artifact | Status |
|:---|:---|:---|
| Pure evaluation `eval()` without mutation | `crates/mlapp-sim/tests/sim_tests.rs::test_eval_pure_immutability` | Verified |
| Topological execution order via Kahn / Tarjan SCC | `crates/mlapp-sim/tests/sim_tests.rs::test_topological_sort_order` | Verified |
| Algebraic loop detection emitting `E-ALG` | `crates/mlapp-sim/tests/sim_tests.rs::test_algebraic_loop_detection_tarjan` | Verified |
| Zero per-step heap allocations during discrete stepping | `crates/mlapp-sim/tests/alloc_test.rs::test_zero_allocations_during_eval_and_update_discrete` | Verified |
| Numerical solvers: Euler, RK4, Dormand-Prince 5(4) | `crates/mlapp-sim/tests/solver_tests.rs` | Verified |
| Solver validation diagnostics (`E-STEP`, `E-NAN`, `E-SAMPLE`, `E-PARAM`) | `crates/mlapp-sim/tests/solver_tests.rs::test_solver_diagnostics` | Verified |
| Specification table driving block parameters, ports, and feedthrough | `crates/mlapp-sim/tests/m3_blocks_tests.rs::test_m3_spec_table_definitions` | Verified |
| Linear & nonlinear control blocks (PID with anti-windup, TransferFunction, RelOp, Switch) | `crates/mlapp-sim/tests/m3_blocks_tests.rs::test_m3_pid_analytical_and_antiwindup` | Verified |

---

## 2. Electrical Modified Nodal Analysis & Circuits (M4 - M6)

| Charter Claim | Verifying Test / Artifact | Status |
|:---|:---|:---|
| Electrical data model, PortKind (Signal vs Electrical), union-find nets | `crates/mlapp-sim/tests/m4_electrical_tests.rs::test_m4_netlist_union_find` | Verified |
| MNA matrix formulation, companion models (Backward Euler & Trapezoidal) | `crates/mlapp-sim/tests/m5_mna_tests.rs::test_rc_circuit_transient_analytic` | Verified |
| Self-contained dense LU solver with partial pivoting and row swaps | `crates/mlapp-sim/tests/m5_mna_tests.rs::test_lu_pivoting_known_matrix` | Verified |
| Singular matrix detection emitting `E-SINGULAR` | `crates/mlapp-sim/tests/m5_mna_tests.rs::test_singular_circuit_handling` | Verified |
| Piecewise-linear Diode and Ideal Switch companion stamps | `crates/mlapp-sim/tests/m6_nonlinear_tests.rs::test_half_wave_rectifier_waveform` | Verified |
| Within-step switching iteration and limit cycle chatter warning `W-CHATTER` | `crates/mlapp-sim/tests/m6_nonlinear_tests.rs::test_switch_chatter_detection_w_chatter` | Verified |
| Escalation of limit cycle chatter to `E-CHATTER` after 100 consecutive steps | `crates/mlapp-sim/tests/m6_nonlinear_tests.rs::test_switch_chatter_escalation_e_chatter` | Verified |
| Pre-factored switch topology cache ($2^{N_s} \le 1024$) with zero step allocations | `crates/mlapp-sim/tests/alloc_test.rs::test_zero_allocations_during_nonlinear_circuit_simulation_step` | Verified |

---

## 3. Hybrid Coupling & Sensors (M7)

| Charter Claim | Verifying Test / Artifact | Status |
|:---|:---|:---|
| Ideal `VoltageSensor` measuring differential node voltage with infinite impedance | `crates/mlapp-sim/tests/m7_sensors_sources_tests.rs::test_voltage_sensor_resistive_divider` | Verified |
| Ideal `CurrentSensor` measuring branch current as 0V source without series resistance | `crates/mlapp-sim/tests/m7_sensors_sources_tests.rs::test_current_sensor_series_circuit` | Verified |
| Controlled sources (`VCVS`, `VCCS`) driven by control signals | `crates/mlapp-sim/tests/m7_sensors_sources_tests.rs::test_voltage_controlled_voltage_source_gain` | Verified |
| Direct cross-domain algebraic feedback emitting `W-DELAY` with 1-step latency | `crates/mlapp-sim/tests/m7_sensors_sources_tests.rs::test_direct_algebraic_loop_triggers_w_delay` | Verified |
| Closed-loop inverting op-amp macro-model settling to exact theoretical gain | `crates/mlapp-sim/tests/m7_sensors_sources_tests.rs::test_op_amp_inverting_amplifier_closed_loop_gain` | Verified |

---

## 4. Measurements & Signal Analysis (M8)

| Charter Claim | Verifying Test / Artifact | Status |
|:---|:---|:---|
| Running `Rms` and `Mean` blocks with exact analytical windowing | `crates/mlapp-sim/tests/m8_measurements_tests.rs::test_rms_and_mean_sine_wave` | Verified |
| `PowerMeasurement` block computing P, Q1, S, PF, and DPF | `crates/mlapp-sim/tests/m8_measurements_tests.rs::test_power_measurement_resistive_inductive_load` | Verified |
| Fast Fourier Transform (Radix-2 FFT and arbitrary-N DFT) | `crates/mlapp-sim/tests/m8_measurements_tests.rs::test_fft_and_dft_sinusoid_peak` | Verified |
| Total Harmonic Distortion (`Thd`) block analytical square wave oracle | `crates/mlapp-sim/tests/m8_measurements_tests.rs::test_thd_square_wave_analytical` | Verified |
| RFC 4180 compliant CSV export with formula injection escaping | `crates/mlapp-sim/tests/m8_measurements_tests.rs::test_csv_export_rfc4180_and_injection_defense` | Verified |

---

## 5. Diagnostics, CLI, and Developer Support (M9)

| Charter Claim | Verifying Test / Artifact | Status |
|:---|:---|:---|
| Proactive topological static checking (`E-FLOAT`, `E-VLOOP`, `E-ICUT`) | `crates/mlapp-sim/tests/m9_diagnostics_tests.rs::test_m9_floating_island_and_autofix` | Verified |
| Strict port validation in `schema_version >= 4` (`E-PORT`, `E-MULTI`) | `crates/mlapp-sim/tests/m9_diagnostics_tests.rs::test_m9_strict_schema_version_4_port_errors` | Verified |
| Heuristic step size check `W-STEP` ($h > 2\tau_{min}$) | `crates/mlapp-sim/tests/m9_diagnostics_tests.rs::test_m9_w_step_heuristic_warning` | Verified |
| Automated fix engine `apply_fix()` for canonical issues | `crates/mlapp-sim/tests/m9_diagnostics_tests.rs::test_m9_autofix_engine_mutations` | Verified |
| CLI command `mlapp-cli check <file.json> [--json]` with exit codes 0/1/2 | `crates/mlapp-cli/tests/cli_tests.rs::test_mlapp_cli_check_clean_model` | Verified |

---

## 6. Worked Examples & Reference Models (M10)

| Model Name | Physical System | Verifying Test | Status |
|:---|:---|:---|:---|
| `rlc_transient.json` | 2nd-order underdamped step response | `crates/mlapp-sim/tests/examples_tests.rs::test_example_rlc_transient` | Verified |
| `rectifiers.json` | Diode rectifier with capacitive filter | `crates/mlapp-sim/tests/examples_tests.rs::test_example_rectifiers` | Verified |
| `buck_converter.json` | Closed-loop step-down DC-DC converter | `crates/mlapp-sim/tests/examples_tests.rs::test_example_buck_converter` | Verified |
| `boost_converter.json` | Switched step-up DC-DC converter | `crates/mlapp-sim/tests/examples_tests.rs::test_example_boost_converter` | Verified |
| `inverting_opamp.json` | Dominant-pole inverting amplifier | `crates/mlapp-sim/tests/examples_tests.rs::test_example_inverting_opamp` | Verified |
| `non_inverting_opamp.json` | Non-inverting operational amplifier | `crates/mlapp-sim/tests/examples_tests.rs::test_example_non_inverting_opamp` | Verified |
| `dc_motor.json` | Closed-loop electromechanical servomotor | `crates/mlapp-sim/tests/examples_tests.rs::test_example_dc_motor` | Verified |
| `voice_coil_actuator.json` | Resonant electromagnetic transducer | `crates/mlapp-sim/tests/examples_tests.rs::test_example_voice_coil_actuator` | Verified |

---

## 7. Interactive Desktop Application (Editor UI)

| Component | Functionality | Status |
|:---|:---|:---|
| Editor Canvas & Wire Routing | Interactive block placement, rotation, port snapping | GUI: unverified |
| Library Tabs | Signal, Math, Control, Electrical, Measurements | GUI: unverified |
| Scope & Spectrum Views | Real-time waveform display, FFT bar charts, autoscale | GUI: unverified |
| Problems Panel | Real-time diagnostic list, one-click fix, undo fix | GUI: unverified |
| Export Controls | CSV export button, JSON file save/load | GUI: unverified |
