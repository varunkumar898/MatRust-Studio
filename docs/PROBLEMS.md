# MatRust-Studio Problem Log

This document records issues encountered during development, their reproduction, root cause analysis, resolution, and commit SHA.

---

### Issue 1: Unused `Command` Import on Non-Windows Targets in `mlapp-matlab`
- **What Broke**: GitHub Actions CI failed on `ubuntu-latest` and `macos-latest` with `-D unused-imports` during `cargo clippy`.
- **Reproduction**: Run `cargo clippy --target x86_64-unknown-linux-gnu` or run CI on Linux/macOS.
- **Root Cause**: `use std::process::Command;` was imported at the file root in `crates/mlapp-matlab/src/detect.rs`, but was only referenced inside `probe_windows_registry()`, which is conditional on `#[cfg(target_os = "windows")]`. On Unix platforms, `Command` was unused.
- **Fix**: Moved the import into `probe_windows_registry()` so it is only compiled on Windows targets.
- **Commit SHA**: `36651d0` (merged into `main` at `bdf5211`).

---

### Issue 2: Relative Fixture Path Resolution Across Crate Workdirs
- **What Broke**: `cargo test -p mlapp-cli` and `cargo test -p mlapp-host` failed with `os error 3 (The system cannot find the path specified)` when attempting to load `fixtures/sample_models/simple_feedback.slx`.
- **Reproduction**: Run `cargo test -p mlapp-cli` or `cargo test -p mlapp-host` from the workspace root or crate directory.
- **Root Cause**: Cargo executes integration tests with the current working directory set to the crate directory (`crates/mlapp-cli/`), where `fixtures/...` does not exist directly.
- **Fix**: Implemented candidate path resolution checking `model_name`, `fixtures/sample_models/{name}.slx`, and `../../fixtures/sample_models/{name}.slx` in both CLI test helpers and `mlapp-runtime` model loaders.
- **Commit SHA**: `fc9dfcd` (merged into `main` at `ba7137f`).

---

### Issue 3: Missing `std::io::Write` Trait in Scope for `ZipWriter`
- **What Broke**: Compilation error `error[E0599]: no method named 'write_all' found for struct 'ZipWriter<W>'` when adding new integration tests in `host_tests.rs`.
- **Reproduction**: Construct an in-memory zip archive in an integration test without importing `std::io::Write`.
- **Root Cause**: `write_all()` is an extension method from the `std::io::Write` trait, which must be in scope.
- **Fix**: Added `use std::io::Write;` at the top of `crates/mlapp-host/tests/host_tests.rs`.
- **Commit SHA**: `fc9dfcd` (merged into `main` at `ba7137f`).

---

### Issue 4: Simulation UI Property Delta Deduplication
- **What Broke**: In `test_simulink_app_designer_integration`, the test failed with `Expected 'Simulation Finished' status update for gain 3.5`.
- **Reproduction**: Run multiple simulation iterations that set `app.StatusLabel.Text = 'Simulation Finished'`.
- **Root Cause**: `AppRuntime` evaluates whether component properties changed before emitting a `SetProperty` mutation (`comp.properties.get(&prop_name) != &prop_val`). Because iteration 1 already updated the label to `'Simulation Finished'`, iteration 2's identical string was not considered a mutation.
- **Fix**: Dynamically formatted the status text to include the gain parameter: `sprintf('Simulation Finished (Gain=%.1f)', app.GainSlider.Value)`, ensuring a deterministic, verifiable delta on every test iteration.
- **Commit SHA**: `fc9dfcd` (merged into `main` at `ba7137f`).

---

### Issue 5: Clippy `collapsible_match` and `approx_constant` Warnings
- **What Broke**: `cargo clippy --workspace --all-targets --all-features -- -D warnings` rejected nested `if` statements inside match arms in `host_tests.rs` and approximate float constants (`stop_time: 6.28318`) in `sim_tests.rs`.
- **Reproduction**: Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **Root Cause**: Clippy enforces idiomatic match guards (`UiMutation::PlotCall { axes, args, .. } if ... =>`) and direct use of standard math constants.
- **Fix**: Collapsed the match arm and used `std::f64::consts::TAU`.
- **Commit SHA**: `fc9dfcd` (merged into `main` at `ba7137f`).
