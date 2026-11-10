# MatRust Studio Contributor Guide

Welcome to **MatRust Studio**! This guide is designed to help first-time open-source contributors and experienced Rust engineers understand the codebase, set up a local development environment, find an appropriate task, write correct code and tests, and submit high-quality pull requests.

---

## 1. What MatRust Studio Is (and What It Is Not)

**MatRust Studio** is an open-source, clean-room numerical computing and simulation platform written in 100% Rust. It integrates:
1. **RustLab Numerical Engine**: A MATLAB-compatible high-performance language runtime, AST parser, tree-walking interpreter, column-major multidimensional tensor core, and linear algebra engine.
2. **App Designer Runtime (`mlapp-*`)**: A container reader, callback engine, and headless/GUI renderer for `.mlapp` applications.
3. **Multi-Domain Simulation Engine (`mlapp-sim`)**: A graphical block-diagram and physical circuit simulation engine supporting continuous ODEs, discrete multi-rate sampling, Ho-Ruehli-Brennan Modified Nodal Analysis (MNA), semiconductor switching, and ANSI/ISO C99 code generation.
4. **Desktop Studio App (`apps/desktop`)**: A cross-platform interactive desktop environment built with `egui` and `eframe`, featuring docking panels, a block diagram schematic editor, live signal scopes, code editor with diagnostics, and file tree browsing.

### What MatRust Studio Is NOT
- It is **not** a 1:1 binary replacement for commercial proprietary products.
- It does **not** decompile, reverse-engineer, or use any proprietary code, P-code, or binaries.
- It does **not** claim complete coverage of all commercial toolboxes; all implemented functions are strictly verified by automated tests against published mathematical standards, textbook oracles, or open references (such as GNU Octave or SciPy).

---

## 2. Project Maturity & Evolution

