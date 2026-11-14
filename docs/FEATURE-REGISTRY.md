# MatRust Studio Feature Registry (docs/FEATURE-REGISTRY.md)

This registry records all merged capabilities, user-facing blocks, and tool modules, tracking automated verification status and human GUI confirmation.

---

## 1. Registry Status Legend
- `VERIFIED`: Automated test suite passes and verified in code.
- `GUI: UNVERIFIED`: Visual rendering and graphical interaction pending human interactive confirmation.
- `HUMAN-VERIFIED`: Formally confirmed by user visual inspection.

---

## 2. Feature & Component Registry

| Feature / Block ID | Category | Implementation Source | Test Evidence | GUI Status |
| :--- | :--- | :--- | :--- | :---: |
| **Sum, Gain, Integrator, UnitDelay** | Continuous / Math | `crates/mlapp-sim/src/eval.rs` | `test_simulation_first_order_step_response` | `GUI: UNVERIFIED` |
| **TransferFunction, DiscreteTransferFunction** | Control Blocks | `crates/mlapp-sim/src/blocks.rs` | `test_m3_transfer_function_analytical_step` | `GUI: UNVERIFIED` |
| **PID (Continuous & Discrete)** | Control Blocks | `crates/mlapp-sim/src/blocks.rs` | `test_m3_pid_analytical_and_antiwindup` | `GUI: UNVERIFIED` |
| **Switch, RelationalOperator, LogicalOperator** | Discontinuous | `crates/mlapp-sim/src/eval.rs` | `test_m3_control_eval_and_feedthrough` | `GUI: UNVERIFIED` |
| **ZeroOrderHold, RateTransition** | Discrete | `crates/mlapp-sim/src/blocks.rs` | `test_unit_delay_sample_time_under_dp` | `GUI: UNVERIFIED` |
| **Resistor, Capacitor, Inductor** | Electrical Linear | `crates/mlapp-sim/src/electrical.rs` | `test_rlc_transient_underdamped_oracle` | `GUI: UNVERIFIED` |
| **DcVoltageSource, AcVoltageSource, CurrentSource**| Electrical Sources | `crates/mlapp-sim/src/electrical.rs` | `test_resistive_divider_dc_solution` | `GUI: UNVERIFIED` |
| **Ground (Node 0)** | Electrical Reference | `crates/mlapp-sim/src/electrical.rs` | `test_m4_electrical_library_and_palette_blocks` | `GUI: UNVERIFIED` |
| **Diode, IdealSwitch** | Electrical Switching | `crates/mlapp-sim/src/nonlinear.rs` | `test_buck_converter_freewheeling_diode_and_switch_oracle` | `GUI: UNVERIFIED` |
| **VoltageSensor, CurrentSensor** | Physical Transducers | `crates/mlapp-sim/src/electrical.rs` | `test_voltage_sensor_resistive_divider` | `GUI: UNVERIFIED` |
| **VCVS, VCCS** | Controlled Sources | `crates/mlapp-sim/src/electrical.rs` | `test_voltage_controlled_voltage_source_gain` | `GUI: UNVERIFIED` |
| **Rms, Mean, PowerMeasurement, Thd** | Measurements | `crates/mlapp-sim/src/measurements.rs` | `test_thd_pure_sine_and_distorted` | `GUI: UNVERIFIED` |
| **Spectrum Analysis (FFT/DFT)** | Signal Analysis | `crates/mlapp-sim/src/measurements.rs` | `test_spectrum_fft_and_dft_and_window_functions` | `GUI: UNVERIFIED` |
| **Topological Diagnostics (`E-FLOAT`, etc.)** | Static Verification | `crates/mlapp-sim/src/diagnostics.rs` | `test_m9_floating_island_and_autofix` | `GUI: UNVERIFIED` |
| **Autofix Engine (`apply_fix`)** | Model Repairs | `crates/mlapp-sim/src/diagnostics.rs` | `test_m9_floating_island_and_autofix` | `GUI: UNVERIFIED` |
| **ISO C99 Code Generator** | Deployment | `crates/mlapp-sim/src/codegen.rs` | `test_codegen_circuit_mna_solver` | `GUI: UNVERIFIED` |
| **Parameter Sweep Engine** | Design Exploration | `crates/mlapp-sim/src/sweep.rs` | `test_parameter_sweep_gain_variation` | `GUI: UNVERIFIED` |
| **Multi-Level Undo/Redo** | Editor Polish | `apps/desktop/src/block_diagram_editor.rs` | `test_block_diagram_duplicate` | `GUI: UNVERIFIED` |
