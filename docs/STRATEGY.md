# MatRust Studio Strategic Positioning & Claims Policy (docs/STRATEGY.md)

This document defines the strategic differentiation, competitive positioning, explicit non-goals, and governance policies for MatRust Studio relative to MATLAB, GNU Octave, and Julia.

---

## 1. Where We Compete

MatRust Studio does not attempt to clone the entirety of legacy scientific monoliths. Instead, it competes decisively on specific architectural, operational, and licensing dimensions:

### 1.1 Tiny Native Binary and Fast Start-Up
- **Instantaneous Startup**: Launches to an interactive prompt or workspace editor in sub-100ms wall-clock time, avoiding the multi-second or multi-minute startup latency of JVM-based, electron-based, or JIT-compiling environments.
- **Minimal Disk Footprint**: Shipped as a self-contained native executable ($\sim 10$ MB to $25$ MB release binary), compared to $20+$ GB installations for commercial engineering suites or multi-gigabyte containerized environments.
- **Zero Heavy Runtime Dependencies**: Requires no Java Virtual Machine, no .NET runtime, and no external runtime installers for base operation.

### 1.2 MathWorks File Compatibility Without MATLAB (.mlapp and .slx)
- **Native App Designer Runtime**: Directly parses, interprets, and renders `.mlapp` XML archives and GUI layouts through native Rust components without requiring a MATLAB license or MATLAB Runtime installation.
- **Clean-Room Simulink Model Support**: Reads, inspects, and executes continuous/discrete `.slx` block diagram models via a high-performance clean-room simulation engine (`crates/mlapp-sim`).
- **Commercial Freedom**: Enables engineering organizations to distribute, run, and execute proprietary engineering models and workflows on edge nodes, CI pipelines, and bare-metal environments without per-seat licensing barriers.

### 1.3 Model Checker with Automated Fixes
- **Static Diagnostic Pre-Flight**: Ahead-of-time topological and domain analysis identifying critical modeling defects before simulation:
  - Algebraic loops (`E-ALG`) with path isolation.
  - Ungrounded floating electrical nodes (`E-FLOAT`).
  - Parallel ideal voltage source loops (`E-VLOOP`).
  - Series inductor/current source cut-sets (`E-ICUT`).
- **One-Click Automated Fixes**: Interactive `apply_fix` engine that programmatically resolves common modeling errors (e.g., inserting high-impedance grounding paths, small stabilization inductors, or algebraic delay elements) with user confirmation.

### 1.4 Free, Standalone C Code Generation
- **ISO C99 Allocation-Free Output**: Translates continuous-time control blocks, state-space systems, and Modified Nodal Analysis (MNA) circuit networks directly into standalone, allocation-free ANSI/ISO C99 source code.
- **Target Hardware Agnostic**: Generated code requires zero runtime libraries, uses static memory layouts, and compiles cleanly for microcontrollers (ARM Cortex-M, RISC-V, ESP32) as well as real-time simulation targets without proprietary code-generator license fees.

### 1.5 Unified Polyglot Workspace
- **Multi-Engine Notebook & Cell Execution**: Execute native RustLab algorithms, Python scripts, Julia models, and Octave/MATLAB code within a single unified workspace document.
- **Shared Variables Without Transposition**: Inter-engine variable exchange preserves native column-major (Fortran order) memory layouts for dense 2D and N-D arrays, eliminating costly array transposition passes.
- **Process Isolation**: External engines operate as sandboxed subprocesses communicating over high-speed binary IPC, ensuring a crash or infinite loop in a third-party script cannot destabilize the MatRust Studio environment.

### 1.6 Reproducible, Deterministic Runs
- **Deterministic Numerical Trajectories**: Explicitly managed pseudorandom number generators (PRNG) and IEEE 754 floating-point guarantees ensure bit-identical or epsilon-bounded results across Windows, Linux, and macOS platforms.
- **Monotonic Test Rigor**: Codebase integrity governed by a strictly monotonic test manifest (`docs/test-manifest.txt`), preventing regression of verified analytical behaviors.

### 1.7 Permissive Dual License
- **Open Commercial Core**: Licensed under dual MIT OR Apache-2.0 terms, permitting unencumbered commercial embedding, modification, proprietary extension, and redistribution.
- **Clean-Room Intellectual Property**: Developed strictly under clean-room engineering rules (P3, P14) from first principles, published literature, and open specifications.

---

## 2. Explicit Non-Goals

To maintain high development velocity, architectural clarity, and rock-solid reliability, the following areas are designated as explicit non-goals:

1. **No Native Julia Interpreter**: MatRust Studio will never construct an in-tree Julia language interpreter or compile Julia source code directly. Julia integration is provided strictly via an external-process subprocess bridge (`crates/mlapp-julia`) utilizing Julia's official distribution.
2. **No Shallow Emulation of Toolbox Breadth**: We do not attempt to duplicate the thousands of niche domain toolboxes available in commercial suites. We focus on world-class core numerical math, dynamic simulation, control systems, and clean-room file interoperability, relying on polyglot bridges (Python, Julia, Octave) when specialized domain libraries are required.
3. **No Claim of Unqualified 100% Replacement**: MatRust Studio is a clean-room numerical computing platform, not a drop-in byte-for-byte clone of MATLAB or GNU Octave. Compatibility is rigorously bounded by automated differential tests; untested compatibility is never claimed.
4. **No Performance Claim Without a Reproducible Benchmark**: We never publish comparative performance assertions (e.g., "10x faster than MATLAB") without a published, reproducible benchmark script, hardware specification, and statistical median/spread recorded in `docs/CLAIMS.md`.

---

## 3. Public Claims Policy

All public-facing communications—including `README.md`, documentation, marketing materials, and release notes—are strictly bound by the following governance principles:

1. **1:1 Mapping to Test Evidence**: Every capability claim must cite an active, passing automated regression test in `docs/test-manifest.txt` or an Architecture Decision Record.
2. **Statistical Rigor for Performance**: Every performance metric must specify:
   - Median execution time and spread (variance/range) over $\ge 5$ warm iterations.
   - Specific commit SHA and release profile (`--release`).
   - Standardized hardware environment (CPU model, core count, memory, OS) using `<user-home>` and `<repo>` path sanitization.
3. **Status Integrity**:
   - Capabilities undergoing implementation are marked `TODO`.
   - Automated verified capabilities are marked `EXISTS-VERIFIED`.
   - Features implemented in desktop UI without automated headless tests are explicitly marked `EXISTS-UNVERIFIED` or `GUI: UNVERIFIED`.
   - Unsupported or out-of-scope behaviors are documented transparently in `docs/compatibility/`.
