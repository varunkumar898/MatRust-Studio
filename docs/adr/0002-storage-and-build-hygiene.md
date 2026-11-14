# ADR 0002: Storage Budgets, Build Hygiene, and Artifact Size Guardrails

- **Status:** Accepted
- **Date:** 2026-11-14
- **Deciders:** Project Owner, AI Engineering Team

## Context

MatRust Studio aims to deliver high-performance numerical computing and simulation with an agile, accessible footprint. Without strict automated budgets, rust workspaces can accumulate bloated binary dependencies, uncompressed binary assets, and oversized release executables. In addition, local build target directories can rapidly consume disk space and degrade development velocity.

## Decision

We establish strict, machine-enforced storage budgets and build-hygiene gates:

1. **Repository Tracked Size**: The entire git-tracked repository footprint must remain $\le 50\text{ MB}$.
2. **Individual Tracked File Size**: No individual tracked file may exceed $500\text{ KB}$. Large screenshots or binary fixtures must be compressed or hosted externally.
3. **Release Executable Size**: The release binary (`target/release/matrust_studio.exe`) must not exceed measured baseline plus 20% headroom ($\le 13.0\text{ MB}$, based on measured 10.20 MB baseline).
4. **Installer Packaging**: Currently, there is no installer. When an installer is introduced, its distribution package size must not exceed $100\text{ MB}$.
5. **Target Directory Alarm**: A shared local target directory exceeding $20\text{ GB}$ triggers an alarm recommending `cargo clean` and cache pruning.
6. **Free Disk Space Floor**: Available disk space on the build filesystem must not drop below $20\text{ GB}$.
7. **Automated Verification**: The automated audit scripts `scripts/storage-report.ps1` and `scripts/storage-report.sh` are integrated into pre-release and Wave 2 quality gates.

## Consequences

- Core crates remain free of heavy GUI or hardware bindings.
- Heavy optional dependencies (e.g., PyO3 C-API, FMU co-simulation, GPU kernels) must remain behind optional cargo feature flags.
- Any regression in release binary size or repository bloat is halted immediately at verification.
