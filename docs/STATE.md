# RustLab Project State (docs/STATE.md)

**Current Milestone:** G0 (Closeout of B2.1.1)
**Repository HEAD:** `230ade5` (tag `b2.1.1`)
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`), Rust 1.99.1 stable
**Verification Status:** All gates passing (`scripts/verify.ps1`)
**Total Tests:**
- Default features: **196** passed
- All features: **196** passed
- No-default features: **195** passed (`test_a2_both_native_and_faer_eig_paths_match`, `test_a2_golden_eig_rotation_complex_conjugate`, and `test_a2_symmetric_eig_routes_to_faer_and_canonicalizes` replaced by `test_a2_faer_feature_off_returns_clear_error` and `test_a2_native_eig_path_without_faer`)

---

## 1. Crate Hierarchy & Layout

| Crate | Responsibility | Dependencies | Feature Flags |
| :--- | :--- | :--- | :--- |
| `crates/rustlab-parse` | Lexer, AST definitions with `Span`, recursive-descent parser | `num-traits` | None |
| `crates/rustlab-math` | Column-major `DenseArray`, `Value` enum, CSC sparse matrix, integer classes | `rustlab-parse`, `matrixmultiply`, `num-complex` | None |
| `crates/rustlab-eval` | AST evaluator, runtime `Environment`, closure handles | `rustlab-parse`, `rustlab-math` | None |
| `crates/rustlab-stdlib` | Standard library builtins (elementary math, linalg, FFT, string, IO) | `rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-matio`, `rustlab-data` (opt) | `faer`, `data` |
| `crates/rustlab-matio` | Level 5 MAT-file read/write | `rustlab-math`, `flate2` | None |
| `crates/rustlab-data` | Summary statistics, windowing, rankings, CSV matrix I/O | `rustlab-parse`, `rustlab-math`, `rustlab-eval`, `csv = "1.4.0"` | None |
| `crates/rustlab-cli` | REPL, line-editor, diagnostic error reporting, benchmark suite | `rustlab-eval`, `rustlab-stdlib`, `rustyline`, `clap` | `faer`, `data` |

---

## 2. Binary Hashes & REA Target

- `target/release/rustlab.exe`:
  - Size: 5,400,064 bytes
  - SHA-256: `683ea9cbc8bd2b025d93f1ead5281e3a36fbae0c01eae57365ff0681ed6be770`
- REA MCP session integration verified via native MCP calls (`open_binary`, `binary_session`, `close_binary`).

---

## 3. Milestones Completed

- **Milestone 7**: Baseline foundation (68 tests)
- **Milestone A1**: Language, closures, string library (91 tests)
- **Milestone A2**: Native symmetric eig, svd, expm, fft (96 tests)
- **Milestone A3**: Integer classes, CSC sparse, N-D tensor ops (111 tests)
- **Milestone A3.1 - A3.1.4**: Corrective and integrity passes, zero unwrap policy, test manifest check (153 tests)
- **Milestone B2.1 - B2.1.1**: Clean-room statistics, RFC 4180 CSV matrix I/O, display goldens, MATLAB percentile rule, singular matrix warning + Inf (196 tests)
- **Milestone G0 (Active)**: B2.1.1 closeout, performance contracts, error ID standardization (`RUSTLAB:`), omitnan core migration.
