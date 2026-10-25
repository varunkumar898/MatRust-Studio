# RustLab Test Count and Accounting Ledger

This document tracks the cumulative and per-suite test counts across milestones, providing full auditability and preventing test regressions.

---

## 1. Milestone Accounting and Progression

| Milestone | Total Tests (Default) | Total Tests (No-Default) | Notes |
| :--- | :--- | :--- | :--- |
| **Milestone 7** | 68 | 68 | Baseline foundation |
| **Milestone A1** | 91 | 91 | +6 Milestone A1 integration tests, +13 stdlib A1 tests |
| **Milestone A2** | 96 | 96 | +11 stdlib A2 linear algebra tests |
| **Milestone A3** | 111 | 111 | +5 sparse/int math tests, +10 stdlib A3 tests |
| **Milestone A3.1** | 119 | 115 | +6 stdlib A3.1 golden tests |
| **Milestone A3.1.2**| 121 | 118 | +1 doc/help sync, +1 scipy.io MAT interop |
| **Milestone A3.1.3**| **152** | **148** | +12 diagnostics tests, +11 regression A3.1.2 tests, +8 A2.1 named tests (`expm`, `pinv`, pivoted QR, tolerance rule), 100% PASS |

---

## 2. Comprehensive Suite Breakdown (Milestone A3.1.3)

| Crate / Target | Test File / Module | Tests Passed (Default) | Tests Passed (No-Default) | Status |
| :--- | :--- | :---: | :---: | :--- |
| **rustlab-cli** | `tests/cli_tests.rs` | 3 | 3 | PASSED |
| **rustlab-cli** | `tests/diagnostics_tests.rs` | 12 | 12 | PASSED |
| **rustlab-eval** | `tests/eval_tests.rs` | 9 | 9 | PASSED |
| **rustlab-eval** | `tests/milestone_a1_tests.rs` | 6 | 6 | PASSED |
| **rustlab-math** | `src/lib.rs` (Unit tests) | 10 | 10 | PASSED |
| **rustlab-math** | `tests/math_tests.rs` | 12 | 12 | PASSED |
| **rustlab-matio**| `tests/matio_tests.rs` | 7 | 7 | PASSED |
| **rustlab-matio**| `tests/scipy_mat_tests.rs` | 1 | 1 | PASSED (validates 40 scipy.io MAT files) |
| **rustlab-parse**| `tests/lexer_tests.rs` | 4 | 4 | PASSED |
| **rustlab-parse**| `tests/parser_tests.rs` | 9 | 9 | PASSED |
| **rustlab-stdlib**| `src/lib.rs` (Unit tests) | 7 | 7 | PASSED |
| **rustlab-stdlib**| `tests/regression_a312_tests.rs` | 11 | 11 | PASSED |
| **rustlab-stdlib**| `tests/stdlib_a1_tests.rs` | 13 | 13 | PASSED |
| **rustlab-stdlib**| `tests/stdlib_a2_tests.rs` | 19 | 15 | PASSED (`faer`-off clear error verified) |
| **rustlab-stdlib**| `tests/stdlib_a3_tests.rs` | 17 | 17 | PASSED |
| **rustlab-stdlib**| `tests/stdlib_tests.rs` | 12 | 12 | PASSED |
| **TOTAL** | | **152** | **148** | **100% PASSED** |

---

## 3. Two-Way Sync Audit
- Registered stdlib builtins: 164
- Evaluator builtins: 7 (`cellfun`, `arrayfun`, `structfun`, `which`, `exist`, `addpath`, `rmpath`)
- Newly registered N-D builtins: 2 (`ipermute`, `circshift`)
- Total registered engine builtins: **173**
- Markdown documentation files in `docs/functions/*.md`: **173**
- Builtin Help registry entries in `help.rs`: **173**
- Verification: Tested and guaranteed by `test_a3_all_registered_builtins_have_markdown_docs_and_help`. Zero orphans, zero missing.
