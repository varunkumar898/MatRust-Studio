# MatRust Studio — Development & Verification Guide

This document provides exact, verified commands and operational instructions for building, running, testing, linting, and validating the MatRust Studio repository.

---

## 1. Toolchain & Environment Verification

### 1.1 Minimum Supported Rust Version (MSRV)
The workspace declares MSRV **`1.88.0`** in `Cargo.toml`. To verify your installed compiler:

```bash
rustc -vV
cargo --version
```

### 1.2 Required Components
Ensure `rustfmt` and `clippy` are installed for your active toolchain:

```bash
rustup component add rustfmt clippy
```

---

## 2. Compilation & Execution Commands

### 2.1 Workspace Builds
Compile the entire workspace (all 22 crates):

```bash
# Debug build (fast compile, full debug assertions)
cargo build --workspace

# Release build (heavily optimized, LTO enabled)
cargo build --release --workspace
```
*Required*: Run before submitting pull requests to ensure all workspace members build cleanly.

### 2.2 Package-Specific Builds
To build a specific subsystem without compiling unrelated crates:

```bash
# Build the simulation engine
cargo build -p mlapp-sim

# Build the numerical standard library
cargo build -p rustlab-stdlib

# Build the desktop GUI application
cargo build -p matrust_studio
```

### 2.3 Running Applications & CLIs

#### Desktop Studio Application
```bash
# Launch MatRust Studio Desktop GUI
cargo run --release -p matrust_studio
```
*Platform Note (Windows ARM64 gnullvm)*: If running the release binary directly outside of Cargo, ensure `libunwind.dll` from the toolchain binary directory is accessible on the system `PATH`.

#### RustLab Numerical REPL CLI
```bash
# Launch interactive numerical computing shell
cargo run -p rustlab-cli
```

#### App Designer & Simulation CLI
```bash
# Run headless simulation CLI
cargo run -p mlapp-cli -- --help

# Run model diagnostic check
cargo run -p mlapp-cli -- check <path-to-model.json>
```

---

## 3. Testing Workflows

### 3.1 Running All Tests
```bash
# Run all unit tests, integration tests, and doc tests across the workspace
cargo test --workspace
```

### 3.2 Running Crate-Specific Tests
```bash
# Test the simulation engine
cargo test -p mlapp-sim

# Test the numerical standard library
cargo test -p rustlab-stdlib

# Test the desktop UI logic
cargo test -p matrust_studio
```

### 3.3 Running a Specific Named Test
To run a single test function with standard output visible:

```bash
# Run Dormand-Prince adaptive ODE solver test
cargo test -p mlapp-sim --test solver_tests test_dormand_prince_tolerance_compliance -- --nocapture

# Run MNA electrical diode rectifier test
cargo test -p mlapp-sim --test m6_nonlinear_tests test_half_wave_rectifier_forward_drop -- --nocapture

# Run Block Diagram Editor alignment test
cargo test -p matrust_studio --lib test_f1_block_alignment_operations -- --nocapture
```

### 3.4 Feature-Gate Test Suites
MatRust Studio tests all combinations of feature configurations:

```bash
# 1. Workspace default features
cargo test --workspace

# 2. Workspace all features enabled
cargo test --workspace --all-features

# 3. Workspace with default features disabled
cargo test --workspace --no-default-features

# 4. Individual crates tested strictly without default features
cargo test -p rustlab-stdlib --no-default-features
cargo test -p rustlab-math --no-default-features
cargo test -p rustlab-cli --no-default-features
cargo test -p matrust_studio --no-default-features
```

---

## 4. Linting & Code Hygiene

### 4.1 Formatting
Check formatting compliance:
```bash
cargo fmt --all -- --check
```
Auto-format the codebase:
```bash
cargo fmt --all
```

### 4.2 Clippy Linter
All code must compile with zero clippy warnings (`-D warnings`):

```bash
# Standard workspace clippy
cargo clippy --workspace --all-targets -- -D warnings

# Clippy with all features
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Clippy with no default features
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
```

---

## 5. Automated Verification Scripts

The repository includes end-to-end platform verification scripts that run formatting, clippy across all feature combinations, full test suites, test manifest validation, and clean-working-tree audits.

### Windows (PowerShell)
```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify.ps1
```

### Linux / macOS (Bash)
```bash
bash scripts/verify.sh
```

---

## 6. Test Manifest & Documentation Audits

### 6.1 Test Manifest Integrity Check
The file `docs/test-manifest.txt` records all active tests. To verify no tests have disappeared:

```powershell
# List current tests and verify against manifest
$current_tests = (cargo test --workspace -- --list | Select-String "^\w.*: test$") | ForEach-Object { $_.Line.Trim() }
$manifest_tests = (Get-Content docs/test-manifest.txt | Select-String ": test$") | ForEach-Object { $_.Line.Trim() }

$missing = @()
foreach ($t in $manifest_tests) {
    if ($current_tests -notcontains $t) {
        $missing += $t
    }
}
if ($missing.Count -gt 0) {
    Write-Error "Test manifest failure! Missing tests: $($missing -join ', ')"
} else {
    Write-Host "Manifest check passed: all $($manifest_tests.Count) tests present."
}
```

### 6.2 Documentation & Privacy Path-Hygiene Check
Run the automated documentation validator to ensure all documentation files contain zero personal path leaks (`<repo>` and `<user-home>` placeholders used exclusively):

```bash
python scripts/check_docs.py
```
*Pass Criteria*: Zero personal path hits and all documented built-in functions verified.
