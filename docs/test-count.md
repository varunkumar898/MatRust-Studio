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

---

## 2. Comprehensive Suite Breakdown (Milestone G0.1-VERIFY)

| Crate | Unit (cargo header) | Default Passed | No-Default Passed | Status |
| :--- | :--- | :---: | :---: | :--- |
| **rustlab-cli** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-cli** | `unittests src\main.rs` | 0 | 0 | clean |
| **rustlab-cli** | `tests/cli_tests.rs` | 4 | 4 | PASSED |
| **rustlab-cli** | `tests/diagnostics_tests.rs` | 12 | 12 | PASSED |
| **rustlab-data** | `unittests src\lib.rs` | 0 | 0 | clean |
| **rustlab-data** | `tests/data_tests.rs` | 37 | 37 | PASSED |
| **rustlab-data** | `tests/g01_verify_tests.rs` | 1 | 1 | PASSED |
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
| **TOTAL** | 33 units incl. doc-tests | **205** | **204** | **100% PASSED** |

---

## 3. Justification of Test Changes (B2.1.1 and G0.1-VERIFY)

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

### G0.1-VERIFY additions (9 tests)

4. **Added 7 tests to `crates/rustlab-stdlib/tests/g01_verify_tests.rs`**:
   - `test_g01_lu_residuals_random_5x5` and `test_g01_lu_residuals_random_50x50`:
     deterministic random, diagonally dominant matrices; check `P*A = L*U`
     and `||A*x - b||_inf < 1e-10`.
   - `test_g01_mldivide_singular_returns_inf`: singular `A \ b` returns an
     all-`Inf` result.
   - `test_g01_core_omitnan_sum_mean_min_max`: core `sum`/`mean`/`min`/`max`
     with explicit `"omitnan"`; also checks `sum`/`mean` default and
     `"includenan"` NaN propagation.
   - `test_g01_mask_count_and_size_identifiers_settled`: mask-count error id is
     `MATLAB:subsassigndimmismatch`; mask-size error id/message is
     `MATLAB:dimagree` / `"Array dimensions must match for binary array op or logical indexing."`
   - `test_g01_display_scale_factor_0_001_1000_golden_unverified` and
     `test_g01_display_extreme_range_1e_16_1_golden_unverified`: byte-exact
     display strings. The `_unverified` suffix records that Octave/MATLAB was
     not available to confirm the exact layout.
5. **Added 1 test to `crates/rustlab-data/tests/g01_verify_tests.rs`**:
   - `test_g01_rustlab_only_ids_do_not_use_matlab_prefix`: source scan rejects
     future RustLab-only `MATLAB:` identifiers in `stats.rs`/`csv_io.rs` unless
     the id is in the documented generic allow-list.
6. **Added 1 integration test to `crates/rustlab-cli/tests/cli_tests.rs`**:
   - `test_g01_singular_inv_warns_on_stderr_and_prints_inf`: runs the real
     `rustlab` test binary, asserts the singular warning is on stderr and
     `Inf` is on stdout, and that the process succeeds.
7. **No tests removed.** Default count went from 196 to 205; no-default from
   195 to 204. The default/no-default name diff is the same five faer/native
   eig tests as B2.1.1 (3 default-only, 2 no-default-only).

---

## 4. Test Manifest Integrity CI Enforcement

All 205 default test identifiers are recorded in `docs/test-manifest.txt`
directly from `cargo test --workspace -- --list` (duplicate names from
different test binaries are preserved as separate lines). `scripts/verify.ps1`
runs a manifest integrity check (Step 7) comparing active tests against
`docs/test-manifest.txt`, failing if any entry disappears without explicit
manifest modification. Raw lists:
`docs/verify-logs/2026-10-28-g0.1-verify-test-list-default.txt` and
`...-test-list-no-default.txt`.
