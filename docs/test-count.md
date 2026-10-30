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

---

## 2. Comprehensive Suite Breakdown (Milestone G1)

| Crate | Unit (cargo header) | Default Passed | No-Default Passed | Status |
| :--- | :--- | :---: | :---: | :--- |
| **rustlab-cli** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-cli** | `unittests src\main.rs` | 0 | 0 | clean |
| **rustlab-cli** | `tests/cli_tests.rs` | 4 | 4 | PASSED |
| **rustlab-cli** | `tests/diagnostics_tests.rs` | 12 | 12 | PASSED |
| **rustlab-data** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-data** | `tests/data_tests.rs` | 37 | 37 | PASSED |
| **rustlab-data** | `tests/g01_verify_tests.rs` | 1 | 1 | PASSED |
| **rustlab-data** | `tests/stats_reference_tests.rs` | 7 | 7 | PASSED |
| **rustlab-eval** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-eval** | `tests/eval_tests.rs` | 9 | 9 | PASSED |
| **rustlab-eval** | `tests/milestone_a1_tests.rs` | 6 | 6 | PASSED |
| **rustlab-math** | `unittests src\lib.rs` | 10 | 10 | PASSED |
| **rustlab-math** | `tests/math_tests.rs` | 12 | 12 | PASSED |
| **rustlab-matio** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-matio** | `tests/matio_tests.rs` | 7 | 7 | PASSED |
| **rustlab-matio** | `tests/scipy_mat_tests.rs` | 1 | 1 | PASSED |
| **rustlab-parse** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-parse** | `tests/lexer_tests.rs` | 4 | 4 | PASSED |
| **rustlab-parse** | `tests/parser_tests.rs` | 9 | 9 | PASSED |
| **rustlab-stdlib** | `unittests src\lib.rs` | 7 | 7 | PASSED |
| **rustlab-stdlib** | `tests/g01_verify_tests.rs` | 7 | 7 | PASSED |
| **rustlab-stdlib** | `tests/regression_a312_tests.rs` | 16 | 16 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_tests.rs` | 13 | 13 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a1_tests.rs` | 13 | 13 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a2_tests.rs` | 21 | 20 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a3_tests.rs` | 16 | 16 | PASSED |
| *Doc-tests* | all 7 crates | 0 | 0 | clean |
| **TOTAL** | 34 units incl. doc-tests | **212** | **211** | **100% PASSED** |

---

## 3. Justification of Test Changes (B2.1.1, G0.1-VERIFY, and G1)

As strictly mandated by reporting rules, all additions, modifications, or removals are documented:
1. **Added 24 Tests to `crates/rustlab-data/tests/data_tests.rs`** (expanding from 13 to 37).
2. **Added 1 Test to `crates/rustlab-stdlib/tests/stdlib_tests.rs`**.
3. **No Tests Removed**: Total test count strictly increased.

### G0.1-VERIFY additions (9 tests)
4. **Added 7 tests to `crates/rustlab-stdlib/tests/g01_verify_tests.rs`**.
5. **Added 1 test to `crates/rustlab-data/tests/g01_verify_tests.rs`**.
6. **Added 1 integration test to `crates/rustlab-cli/tests/cli_tests.rs`**.

### G1 additions (7 tests)
7. **Added 7 reference tests to `crates/rustlab-data/tests/stats_reference_tests.rs`**:
   - `test_reference_empty_and_len1`: edge cases for empty and length-1 vectors across `movmean`, `zscore`, `cummax`.
   - `test_reference_constant_data`: verifies identical results between naive references and optimized routines for constant arrays across all $k$.
   - `test_reference_drift_over_100k_elements`: asserts drift $< 10^{-10}$ over 100,000 elements with periodic exact recompute every 1024 steps.
   - `test_reference_random_data`: validates `movmean` against naive window recompute on LCG random data.
   - `test_reference_offset_1e8_plus_noise`: validates catastrophic cancellation resistance for data with large offsets ($10^8$) plus small perturbations.
   - `test_reference_nan_and_inf_windows`: confirms exact fallback on non-finite windows (NaN, Inf, -Inf) and clean recovery to running sum once windows return to all-finite.
   - `test_reference_window_edges_and_k_equals_len`: tests boundary behaviors for $k=1$ and $k=\text{len}$.

---

## 4. Test Manifest Integrity CI Enforcement

All 212 default test identifiers are recorded in `docs/test-manifest.txt` directly from `cargo test --workspace -- --list`. `scripts/verify.ps1` runs a manifest integrity check (Step 7) comparing active tests against `docs/test-manifest.txt`, failing if any entry disappears without explicit manifest modification.
