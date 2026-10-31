# Milestone backend-expansion: Retraction and Verification Record

## 1. Executive Summary & Verdict Correction
- **Original Claimed Verdict**: DONE (240/240/240 tests across workspace).
- **Corrected Verdict**: **DONE WITH MISSES**
- **Reason for Correction**:
  1. The statement in the prior report that feature drift was resolved and that all crates build identically with 240 tests under workspace `--no-default-features` was inaccurate. Due to Cargo's feature unification algorithm in workspace builds, workspace-wide `--no-default-features` was unifying features through other dependencies (`rustlab-cli` and `matrust_studio`), masking the actual per-crate feature differences.
  2. Per-crate runs truly without default features reveal that `rustlab-stdlib` executes 107 tests with default features, and 103 tests with `--no-default-features`.
  3. Interactive GUI behaviors in `matrust_studio` are NOT_TESTED live beyond confirming process execution, window creation, and event loop startup.
  4. G1 benchmark contracts (`movmean`, `cummax`, `A+B`) remain MISSED/OPEN.
  5. Octave parity is incomplete across extended capability builtins.
  6. REA ARM64 decompiler support is unavailable.
  7. Milestone B2.2 (headless SVG renderer) is NOT STARTED.
  8. Milestone B8 (Hardware and Embedded Systems) is PLANNED / BLOCKED.

---

## 2. Real Per-Crate Test Counts (Default vs Truly Without Default Features)

When tested per crate via `cargo test -p <crate> --no-default-features`:
- `rustlab-stdlib`: **107 tests (default)** vs **103 tests (no-default)** (Net diff: -4 tests).
  - In `stdlib_a2_tests.rs`:
    - With `faer`: 21 tests run (includes `test_a2_symmetric_eig_routes_to_faer_and_canonicalizes`, `test_a2_golden_eig_rotation_complex_conjugate`, `test_a2_both_native_and_faer_eig_paths_match`).
    - Without `faer`: 20 tests run (the 3 above are feature-gated out, while 2 non-faer fallback tests run: `test_a2_faer_feature_off_returns_clear_error` and `test_a2_native_eig_path_without_faer`).
  - In `table_tests.rs`:
    - With `data`: 3 tests run (`test_readtable_and_dot_indexing`, `test_head_and_tail_table`, `test_summary_and_writetable_roundtrip`).
    - Without `data`: 0 tests run (entire test file is `#![cfg(feature = "data")]`).
- `rustlab-math`: **22 tests (default)** vs **22 tests (no-default)**.
- `rustlab-cli`: **16 tests (default)** vs **16 tests (no-default)**.
- `matrust_studio`: **14 tests (default)** vs **14 tests (no-default)**.
- Non-feature crates:
  - `rustlab-data`: **45 tests**.
  - `rustlab-parse`: **13 tests**.
  - `rustlab-eval`: **15 tests**.
  - `rustlab-matio`: **8 tests**.

Total unique tests across workspace: 240.

---

## 3. Test File Integrity (5667f83 to 118b9e3)

No test files were deleted or weakened between G1 (`5667f83`) and `118b9e3`.
`git diff --name-status 5667f83..118b9e3` confirms only test additions and non-breaking edits:
- `A  apps/desktop/tests/desktop_tests.rs` (+14 tests)
- `M  crates/rustlab-data/tests/g01_verify_tests.rs` (extended error ID scanner)
- `A  crates/rustlab-stdlib/tests/extended_capabilities_tests.rs` (+5 tests)
- `A  crates/rustlab-stdlib/tests/numerical_tests.rs` (+6 tests)
- `M  crates/rustlab-stdlib/tests/regression_a312_tests.rs`
- `A  crates/rustlab-stdlib/tests/table_tests.rs` (+3 tests)

Total tests added since G1: exactly 28 tests (212 + 28 = 240).
The historical functional category table (25/11/16/43 rows summing to 284 vs 248) was inaccurate and is superseded by the verified canonical per-binary breakdown.
