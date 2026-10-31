# RustLab Test Count and Accounting Ledger

This document tracks the cumulative and per-suite test counts across milestones, providing full auditability and preventing test regressions.

---

## 1. Milestone Accounting and Progression

| Milestone | Total Tests (Default) | Total Tests (No-Default-Features) | Suite Breakdown | Notes |
| :--- | :---: | :---: | :--- | :--- |
| **Milestone 7** | 68 | 68 | Parse: 13, Math: 17, Eval: 9, Stdlib: 19, MatIO: 7, CLI: 3 | Baseline foundation |
| **Milestone A1** | 91 | 91 | +6 Milestone A1 integration tests, +13 stdlib A1 tests | Language, closures, string library |
| **Milestone A2** | 96 | 96 | +11 stdlib A2 linear algebra tests | Native symmetric eig, svd, expm, fft |
| **Milestone A3** | 111 | 111 | +5 sparse/int math tests, +10 stdlib A3 tests | Int classes, CSC sparse, N-D tensor ops |
| **Milestone A3.1** | 119 | 119 | +6 stdlib A3.1 golden tests | Corrective pass: 0 unwraps, exact error messages |
| **Milestone A3.1.2** | 152 | 148 | +11 regression_a312, +12 diagnostics, +8 stdlib_a2 | Deny lints, display parity, R0 clean-room |
| **Milestone A3.1.4** | **153** | **153** | +1 `test_a2_both_native_and_faer_eig_paths_match`, +1 `test_scientific_notation_threshold_rule_cited`, -1 duplicate builtin test deduplicated, native Jacobi SVD enabled in no-default-features | Integrity pass: test-manifest CI check, 173 builtins verified two-way |
| **Milestone B2.1** | **171** | **171** | +13 `rustlab-data` (statistics + CSV matrix I/O), +4 display goldens, +1 symmetric eig route test | Clean-room statistics, RFC 4180 CSV matrix I/O, BOM/CRLF, ragged row detection |
| **Milestone B2.1.1** | **196** | **195** | +24 `rustlab-data` tests (depth, properties, CSV edge cases, display goldens), +1 `rustlab-stdlib` singular matrix inv/mldivide Inf test | B2.1.1 corrective pass: display goldens, MATLAB percentile rule, singular matrix warning + Inf, $O(n)$ movmean/cummax/zscore |
| **Milestone G0.1-VERIFY** | **205** | **204** | +7 `rustlab-stdlib` G0.1 regressions, +1 `rustlab-data` RUSTLAB-id scan, +1 CLI singular-warning integration test | Verification-only pass: LU residuals, singular warning, core `omitnan`, mask ids, display goldens, test-manifest refresh |
| **Milestone G1** | **212** | **211** | +7 `rustlab-data` stats reference tests | Stats contract optimization & reproducible build validation |
| **Branch wip/backend-expansion** | **240** | **240 (workspace) / 103 stdlib (per-crate)** | +14 desktop tests, +6 numerical tests, +5 extended capabilities tests, +3 table tests | Unified Rust backend expansion across 8 capability areas & MatRust Studio desktop IDE. Net +28 tests since G1. |
| **Milestone mlapp-rs & GitHub CI** | **302** | **302** | +16 `mlapp-container`, +7 `mlapp-cli`, +5 `mlapp-ui`, +5 `mlapp-lang`, +10 `mlapp-runtime`, +7 `mlapp-host`, +4 `mlapp-plot`, +3 `mlapp-render`, +3 `mlapp-ffi`, +2 `rustlab-math` proptests | Complete MATLAB App Designer runtime (.mlapp/.mlappinstall), container security, C FFI handles, conformance suite, and property tests. Net +62 tests. Zero regressions. |
| **Milestone Hardening & Fuzz/Proptest** | **309** | **309** | +3 `rustlab-parse` proptests, +1 `rustlab-math` N-D indexing proptest, +1 `mlapp-render` headless screenshot test, +1 `mlapp-ffi` alias header test, +1 `rustlab-stdlib` doc generator parity test | Full workspace hardening, zip 9.2 & quick-xml 0.41 upgrades, 0 unreachable ignores, cargo-fuzz targets, and criterion host benchmarks. Net +7 tests. |

---

## 2. Reconciliation of Math and Stdlib Test Counts (37 vs 13 Explanation)

A question arose during audits regarding whether `rustlab-math` dropped from 37 tests to 13.
Investigation and historical logs confirm:
- The suite containing **37 tests** is `crates/rustlab-data/tests/data_tests.rs` (37 tests).
- `crates/rustlab-math` has continuously maintained:
  - `crates/rustlab-math/src/lib.rs` (unit tests): 10 tests
  - `crates/rustlab-math/tests/math_tests.rs` (integration tests): 13 tests
  - Added in CI / Hardening: `crates/rustlab-math/tests/proptest_math.rs`: 3 property tests
- Total in `rustlab-math` is currently **26 tests**. Zero math tests were ever lost, deleted, or skipped. All test suites across all crates are fully tracked and verified.