The project is actively evolving. Milestone phases M1 through M12 have established the core language, App Designer extraction, block simulation, MNA electrical networks, and C99 code generation. The current roadmap focuses on **Phases F1 through F14** (detailed in [docs/WORKPLAN-PHASES.md](file:///<repo>/docs/WORKPLAN-PHASES.md)), expanding the block editor, block catalog, hierarchical modeling, stiff solvers, multi-domain physics, and testing infrastructure.

---

## 3. High-Level Capabilities & Status

Capabilities in MatRust Studio fall into distinct maturity tiers:
- **Verified (`EXISTS-VERIFIED`)**: Implemented and backed by named automated unit/integration tests running in CI (e.g., column-major array operations, adaptive Dormand-Prince ODE45, MNA circuit solving, VCVS/VCCS transducers, parameter sweeps, ISO C99 code generation, FFT/THD analysis).
- **Partially Verified (`EXISTS-UNVERIFIED`)**: Implemented in code, but awaiting either expanded automated tests or human visual validation (e.g., interactive GUI canvas behavior, breadcrumb hierarchy navigation, undo/redo round-trip tests).
- **Planned (`TODO`)**: Explicitly planned roadmap features scheduled in Phases F1–F14 (e.g., BDF/Radau stiff solvers, statecharts, multi-domain mechanical/thermal networks, FMU export).
- **Hardware-Dependent (`NEEDS-HUMAN-HARDWARE`)**: Features requiring physical microcontrollers, robotic hardware, or external test benches (e.g., on-target embedded PIL/HIL testing). The software code generator and host mock tests run in CI; on-target claims require manual verification procedures.

---

## 4. Clean-Room Implementation & Legal Boundaries

MatRust Studio strictly enforces a clean-room development methodology:
1. **Permitted Information Sources**:
   - Publicly available mathematical definitions, textbook formulas, and published academic papers.
   - Public API documentation and behavioral specifications from open documentation.
   - Black-box behavioral parity verified against open-source tools (GNU Octave, SciPy, NumPy). When citing open-source behavior, explicitly state the tool and version (e.g., `GNU Octave 8.4.0`); Octave behavior must not be conflated with MATLAB behavior.
2. **Strictly Prohibited**:
   - No decompilation, disassembly, or reverse engineering of proprietary binaries, license daemons, or P-code.
   - No copying of GPL code (MatRust Studio uses MIT OR Apache-2.0).
   - No copying of proprietary source files, sample models, or copyrighted icons/assets.
3. **Generic Naming Standards**:
   - All user-facing UI labels, error messages, and documentation must use generic technical descriptors (e.g., *state machine editor*, *block diagram editor*, *model diagnostics*).

---

## 5. Development Prerequisites & Environment

### Supported Platforms
- **Windows**: x86_64 MSVC and ARM64 (`aarch64-pc-windows-gnullvm`).
  - *Note on Windows ARM64 gnullvm*: Run via `cargo run --release -p matrust_studio` or ensure `libunwind.dll` is on your PATH (see [docs/DEVELOPMENT.md](file:///<repo>/docs/DEVELOPMENT.md)).
- **Linux**: x86_64 and AArch64 (Ubuntu 22.04+ or modern glibc).
- **macOS**: Apple Silicon (`aarch64-apple-darwin`) and Intel (`x86_64-apple-darwin`).

### Required Toolchain
- **Minimum Supported Rust Version (MSRV)**: **`1.88.0`** (declared in workspace `Cargo.toml`).
- **Cargo Tools**: `clippy`, `rustfmt`.
- Optional: Python 3.9+ (for documentation and verification scripts).

---

## 6. Build, Test, and Verification Quickstart

### 6.1 Checking Your Toolchain
```bash
rustc --version
cargo --version
```
Ensure your compiler satisfies MSRV $\ge 1.88.0$.

### 6.2 Building the Workspace
```bash
# Debug build of the entire workspace
cargo build --workspace

# Optimized release build of the desktop application
cargo build --release -p matrust_studio
```

### 6.3 Running Tests
```bash
# Run all workspace unit and integration tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p mlapp-sim

# Run a specific named test with output
cargo test -p mlapp-sim --test solver_tests test_dormand_prince_tolerance_compliance -- --nocapture
```

### 6.4 Code Quality & Formatting
```bash
# Format check
cargo fmt --all -- --check

# Clippy linter with zero-warning tolerance
cargo clippy --workspace --all-targets -- -D warnings
```

### 6.5 Full Platform Verification Script
On Windows (PowerShell):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```
On Linux/macOS:
```bash
bash scripts/verify.sh
```

---

## 7. How to Choose a Task

1. **Check the Issue Tracker**: Review issues labeled by component or domain.
2. **Review the Roadmap**: Consult [docs/WORKPLAN-PHASES.md](file:///<repo>/docs/WORKPLAN-PHASES.md) and [docs/FEATURE-REGISTRY.md](file:///<repo>/docs/FEATURE-REGISTRY.md) for current phase priorities.
3. **Scope Discipline**: Always propose or claim a task before writing code. If a planned roadmap item is sized **XL**, it must first undergo a planning breakdown (`PLAN-F<n>`) with an Architecture Decision Record (ADR) before implementation.
4. **Good First Contributions**:
   - Adding missing unit tests for existing built-in functions.
   - Expanding docstrings and generating function documentation in `docs/functions/`.
   - Adding analytic oracle tests for physical components or linear algebra edge cases.
   - Refactoring duplicated test helpers or cleaning compiler lints.

---

## 8. The Small-Change Workflow

Every change in MatRust Studio follows a disciplined lifecycle:

```mermaid
flowchart LR
    A["1. Reproduce / Specify"] --> B["2. Write Test First"]
    B --> C["3. Implement Small (~150 LOC)"]
    C --> D["4. Run Focused Test"]
    D --> E["5. Quality Gates (Fmt + Clippy)"]
    E --> F["6. Update Docs & Registry"]
    F --> G["7. Submit Focused PR"]
```

1. **Reproduce / Specify**: Clearly state the bug, numerical requirement, or specification.
2. **Write Focused Test First**: Create a golden test or failing regression test in the appropriate test suite (e.g., `crates/mlapp-sim/tests/`).
3. **Implement in Small Steps**: Limit changes to small, cohesive increments ($\sim 150$ lines of change where feasible).
4. **Verify Locally**: Run the specific test using `cargo test -p <crate> --test <suite> <test_name>`.
5. **Run Quality Gates**: Run formatting and clippy (`cargo clippy --workspace --all-targets -- -D warnings`).
6. **Update Documentation**: Update [docs/FEATURE-REGISTRY.md](file:///<repo>/docs/FEATURE-REGISTRY.md) and `docs/test-manifest.txt` if tests were added.
7. **Submit PR**: Open a focused pull request against `main`.

---

## 9. Extending Specific Subsystems

### 9.1 Adding a Numerical Built-in Function
- **Crate**: `crates/rustlab-stdlib/src/`
- Implement function logic under the appropriate module (e.g., `elfun.rs`, `linalg.rs`, `signal.rs`).
- Register the function in `crates/rustlab-stdlib/src/lib.rs` and its help string in `help.rs`.
- Add golden tests comparing outputs against analytical oracles and edge cases (`NaN`, `Inf`, empty matrices, dimensional broadcasting).
- Add Markdown documentation in `docs/functions/<name>.md`.

### 9.2 Adding a Simulation Block
- **Crate**: `crates/mlapp-sim/`
- Define block variant in `model.rs` (`BlockType` enum).
- Define specification, parameters, and input/output port counts in `spec.rs`.
- Implement execution semantics in `engine.rs` (or `electrical.rs` if an MNA physical component).
- If supported by code generation, add C99 template in `codegen.rs`.
- Add integration tests in `crates/mlapp-sim/tests/` verifying time-domain response against an analytical oracle.

### 9.3 Modifying the Desktop UI
- **Crate**: `apps/desktop/`
- All UI logic resides under `apps/desktop/src/` (`block_diagram_editor.rs`, `scope_view.rs`, `docking.rs`, etc.).
- UI interactions must be testable headlessly via automated logic tests or `egui` harness tests.
- UI visual changes must be flagged as `GUI: UNVERIFIED` until human visual inspection by the project owner.

---

## 10. Engineering Standards & Code Invariants

- **No Undocumented `unsafe`**: Any `unsafe` block must include a written safety justification comment `// SAFETY: ...`.
- **No Uncontrolled Panics**: Library crates (`crates/*`) must not use `unwrap()`, `expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in production paths. Return structured `Result<T, SimError>` or `Result<T, RustLabError>`.
- **Column-Major Array Ordering**: Multidimensional tensor arrays are strictly column-major (Fortran order), 1-based indexed at the script interface, and 0-based in internal Rust vectors.
- **Determinism**: Simulation runs and mathematical algorithms must be strictly deterministic given the same inputs and seeds.
- **Error IDs**: Standardized error IDs use `MATLAB:` prefix when replicating public standard errors, or `RUSTLAB:` / `SIM:` for internal errors. Every error ID must be cataloged in `docs/spec/error-catalog.md`.

---

## 11. Pull Request Guidelines

1. **Single-Purpose PRs**: One roadmap item or bug fix per pull request. Avoid mixing refactorings with feature work.
2. **Zero Warnings**: CI enforces zero warnings across all targets (`-D warnings`).
3. **Tests Never Decrease**: The total number of passing tests in `docs/test-manifest.txt` must never decrease. If a test is renamed or replaced, document the exact reason.
4. **Privacy & Path Hygiene**: Never include personal machine paths (e.g., `C:\Users\...` or `/home/...`), internal tokens, or local secrets in code or documentation. Use `<repo>` and `<user-home>` placeholders.
5. **No Force-Pushes to Shared Branches**: Keep git history clean and traceable.

---

## 12. First-Time Contributor Checklist

- [ ] Read this guide and [docs/ENGINEERING-CHARTER.md](file:///<repo>/docs/ENGINEERING-CHARTER.md).
- [ ] Confirmed compiler meets MSRV (`rustc --version` $\ge 1.88.0$).
- [ ] Built the workspace cleanly (`cargo build --workspace`).
- [ ] Created a new branch from `main` (`git checkout -b feat/my-change`).
- [ ] Wrote automated unit/integration tests covering new behavior.
- [ ] Ran `cargo fmt --all -- --check`.
- [ ] Ran `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Ran full test suite (`cargo test --workspace`).
- [ ] Verified zero personal machine paths in added files.
- [ ] Updated documentation and [docs/FEATURE-REGISTRY.md](file:///<repo>/docs/FEATURE-REGISTRY.md).
