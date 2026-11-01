# Blockers and Human Requirements Log

Per Rule 12 and the Project Contract:
"If blocked after 3 distinct fix attempts, write the exact error, attempts, and needed human action to docs/BLOCKERS.md, then continue with other work. Never invent results."

---

## Blocker 1: MATLAB / Simulink Executable Not Found

* **Date Recorded**: 2026-11-01
* **Severity**: Blocker for Live Oracle Validation (Phase 2 Live Bridge & Phase 5/6 Live Host Execution)
* **Exact Error**:
  ```
  INFO: Could not find files for the given pattern(s) "matlab.exe".
  matlab -batch "disp(version)" : The term 'matlab' is not recognized as the name of a cmdlet, function, script file, or operable program.
  ```
* **Attempts Made**:
  1. *Attempt 1 (PATH Resolution)*: Executed `where.exe matlab` and `Get-Command matlab` in PowerShell. MATLAB executable is not present on system PATH.
  2. *Attempt 2 (Installation Directory Probe)*: Probed `C:\Program Files\MATLAB\R2025a`. Found `VersionInfo.xml` (R2025a Update 1), but `bin\win64` contains auxiliary binaries (`MATLABWindow.exe`, `MathWorksLicenseDeactivation.exe`, `MathWorksProductUninstaller.exe`) while `matlab.exe` is absent from the installation directory.
  3. *Attempt 3 (Registry & Filesystem Scan)*: Queried `HKLM:\SOFTWARE\MathWorks\MATLAB` and scanned user directories on `C:\`. No `matlab.exe` was located.
* **Needed Human Action**:
  * Ensure full MATLAB R2025a (or earlier release with Simulink) is installed and activated on the machine.
  * Add the MATLAB `bin` folder to system PATH, or set the `MATLAB_ROOT` environment variable (e.g., `MATLAB_ROOT="C:\Program Files\MATLAB\R2025a"`).

---

## Blocker 2: External Real .mlapp / .slx Archives Absent

* **Date Recorded**: 2026-11-01
* **Severity**: Blocker for Phase 3 Real App Ingestion & Phase 6 External .slx Verification
* **Exact Error**:
  ```
  MLAPP_REAL_APPS_DIR is unset.
  SLX_REAL_APPS_DIR is unset.
  No proprietary .mlapp or .slx files found in user desktop, downloads, or documents folders.
  ```
* **Attempts Made**:
  1. *Attempt 1 (Environment Variable Check)*: Queried `$env:MLAPP_REAL_APPS_DIR` and `$env:SLX_REAL_APPS_DIR`. Both are unset.
  2. *Attempt 2 (Opt-in Test Verification)*: Executed `crates/mlapp-container/tests/container_tests.rs::test_opt_in_real_apps_validation`. Correctly prints `[SKIPPED]` per clean-room specification.
  3. *Attempt 3 (Disk Search)*: Searched user profile paths for external `.mlapp` and `.slx` archives. Only clean-room synthesized repository fixture files exist in `fixtures/sample_apps/`.
* **Needed Human Action**:
  * Place real proprietary `.mlapp` files (from 3+ distinct MATLAB releases, e.g. R2018a, R2021a, R2024b) and `.slx` files in external folders outside the repository.
  * Export environment variables:
    ```powershell
    $env:MLAPP_REAL_APPS_DIR = "C:\path\to\external\real_mlapps"
    $env:SLX_REAL_APPS_DIR   = "C:\path\to\external\real_slx"
    ```

---

## Autonomous Continuation Plan

While awaiting human action on external MATLAB licenses and external proprietary files:
1. **Phase 1**: Reconcile manifest diffs across historical commits `05fdca5`, `5780ac1`, and HEAD; verify all gates; merge PR #11 into `main`.
2. **Phase 2**: Implement the expanded conformance runner with 40+ comprehensive MATLAB semantic test scripts in `tests/conformance/`.
3. **Phase 4**: Implement the `--autotest` scripting engine (`wait_ms`, `click`, `set_value`, `expect_property`, `screenshot`, `close`), wire real egui window rendering and event dispatch, execute live GUI runs, and verify frame captures.
4. **Phase 5**: Create the `crates/mlapp-matlab` crate behind `--allow-matlab` / `feature = "matlab-bridge"`, implementing the typed IPC protocol, graceful degradation, and auto-SKIPPED test harness when MATLAB is unavailable.
5. **Phase 6**: Implement `mlapp slx-inspect` with OPC ZIP and XML blockdiagram parsing, model parameter extraction, and the standalone fixed-step Simulink block diagram solver.
6. **Phase 7 & 8**: Update `docs/PROBLEMS.md`, `docs/screenshots/INDEX.md`, `docs/DECISIONS.md`, and clean up Git branches.
