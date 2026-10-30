# RustLab Project State (docs/STATE.md)

**Current Milestone:** G0.1-VERIFY (verification-only closeout; no new features)
**Next Milestone:** G1 (self-debug tools; wait for owner approval)
**Root Path:** `C:\Users\varun\Desktop\MATSTUDIO`
**Date:** 2026-10-28
**Repository Commit:** tag `g0.1-verified` (commit hash printed by `git log -1`)
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`), Rust 1.99.1 stable
**Verification Status:** `scripts/verify.ps1` run after `cargo clean` exited 0; log:
`docs/verify-logs/2026-10-28-g0.1-verify-clean.log` (UTF-8, 205/205/204 test results,
manifest check passed).
**Total Tests:**
- Default features: **205** passed
- All features: **205** passed
- No-default features: **204** passed (`test_a2_both_native_and_faer_eig_paths_match`, `test_a2_golden_eig_rotation_complex_conjugate`, and `test_a2_symmetric_eig_routes_to_faer_and_canonicalizes` replaced by `test_a2_faer_feature_off_returns_clear_error` and `test_a2_native_eig_path_without_faer`)

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

### Canonical G0.1-VERIFY release build

- Size: **5,416,960 bytes**
- SHA-256: `80E6AF2442D97CED5E67145D0DF2B9961EF5957ED178049C689457B67CD5CA58`
- Preserved copy: `re/targets/rustlab-g0.1-verified.exe`
- Also present at `target/release/rustlab.exe` after the post-verify rebuild; both
  files had the same size and SHA-256 when checked.
- Build command: `cargo build --release -p rustlab-cli` on the final source tree.
- The REPL transcript (`bench-results/g0.1/repl.txt`) and the three measured
  benchmark runs were regenerated with this exact binary.

The pre-`cargo clean` build (same size, SHA-256 `F6A302EB92DD04CDE4C3B54D9F90E3A7288341737D7089EC5F1341CDDCDEDAC0`)
is retained as `re/targets/rustlab-g0.1-verified-preclean-F6A302EB.exe` and
under `bench-results/g0.1/preclean-F6A302EB/`. It is **not** the canonical build;
the post-clean rebuild is authoritative.

### Historical hashes

| Build | Size | SHA-256 | Notes |
| :--- | ---: | :--- | :--- |
| M0 relocation copy (historical) | 5,400,064 B | `02bbda0c68cafe87e01a33fa15f71c2bae96e7cd3f95dafcb82c712380f7460c` | Executable code from `24ac6e2` (`g0`); M0 commit `e84d542` changed docs/config only. Not the current `re/targets/rustlab.exe`. |
| G0.1 metadata rebuild, current `re/targets/rustlab.exe` | 5,400,064 B | `78a31a97b1e73696d5ec2ea434af9878da543bda43d4f058cf5c21a6eb42c3c7` | Built at `95dcbca`; only crate license metadata changed. Previously mislabeled as the canonical G0.1 binary. |
| G0 closeout | 5,400,064 B | `683ea9cbc8bd2b025d93f1ead5281e3a36fbae0c01eae57365ff0681ed6be770` | `24ac6e2`, tag `g0`. |
| B2.1 | 5,393,408 B | `8fd8d28df96d1f36e9dfbd82c4566a26ee00cfee05add359cfb31fe191ff58d3` | Not 5,177,856 B; the earlier G0 note was wrong. |
| A3.1.4 intermediate | 5,177,856 B | `5F12F38994E80D2BC910CD8D36D611DE78CF4CD9004B2EC6DF759A11BFED0288` | Historical. |

### REA

- REA Project Root: `C:\Users\varun\Desktop\MATSTUDIO\re\projects`
- REA is configured in Antigravity, **not** in OpenCode. G0.1-VERIFY's R-check is
  **SKIPPED** with exact errors; see `docs/re-notes/CALL-LOG.md` and
  `docs/re-notes/g0.1-add-path.md`.
- No new REA session hash or disassembly was produced in this session.

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
- **Milestone G0.1**: Unverified baseline only. Added owed-items review, a REA R-check claim, a REPL transcript at the wrong path, and a release benchmark triplet; rejected by the owner as not sufficient evidence.
- **Milestone R0**: Added `docs/ROADMAP.md` with status from git tags; recorded next milestone as G0.1. Documentation only; no feature code.
- **Milestone G0.1-VERIFY**: Verification-only closeout. Added nine missing regressions, settled the mask-count error id, corrected display rules and LU backlog notes, reconciled binary hashes, re-ran release benchmarks with a 1,000,000-row CSV and single-thread GEMM proof, regenerated the 19-prompt REPL transcript at `bench-results/g0.1/repl.txt`, and passed `scripts/verify.ps1` from `cargo clean` at 205/205/204. Tag: `g0.1-verified`.
