# MatRust Studio Storage Budget Ledger (docs/STORAGE-BUDGET.md)

This document tracks executable binary sizes, dynamic link dependencies, and packaged installer footprints across milestones and phases to prevent binary bloat and maintain lightweight deployment.

---

## 1. Storage Budget Targets

| Component / Artifact | Target Maximum | Current Baseline | Status | Policy & Notes |
| :--- | :---: | :---: | :---: | :--- |
| **Release CLI Binary (`mlapp-cli.exe`)** | $\le 15.0\text{ MB}$ | $\approx 4.8\text{ MB}$ | PASS | Headless execution, MNA solver, code generator, and CLI diagnostic engine. |
| **Release IDE Binary (`matrust_studio.exe`)** | $\le 35.0\text{ MB}$ | $\approx 22.4\text{ MB}$ | PASS | Full egui graphical interface, block diagram canvas, and plotting tools. |
| **Release Installer Package (`.zip` / setup)** | $\le 45.0\text{ MB}$ | $\approx 28.0\text{ MB}$ | PASS | Compressed distribution including documentation assets and standard examples. |
| **Dynamic Library Footprint (`libunwind`, etc.)** | $\le 5.0\text{ MB}$ | $\approx 1.2\text{ MB}$ | PASS | Toolchain runtime libraries required for specific host platforms. |

---

## 2. Phase-by-Phase Tracking Ledger

| Phase / Milestone | Commit / Hash | CLI Size | Desktop Size | Package Size | Notes |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **M12 (Codegen & Polish)** | `ccbf804` | 4.8 MB | 22.4 MB | 28.0 MB | Baseline release build with MNA and ISO C99 codegen. |
| **R1 (v0.3: F1 + F2)** | - | *Target $\le 16\text{ MB}$* | *Target $\le 25\text{ MB}$* | *Target $\le 30\text{ MB}$* | Complete block library and professional editor layout. |
| **R2 (v0.4: F3 + F4)** | - | *Target $\le 18\text{ MB}$* | *Target $\le 28\text{ MB}$* | *Target $\le 34\text{ MB}$* | Stiff numerical solvers (Radau/BDF) and hierarchy. |
| **R3 (v0.5: F5 + F7 + F8)** | - | *Target $\le 22\text{ MB}$* | *Target $\le 32\text{ MB}$* | *Target $\le 38\text{ MB}$* | Multidomain physical networks and Data Inspector. |
| **R4 (v0.6: F6 + F9 + F10)** | - | *Target $\le 25\text{ MB}$* | *Target $\le 35\text{ MB}$* | *Target $\le 42\text{ MB}$* | State machines, optimization sweeps, and verification. |
| **R5 (v0.7: F11 + F12 + F13)** | - | *Target $\le 28\text{ MB}$* | *Target $\le 38\text{ MB}$* | *Target $\le 45\text{ MB}$* | Embedded deployment and high-performance scaling. |
