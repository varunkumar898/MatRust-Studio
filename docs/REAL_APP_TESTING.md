# Real-World MATLAB App Designer (.mlapp) Testing Guide

This guide explains how to test proprietary or real-world MATLAB App Designer (`.mlapp`) files locally using MatRust Studio (`mlapp-rs`).

---

## 1. Clean-Room Legal Isolation Policy

To strictly adhere to clean-room reverse-engineering standards:
- **Zero Proprietary Files in Repository**: Proprietary MathWorks `.mlapp` files, sample apps from MATLAB installations, and licensed assets are **never** committed to this repository.
- **Synthesized Clean-Room Fixtures**: Repository test fixtures (`r2018a_sample.mlapp`, `r2020b_sample.mlapp`, `r2021a_sample.mlapp`, `r2023b_sample.mlapp`) are clean-room synthesized archives adhering to **ISO/IEC 29500-2:2021 (Open Packaging Conventions)** and public component hierarchies documented in MathWorks Help Center.
- **External Real-App Validation**: Users wishing to validate their proprietary or complex App Designer applications can do so by pointing an environment variable to an external directory on their local filesystem.

---

## 2. Running Local Real-App Validation

### Step 1: Place Your `.mlapp` Files in an External Folder
Create a directory outside this repository (for example, `C:\my_matlab_apps` or `~/matlab_apps`) and copy your `.mlapp` or `.mlappinstall` files into it:

```text
C:\my_matlab_apps\
  ├── SignalAnalyzer.mlapp
  ├── FlightDynamicsSim.mlapp
  └── ControlTuner.mlapp
```

### Step 2: Run the Opt-In Corpus Test Suite

#### Windows PowerShell:
```powershell
$env:MLAPP_REAL_APPS_DIR = "C:\my_matlab_apps"
cargo test -p mlapp-container --test container_tests -- test_opt_in_real_apps_validation --nocapture
```

#### Linux / macOS Bash:
```bash
MLAPP_REAL_APPS_DIR=/path/to/my_matlab_apps cargo test -p mlapp-container --test container_tests -- test_opt_in_real_apps_validation --nocapture
```

### Behavior:
1. **Unset Environment Variable**: If `MLAPP_REAL_APPS_DIR` is unset, the test prints:
   ```text
   SKIPPED: MLAPP_REAL_APPS_DIR unset (0 real apps tested).
   ```
   and exits cleanly without failure.
2. **Set but Empty / Missing Directory / 0 `.mlapp` Files**: If the variable is set to an empty string, a non-existent path, or a directory containing zero `.mlapp` files, the test deliberately **fails** (`panic!`) to avoid false-positive test passes.
3. **Valid Directory**: For each `.mlapp` file discovered:
   - Validates Open Packaging Conventions (OPC) headers and directory structures.
   - Enforces Zip-Slip and decompression-bomb security limits.
   - Parses XML component trees (`appdesigner/app.xml`).
   - Extracts and parses MATLAB classdef source code (`matlab/document.xml`).
   - Verifies component geometry, callbacks, and compatibility scores.

---

## 3. CLI Direct Inspection & Execution

You can also use the `mlapp` CLI directly to inspect or run real apps:

### Inspection:
```bash
cargo run -p mlapp-cli -- inspect C:\my_matlab_apps\SignalAnalyzer.mlapp
```

### Compatibility Audit:
```bash
cargo run -p mlapp-cli -- check-compat C:\my_matlab_apps\SignalAnalyzer.mlapp
```

### Execution (Headless or GUI):
```bash
cargo run -p mlapp-cli -- run C:\my_matlab_apps\SignalAnalyzer.mlapp
cargo run -p mlapp-cli -- run --gui C:\my_matlab_apps\SignalAnalyzer.mlapp
```
