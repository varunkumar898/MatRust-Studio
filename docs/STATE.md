# MatRust Studio / RustLab Project State

**Current branch:** `main`
**Repository:** `https://github.com/varunkumar898/MatRust-Studio.git`
**Current code commit:** `4ae9c687e0ea5e89d81373305a415ff648aa0c31` (PR #20 merged)
**Date:** 2026-11-03
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`)
**Local stable toolchain:** Rust 1.99.1
**Declared MSRV:** Rust 1.88.0
**Verification status:** 
- Local default, all-features, and no-default-features test runs pass at **351/351/351**.
- Local clippy (all-features, no-default-features, per-crate no-default-features) passes with 0 warnings.
- `cargo fmt --all -- --check` passes cleanly.
- `scripts/verify.ps1` runs from cargo clean and exits 0 with all verification steps passing.
- GitHub Actions CI matrix: 100% green on all 14 jobs on PR #20 (run `41584278404`) and running on `main` (run `41592329702`).

## Test counts

| Configuration | Passed | Failed | Manifest match |
| :--- | ---: | ---: | :--- |
| Default features | 351 | 0 | Exact (351 / 351) |
| `--all-features` | 351 | 0 | Exact (351 / 351) |
| `--no-default-features` | 351 | 0 | Exact (351 / 351) |

`docs/test-manifest.txt` lists all 351 tests. The CI test-count guard strictly asserts exact parity between manifest and active workspace tests.

## Recent Milestones Completed

1. **Runtime & CLI Integration of GNU Octave Execution Bridge (`crates/mlapp-runtime`, `crates/mlapp-host`, `crates/mlapp-cli`)**:
   - Integrated `mlapp-octave` as a first-class execution fallback for unsupported functions across App Designer runtime callbacks.
   - Added `AppRuntime::configure_bridges(allow_matlab: bool, allow_octave: bool)` with backward-compatible `configure_matlab_bridge`.
   - Added `allow_octave: bool` to `HostConfig` in `mlapp-host` and `--allow-octave` CLI flag in `mlapp-cli`.
   - Enhanced runtime error reporting: unsupported functions clearly suggest both `--allow-matlab` and `--allow-octave`.
   - Added unit and E2E host integration tests (`test_runtime_configure_bridges_octave_delegation`, `test_runtime_unsupported_function_hints_both_bridges`, and E2E UI mutation testing in `host_tests`).
   - Workspace test count grew from 349 to 351 tests (+2 net tests, 0 regressions).

2. **GNU Octave Subprocess Bridge (`crates/mlapp-octave`)**:
   - Clean-room out-of-process bridge communicating strictly over standard OS pipes (`stdin`/`stdout`/`stderr`) with `octave-cli`.
   - Atomic batch evaluation via `octave-cli --eval` with JSON response markers, avoiding pipe deadlocks.
   - Function-level safety allowlist denying hazardous system commands (`system`, `dos`, `delete`, `exit`, `fork`, etc.).
   - Full serialization/deserialization between `rustlab_math::Value` and JSON-lines IPC protocol.
   - Comprehensive test suite (8 tests) covering process lifecycle, mock execution, safety denials, protocol roundtrips, and differential oracle conformance when Octave is present.

2. **PyO3 Python Package (`matrust` / `crates/mlapp-py`)**:
   - Clean-room CPython extension module compiled with `abi3-py39` forward compatibility for Python 3.9+.
   - `Package` context manager for inspecting `.mlapp`/`.mlappinstall` archives, extracting components, and inspecting manifests.
   - `Session` interactive handle for evaluating scripts, dispatching UI events, and getting/setting component properties.
   - Custom typed exception hierarchy (`MlappError`, `PackageError`, `SessionError`).
   - Clean zero-warning error conversion pattern satisfying strict `-D warnings` without any `#[allow]`.
   - Complete Python package layout under `python/matrust` with PEP 484/561 type stubs (`__init__.pyi`), marker (`py.typed`), and unit test suite.
   - Tested and passing in CI across 6 platform/version combinations (Python 3.9 & 3.12 on Ubuntu, macOS, Windows).

3. **Desktop IDE Shell & Simulink Integration (`crates/matrust_studio`, `crates/mlapp-sim`)**:
   - Complete desktop IDE layout with multi-tab editor, workspace inspector, simulation canvas, and offline documentation browser.
   - Zero AI / Zero Copilot policy enforced across all source files and dependencies.
   - Acceptance test `T-ENTROPY` verified.
   - Simulink block diagram engine (`crates/mlapp-sim`) with discrete/continuous solvers and `.slx` inspection CLI (`slx-inspect`).

4. **CI Matrix & Security Hardening**:
   - 14/14 automated jobs on GitHub Actions:
     - Code Formatting (`cargo fmt --check`, cbindgen header validation)
     - MSRV Check (`cargo check` on Rust 1.88.0)
     - Security & License Checks (`cargo-deny`, `cargo-audit`, Zero-AI crate/symbol check)
     - Unsafe Code Audit (`cargo-geiger`)
     - Cargo Fuzz Targets (5 targets running 60s each in smoke run)
     - 6x Python Bindings matrix (Python 3.9 and 3.12 on Ubuntu, macOS, Windows)
     - 3x OS Build & Test matrix (Ubuntu, macOS, Windows) with Clippy, unit/integration tests, benchmarks, docs, and test count guard.

## Honest limitations

- **0 real `.mlapp` apps tested.** `MLAPP_REAL_APPS_DIR` is unset. The container opt-in test prints `SKIPPED: MLAPP_REAL_APPS_DIR unset (0 real apps tested).`
- **MATLAB Oracle:** A MATLAB R2025a tree exists at `C:\Program Files\MATLAB\R2025a`, but no runnable `matlab.exe` was found; `matlab -batch` cannot run without a host MATLAB binary. The fallback GNU Octave bridge serves as an active open-source differential oracle.
- **GUI Automation:** Native eframe window and automated `--autotest` headless engine are verified; physical Win32 mouse click simulation on remote headless runners remains guarded by synthetic event queues.
- **C ABI:** Header generation (`mlapp.h`), integer handles, null checks, and panic isolation are tested; no Miri/ASan fuzzing of raw pointers.

## Next steps

1. Tag release / milestone checkpoint.
2. Provide real `.mlapp` fixtures via `MLAPP_REAL_APPS_DIR` when available to test full third-party App Designer packages.
3. Keep CI strictly green on every push without loosening `-D warnings` or deleting tests.
