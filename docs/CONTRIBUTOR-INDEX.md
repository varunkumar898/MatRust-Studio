# MatRust Studio — Contributor Documentation Index

Welcome to the documentation portal for **MatRust Studio**. This index provides a centralized navigation directory for all developer guides, architectural blueprints, roadmaps, engineering rules, and technical specifications.

---

## 1. Onboarding & Contribution

- [**Contributor Guide**](CONTRIBUTOR-GUIDE.md): Primary starting point for newcomers. Covers project vision, clean-room standards, small-change workflows, and first-time contribution checklists.
- [**Development & Build Reference**](DEVELOPMENT.md): Exact compilation, testing, formatting, linting, and verification commands across Windows, Linux, and macOS.
- [**Codebase & Repository Map**](REPOSITORY-MAP.md): Detailed directory tree and catalog of all 22 workspace crates, their roles, source files, and dependencies.
- [**Agent & Automation Protocol**](AGENT-PROTOCOL.md): Operating rules for human engineers and automated AI coding assistants, including Rule P13 for roadmap items.

---

## 2. Architecture & Design

- [**System Architecture Specification**](architecture.md): Subsystem breakdown, dependency direction rules, numerical execution pipeline, App Designer pipeline, and simulation MNA solver architecture.
- [**Simulation Engine Design**](simulation-design.md): Block diagram mathematical foundations, topological sorting, and execution flow.
- [**Numerical Solvers Specification**](solvers.md): Continuous ODE solvers (Euler, RK4, Dormand-Prince DP45), adaptive error control, and FSAL properties.
- [**Electrical Model Specification**](electrical-model.md): Ho-Ruehli-Brennan Modified Nodal Analysis (MNA), companion models, and semiconductor switching.
- [**Measurements & Spectral Analysis**](measurements.md): True RMS, mean, active/reactive power, Total Harmonic Distortion (THD), and Radix-2 FFT spectral analysis.
- [**Diagnostics & Model Checking**](diagnostics.md): Automated static diagnostics (`E-FLOAT`, `E-VLOOP`, `E-ICUT`, `E-PORT`, `E-CHATTER`).

---

## 3. Planning & Roadmaps

- [**Master Workplan**](WORKPLAN.md): Development waves (Waves 0–2), completed milestones (M1–M12), planning task protocols (`PLAN-F1` .. `PLAN-F16`), and strategic sequencing.
- [**Engineering Phases Catalog**](WORKPLAN-PHASES.md): The exhaustive item-by-item roadmap across Phases **F1 through F16**, tracking 281 catalog rows, sizing, release trains (R1..R5), and goal gates (G1..G10).
- [**Feature Status Registry**](FEATURE-REGISTRY.md): Single source-of-truth capability index mapping every capability to implementation files, exact proving tests, and verification status.
- [**Storage & Footprint Budget**](STORAGE-BUDGET.md): Artifact size tracking, release binary guardrails, and installer footprint budgets.

---

## 4. Governance, Safety & Verification

- [**Engineering Charter**](ENGINEERING-CHARTER.md): Core engineering invariants, column-major conventions, error codes, and testing standards.
- [**Safety Audit Report**](SAFETY-AUDIT.md): Workspace-wide audit of `unsafe` blocks, panic-safety in libraries, ZIP-slip mitigations, and arithmetic saturation.
- [**Verification & Test Manifest**](VERIFICATION.md): Verification methodologies, golden test corpus, and automated CI test accounting.
- [**Test Manifest**](test-manifest.txt): Verified catalog of all active tests across the workspace.
- [**Function Coverage Catalog**](COVERAGE.md): Built-in numerical function implementation status and mathematical test evidence.
- [**Backlog & Deferred Items**](BACKLOG.md): Long-term deferred items and technical debt tracker.
