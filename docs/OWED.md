# Owed Items Register (docs/OWED.md)

This document tracks all owed items identified during review of Milestone G0 closeout and their resolutions in Milestone G0.1.

---

## 1. Inventory of Owed Items & Resolutions

| Item | Topic / Category | Source / Requirement | Resolution in G0.1 | Evidence & Artifacts |
| :--- | :--- | :--- | :--- | :--- |
| **O1** | `bench-results/a312/` Purpose & Inventory | Master Prompt 5.G0.a | Complete 13-file breakdown documenting scripts, folded profiles, flamegraphs, diffs, and step logs. | `docs/OWED.md`, `bench-results/a312/` |
| **O2** | Single-line `Cargo.toml` Additions | Master Prompt 5.G0.a | Clarified and verified that in commit `2b7ea43`, single-line additions were `license = "MIT OR Apache-2.0"` for crates (`rustlab-cli`, `rustlab-matio`, `rustlab-stdlib`). `rustlab-data` also added `license = "MIT OR Apache-2.0"` in G0.1. | `Cargo.toml`, crate `Cargo.toml` files |
| **O3** | `cargo deny check licenses` Status | Master Prompt 5.G0.a | Executed in session; cargo plugin `deny` is not installed on host (`error: no such command: deny`). Third-party dependencies verified manually via `cargo tree` to be MIT/Apache-2.0/BSD/ISC/Zlib/Unicode. | Command log in `docs/verify-logs/2026-10-26-g0.1.log` |
| **O4** | Hash Reconciliation (`5F12F389...` vs `8fd8d28d...`) | Master Prompt 5.G0.a | Confirmed builds: `5F12F389...` is A3.1.4 release binary (5,177,856 B); `8fd8d28d...` is B2.1 release binary (5,393,408 B, +`rustlab-data`). G0.1 release binary has hash `78a31a97...` (5,400,064 B). | `docs/re-notes/gemm-codegen.md`, `docs/STATE.md` |
| **O5** | `gemm-codegen.md` Build Markings | Master Prompt 5.G0.a | Documented exact compiler, target, flags, and build associations for A3.1.2, A3.1.4, B2.1, and G0/G0.1 binaries. | `docs/re-notes/gemm-codegen.md` |
| **O6** | Full Clean-Room REPL Transcript | Master Prompt 5.G0.b | Ran all 19 expressions in real release binary `rustlab.exe -e`; saved raw UTF-8 output to `docs/verify-logs/g0-repl-transcript.txt`. | `docs/verify-logs/g0-repl-transcript.txt` (SHA-256 `cab8e959...`) |
| **O7** | GNU Octave Parity Status | Master Prompt 5.G0.b | GNU Octave checked; not installed on Windows ARM64 host. Unverified items remain documented in `docs/spec/display-rules.md` and `docs/BACKLOG.md`. | Command log in `docs/verify-logs/2026-10-26-g0.1.log` |
| **O8** | Performance Contracts (LU, A+B, stats) | Master Prompt 5.G0.c | Measured release benchmarks: LU 100x100 is 0.242 ms (<= 0.6 ms); interpreter A+B is 3.314 ms; movmean/cummax/zscore steady state ~10-25 ms. Recorded in `bench-results/g01/`. | `bench-results/g01/bench-run[1-3].txt` |
| **O9** | Error ID Standardization (`RUSTLAB:`) | Master Prompt 5.G0.d | RustLab-only errors standardized to `RUSTLAB:` prefix across `rustlab-data` (`csv_io`, `stats`). | `crates/rustlab-data/src/csv_io.rs`, `crates/rustlab-data/src/stats.rs` |
| **O10** | Core Semantics & OmitNaN | Master Prompt 5.G0.d | Verified that `omitnan` is supported in core stdlib reductions (`sum`, `mean`, `min`, `max`) without requiring `--features data`. | `crates/rustlab-stdlib/src/stats.rs` |
| **O11** | CSV Version Specification | Master Prompt 5.G0.d | Pinned `csv = "1.4"` in `crates/rustlab-data/Cargo.toml` and documented `csv = "1.4.0"` in specifications. | `crates/rustlab-data/Cargo.toml`, `docs/STATE.md` |
| **O12** | REA R-Check Integration | Master Prompt 3, 5.G0.e | Completed REA tool session (`open_binary`, `binary_session`, `close_binary`) on `re\targets\rustlab.exe` with recorded SHA-256 and disassembled vector loop verification. | `docs/re-notes/CALL-LOG.md`, `re/targets/rustlab.exe` |

---

## 2. Details of `bench-results/a312/` Files (O1)

1. `extract-a2.ps1` (2,347 B): PowerShell script to extract and compare stdlib A2 linear algebra test names from verification logs.
2. `extract-verify.ps1` (4,430 B): PowerShell script parsing cargo test output headers, normalizing test runner executable names into crate units.
3. `flamegraph_ab_1e6.folded` (337 B): Folded stack traces for interpreter `A + B` on 1,000,000 double elements.
4. `flamegraph_ab_1e6.svg` (2,249 B): Visual SVG flamegraph of the interpreter binary addition hot path.
5. `verify-a2-diff.txt` (2,704 B): Test diff listing for `stdlib_a2_tests.rs` across verification stages.
6. `verify-a312-final.log` (107,710 B): Raw full transcript of the final A3.1.2 verification pass.
7. `verify-a312-final.utf8.log` (53,857 B): Cleaned UTF-8 transcript of `verify-a312-final.log`.
8. `verify-counts.txt` (6,128 B): Step-by-step test execution totals and per-unit breakdowns.
9. `verify-extract.txt` (38,476 B): Extracted test list per crate unit.
10. `verify-ordered-units.txt` (4,089 B): Ordered sequence of cargo test suite executions.
11. `verify-stepdiff.txt` (514 B): Differential analysis between Step 4 (default features) and Step 6 (no-default-features).
12. `verify-totals.txt` (114 B): Summary counts for Step 4 (142 passed), Step 5 (142 passed), and Step 6 (138 passed) in A3.1.2.
13. `verify-units.txt` (1,986 B): List of unique test execution units recognized by the parser.
