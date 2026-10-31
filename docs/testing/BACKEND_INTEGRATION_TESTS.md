# MatRust Studio Backend Integration Tests

## 1. Test Suite Architecture

The test suite enforces mathematical correctness, API integrity, and end-to-end GUI-to-backend data paths across all capability areas:

```text
Unit Tests (Pure Rust Algorithms)
      │
      ▼
Runtime Integration Tests (eval_matlab / Interpreter)
      │
      ▼
Desktop GUI Integration Tests (Session, WorkerClient, GraphicsState)
```

## 2. Test Execution Commands & Verified Test Counts

All tests are verified offline from cargo clean.

| Test Binary | Location | Test Count | Status | Description |
|---|---|---|---|---|
| `desktop_tests` | `apps/desktop/tests/desktop_tests.rs` | 14 | **PASSED** | Docking, code folding, worker async, panic recovery, session builtins, numerical methods, simulation & robotics |
| `numerical_tests` | `crates/rustlab-stdlib/tests/numerical_tests.rs` | 6 | **PASSED** | Roots (companion matrix eig), polyval, polyfit (least squares), trapz, cumtrapz, gradient, interp1 |
| `table_tests` | `crates/rustlab-stdlib/tests/table_tests.rs` | 3 | **PASSED** | Readtable (CSV/TSV), dot variable indexing, head, tail, summary, writetable round-trip |
| `extended_capabilities_tests` | `crates/rustlab-stdlib/tests/extended_capabilities_tests.rs` | 5 | **PASSED** | Transfer function step response, digital filtering, window functions, xcorr, rgb2gray, imresize, fitlm, predict, kmeans, confusionmat, rotations, 2-link arm kinematics, jtraj |
| `regression_a312_tests` | `crates/rustlab-stdlib/tests/regression_a312_tests.rs` | 16 | **PASSED** | Bidirectional documentation and interactive help coverage for all registered builtins |
| `data_tests` | `crates/rustlab-data/tests/data_tests.rs` | 37 | **PASSED** | Statistics, windowing (movmean, movmedian), percentiles, cumulative min/max, CSV matrix IO |
| `g1_stats_reference_tests` | `crates/rustlab-data/tests/g1_stats_reference_tests.rs` | 7 | **PASSED** | Welford variance, reference statistics property tests |
| `cli_tests` | `crates/rustlab-cli/tests/cli_tests.rs` | 4 | **PASSED** | Singular matrix inversion stderr warnings, diagnostic error formatting |
| `diagnostics_tests` | `crates/rustlab-cli/tests/diagnostics_tests.rs` | 12 | **PASSED** | Caret positioning, CRLF insensitive span reporting |
| `eval_tests` | `crates/rustlab-eval/tests/eval_tests.rs` | 43 | **PASSED** | AST evaluator, environment scopes, lexical closures, function handles |
| `array_tests` | `crates/rustlab-math/tests/array_tests.rs` | 25 | **PASSED** | Column-major array indexing, elementwise math, matrix multiplication |
| `classes_tests` | `crates/rustlab-math/tests/classes_tests.rs` | 11 | **PASSED** | Integer class saturation, type mixing rules, cast operations |
| `sparse_tests` | `crates/rustlab-math/tests/sparse_tests.rs` | 16 | **PASSED** | CSC sparse representation, sparse-dense arithmetic |
| `value_tests` | `crates/rustlab-math/tests/value_tests.rs` | 1 | **PASSED** | Value enum conversions and memory layout |
| `matio_tests` | `crates/rustlab-matio/tests/matio_tests.rs` | 7 | **PASSED** | Level 5 MAT-file read/write roundtrip |
| `parse_tests` | `crates/rustlab-parse/tests/parse_tests.rs` | 10 | **PASSED** | Lexer tokenization, AST span accuracy |
| `stdlib_a1_tests` | `crates/rustlab-stdlib/tests/stdlib_a1_tests.rs` | 13 | **PASSED** | Anonymous closures, string library, regex |
| `stdlib_a2_tests` | `crates/rustlab-stdlib/tests/stdlib_a2_tests.rs` | 21 | **PASSED** | SVD, symmetric eig, pivoted QR, FFT/IFFT, matrix exponential |
| `stdlib_a3_tests` | `crates/rustlab-stdlib/tests/stdlib_a3_tests.rs` | 16 | **PASSED** | N-D array operations, sparse CSC builtins |
| `stdlib_tests` | `crates/rustlab-stdlib/tests/stdlib_tests.rs` | 13 | **PASSED** | Basic builtins, linear algebra solvers, multiple return values |
| **Total Workspace Tests** | **All Crates** | **244** | **100% PASSED** | **Zero failures, zero regressions** |
