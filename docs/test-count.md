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

---

## 2. Comprehensive Suite Breakdown (Milestone A3.1.4)

| Crate | Unit (cargo header) | Default Passed | No-Default Passed | Status |
| :--- | :--- | :---: | :---: | :--- |
| **rustlab-cli** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-cli** | `unittests src\main.rs` | 0 | 0 | clean |
| **rustlab-cli** | `tests/cli_tests.rs` | 3 | 3 | PASSED |
| **rustlab-cli** | `tests/diagnostics_tests.rs` | 12 | 12 | PASSED |
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
| **rustlab-stdlib** | `tests/regression_a312_tests.rs` | 12 | 12 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_tests.rs` | 12 | 12 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a1_tests.rs` | 13 | 13 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a2_tests.rs` | 20 | 20 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a3_tests.rs` | 16 | 16 | PASSED |
| *Doc-tests* | all 6 crates | 0 | 0 | clean |
| **TOTAL** | 27 units incl. doc-tests | **153** | **153** | **100% PASSED** |

---

## 3. Justification of Test Changes in A3.1.4

As strictly mandated by reporting rules, all additions, modifications, or removals are documented:
1. **Added `test_a2_both_native_and_faer_eig_paths_match`** (`crates/rustlab-stdlib/tests/stdlib_a2_tests.rs`):
   - Verifies that both native symmetric eigenvalue decomposition (`eig_sym`) and `faer` self-adjoint eigenvalue decomposition produce identical sorted eigenvalues ($< 10^{-12}$) and identical sign-normalized eigenvectors.
2. **Added `test_scientific_notation_threshold_rule_cited`** (`crates/rustlab-stdlib/tests/regression_a312_tests.rs`):
   - Explicitly tests the $10^{-3}$ and $10^3$ scale factor and scientific notation threshold rules cited from official MathWorks MATLAB documentation (`format.html`).
3. **Removed Duplicate Test `test_a3_all_registered_builtins_have_markdown_docs_and_help`** (`crates/rustlab-stdlib/tests/stdlib_a3_tests.rs`):
   - **Written Justification**: Task 4 explicitly instructed: *"Ensure only ONE test checks both directions (zero missing docs, zero orphan doc files, zero help mismatches), removing any duplicate test and hardcoded counts."* `every_registered_builtin_has_docs_and_help` in `regression_a312_tests.rs` was enhanced to perform the comprehensive two-way validation across all 173 registered builtins, eliminating the redundant duplicate.
4. **Parity in `--no-default-features` (Native Jacobi SVD)**:
   - With the implementation of `jacobi_svd_native`, four tests in `stdlib_a2_tests.rs` (`test_a2_golden_svd_rank1`, `test_a2_golden_pinv_rank_deficient`, `test_a2_pinv_moore_penrose_condition`, `test_a2_tolerance_rule_max_mn_eps_sigma1`) now execute and pass without `faer`. Total tests in `--no-default-features` mode increased from 148 to **153**.

---

## 4. Test Manifest Integrity CI Enforcement

All 153 test identifiers are recorded in `docs/test-manifest.txt` directly from `cargo test --workspace -- --list`. `scripts/verify.ps1` runs a manifest integrity check (Step 7) comparing active tests against `docs/test-manifest.txt`, failing if any entry disappears without explicit manifest modification.
