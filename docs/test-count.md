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

---

## 2. Comprehensive Suite Breakdown (Milestone B2.1.1)

| Crate | Unit (cargo header) | Default Passed | No-Default Passed | Status |
| :--- | :--- | :---: | :---: | :--- |
| **rustlab-cli** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-cli** | `unittests src\main.rs` | 0 | 0 | clean |
| **rustlab-cli** | `tests/cli_tests.rs` | 3 | 3 | PASSED |
| **rustlab-cli** | `tests/diagnostics_tests.rs` | 12 | 12 | PASSED |
| **rustlab-data** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-data** | `tests/data_tests.rs` | 37 | 37 | PASSED |
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
| **rustlab-stdlib** | `tests/regression_a312_tests.rs` | 16 | 16 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_tests.rs` | 13 | 13 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a1_tests.rs` | 13 | 13 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a2_tests.rs` | 21 | 20 | PASSED |
| **rustlab-stdlib** | `tests/stdlib_a3_tests.rs` | 16 | 16 | PASSED |
| *Doc-tests* | all 7 crates | 0 | 0 | clean |
| **TOTAL** | 31 units incl. doc-tests | **196** | **195** | **100% PASSED** |

---

## 3. Justification of Test Changes in B2.1.1

As strictly mandated by reporting rules, all additions, modifications, or removals are documented:
1. **Added 24 Tests to `crates/rustlab-data/tests/data_tests.rs`** (expanding from 13 to 37):
   - Display goldens: `test_b2_display_mixed_integer_decimals_unverified`, `test_b2_display_scientific_and_scale_factor_goldens_unverified`.
   - Vector orientation: `test_b2_unique_row_orientation` (verifying $1 \times M$ row output for row vector input).
   - Builtin edge cases ($\ge 2$ tests per B2.1 builtin): empty arrays, NaNs, Infs, dimension arguments, integer classes, wrong argument count/type diagnostics, even/odd lengths, ties, window larger than vector, endpoints behavior.
     - `test_b2_mean_and_median_edge_cases`, `test_b2_var_and_std_edge_cases`, `test_b2_var_and_std_flags_dims_integer_classes`, `test_b2_cov_and_corrcoef_edge_cases`, `test_b2_prctile_and_quantile_edge_cases`, `test_b2_prctile_and_quantile_interpolation`, `test_b2_prctile_matlab_rule_goldens`, `test_b2_iqr_and_mad_edge_cases`, `test_b2_mode_edge_cases`, `test_b2_movmean_and_movmedian_edge_cases`, `test_b2_cummax_and_cummin_edge_cases`, `test_b2_zscore_edge_cases`, `test_b2_sortrows_and_unique_edge_cases`.
   - Property tests: `test_b2_percentiles_monotone_property`, `test_b2_variance_non_negative_property`, `test_b2_corrcoef_diagonal_property`, `test_b2_movmean_of_constants_property`.
   - CSV edge cases: `test_b2_csv_quoted_newlines_and_delimiters`, `test_b2_csv_bom_and_crlf`, `test_b2_csv_empty_trailing_field_and_header_only`, `test_b2_csv_nan_inf_roundtrip`, `test_b2_csv_bit_exact_double_roundtrip`.
2. **Added 1 Test to `crates/rustlab-stdlib/tests/stdlib_tests.rs`**:
   - `test_b2_inv_and_mldivide_singular_matrix_returns_inf`: verifies that both `inv(A)` and `A \ b` for a singular matrix emit the warning `"Matrix is singular to working precision."` and return an `Inf` array instead of panicking or throwing an unhandled runtime error.
3. **No Tests Removed**:
   - Total test count strictly increased from 171 to 196 (default) and 195 (no-default).

---

## 4. Test Manifest Integrity CI Enforcement

All 196 test identifiers are recorded in `docs/test-manifest.txt` directly from `cargo test --workspace -- --list`. `scripts/verify.ps1` runs a manifest integrity check (Step 7) comparing active tests against `docs/test-manifest.txt`, failing if any entry disappears without explicit manifest modification.
