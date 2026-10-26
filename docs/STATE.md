# RustLab Project State (docs/STATE.md)

**Current Milestone:** G0.1 (Review Closeout & Owed Items Verification)
**Root Path:** `C:\Users\varun\Desktop\MATSTUDIO`
**Date:** 2026-10-26
**Repository Commit:** `HEAD` (tag `g0.1`)
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`), Rust 1.99.1 stable
**Verification Status:** All gates passing (`scripts/verify.ps1`, log: `docs/verify-logs/2026-10-26-g0.1.log`)
**Total Tests:**
- Default features: **196** passed
- All features: **196** passed
- No-default features: **195** passed (`test_a2_both_native_and_faer_eig_paths_match`, `test_a2_golden_eig_rotation_complex_conjugate`, and `test_a2_symmetric_eig_routes_to_faer_and_canonicalizes` replaced by `test_a2_faer_feature_off_returns_clear_error` and `test_a2_native_eig_path_without_faer`)

> **Note on historical paths:** Paths recorded before milestone M0 refer to the old repository location (`C:\Users\varun\.antigravity\rustlab`). Live configuration, RE targets, and project roots are relocated to `C:\Users\varun\Desktop\MATSTUDIO`.

---

## 1. Crate Hierarchy & Layout

| Crate | Responsibility | Dependencies | Feature Flags |
| :--- | :--- | :--- | :--- |
| `crates/rustlab-parse` | Lexer, AST definitions with `Span`, recursive-descent parser | `thiserror` | None |
| `crates/rustlab-math` | Column-major `DenseArray`, `Value` enum, CSC sparse matrix, integer classes | `rustlab-parse`, `matrixmultiply`, `num-complex`, `num-traits` | `matrixmultiply` |
| `crates/rustlab-eval` | AST evaluator, runtime `Environment`, closure handles | `rustlab-parse`, `rustlab-math`, `thiserror` | None |
| `crates/rustlab-stdlib` | Standard library builtins (elementary math, linalg, FFT, string, IO) | `rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-matio`, `rustlab-data` (opt) | `faer`, `data` |
| `crates/rustlab-matio` | Level 5 MAT-file read/write | `rustlab-math`, `flate2` | None |
| `crates/rustlab-data` | Summary statistics, windowing, rankings, CSV matrix I/O | `rustlab-parse`, `rustlab-math`, `rustlab-eval`, `csv = "1.4"`, `ryu = "1.0"` | None |
| `crates/rustlab-cli` | REPL, line-editor, diagnostic error reporting, benchmark suite | `rustlab-eval`, `rustlab-stdlib`, `rustyline`, `clap` | `faer`, `data` |

---

## 2. Binary Hashes & REA Target

- `target/release/rustlab.exe`:
  - Size: 5,400,064 bytes
  - SHA-256: `78a31a97b1e73696d5ec2ea434af9878da543bda43d4f058cf5c21a6eb42c3c7`
- REA Target: `re/targets/rustlab.exe` (SHA-256 matches `target/release/rustlab.exe`)
- REA Project Root: `C:\Users\varun\Desktop\MATSTUDIO\re\projects`
- REA MCP session integration verified via native MCP calls (`open_binary`, `binary_session`, `close_binary`).

---

## 3. Milestones Completed

- **Milestone 7**: Baseline foundation (68 tests)
- **Milestone A1**: Language, closures, string library (91 tests)
- **Milestone A2**: Native symmetric eig, svd, expm, fft (96 tests)
- **Milestone A3**: Integer classes, CSC sparse, N-D tensor ops (111 tests)
- **Milestone A3.1 - A3.1.4**: Corrective and integrity passes, zero unwrap policy, test manifest check (153 tests)
- **Milestone B2.1 - B2.1.1**: Clean-room statistics, RFC 4180 CSV matrix I/O, display goldens, MATLAB percentile rule, singular matrix warning + Inf (196 tests)
- **Milestone G0**: B2.1.1 closeout, performance contracts, error ID standardization (`RUSTLAB:`), omitnan core migration (196 tests)
- **Milestone M0**: Relocate project to Desktop\MATSTUDIO, verify 196/196/195 tests from cargo clean, configure re\targets and re\projects, record binary hash.
- **Milestone G0.1**: Complete owed items review, docs/OWED.md, REA R-check on release binary, REPL transcript log, release benchmark triplet, gate verification. Next milestone is G1.
