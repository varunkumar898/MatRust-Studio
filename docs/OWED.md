# Owed Items Register (docs/OWED.md)

This document tracks all owed items identified during review of Milestone G0 closeout and their resolutions in Milestone G0.1 and the G0.1-VERIFY audit.

---

## 1. Inventory of Owed Items & Resolutions

| Item | Topic / Category | Source / Requirement | Resolution in G0.1 | Evidence & Artifacts |
| :--- | :--- | :--- | :--- | :--- |
| **O1** | `bench-results/a312/` Purpose & Inventory | Master Prompt 5.G0.a | Complete 13-file breakdown documenting scripts, folded profiles, flamegraphs, diffs, and step logs. File sizes and SHA-256s re-read in G0.1-VERIFY. | `docs/OWED.md` Â§2, `bench-results/a312/`, `docs/verify-logs/2026-10-28-g0.1-verify-a312-inventory.txt` |
| **O2** | Single-line `Cargo.toml` Additions | Master Prompt 5.G0.a | In commit `2b7ea43`, the one-line crate additions were `license = "MIT OR Apache-2.0"` in `rustlab-cli`, `rustlab-matio`, and `rustlab-stdlib`. The root `Cargo.toml` did **not** receive a one-line addition: it removed `panic = "unwind"` from `[profile.bench]` and changed the surrounding comments. `rustlab-data` added its license line in G0.1. | `git show 2b7ea43 -- Cargo.toml crates/*/Cargo.toml`; this file Â§3 |
| **O3** | `cargo deny check licenses` Status | Master Prompt 5.G0.a | **SKIPPED**: `cargo deny check licenses` returned `error: no such command: deny` (exit 101). No cargo-deny run was possible. The prior manual dependency-license review is a substitute, not the requested tool run. Raw output: `docs/verify-logs/2026-10-28-g0.1-verify-cargo-deny.txt`. | Same raw log; `docs/STATE.md` |
| **O4** | Hash Reconciliation (`02BBDA0C...`, `78A31A97...`, `5F12F389...`, `8fd8d28d...`) | Master Prompt 5.G0.a | `5F12F389...` is the A3.1.4 intermediate binary (5,177,856 B); `8fd8d28d...` is the B2.1 binary (5,393,408 B), not 5,177,856 B. `02BBDA0C...` is the M0 relocation build (5,400,064 B) built from the `24ac6e2` (`g0`) source; M0 commit `e84d542` changes docs/config only. `78A31A97...` is the G0.1 metadata build (5,400,064 B) from `95dcbca`; only crate license metadata changed. The canonical G0.1-VERIFY post-clean build is 5,416,960 B, SHA-256 `80E6AF24...` (`re/targets/rustlab-g0.1-verified.exe`). The pre-clean build `F6A302EB...` is preserved but is not canonical. | `docs/re-notes/gemm-codegen.md`, `docs/STATE.md`, hashes of `re/targets/rustlab-g0.1-verified.exe` and `re/targets/rustlab.exe` |
| **O5** | `gemm-codegen.md` Build Markings | Master Prompt 5.G0.a | Added a build-provenance map with a size, SHA-256, source commit/tag, and scope for every binary-specific section (A3.1.2, A3.1.4, B2.1, M0, G0, G0.1). The canonical G0.1-VERIFY build row is filled with 5,416,960 B / `80E6AF24...`; the pre-clean `F6A302EB...` build is marked non-canonical. | `docs/re-notes/gemm-codegen.md` |
| **O6** | Full Clean-Room REPL Transcript | Master Prompt 5.G0.b | The prior transcript was at the wrong path. G0.1-VERIFY regenerates the 19-prompt transcript with the canonical `80E6AF24...` binary and saves it to `bench-results/g0.1/repl.txt` (raw UTF-8). | `bench-results/g0.1/repl.txt` |
| **O7** | GNU Octave Parity Status | Master Prompt 5.G0.b | **SKIPPED**: `octave` and `octave-cli` are not installed on this host. Unverified display rules remain marked in `docs/spec/display-rules.md` and `docs/BACKLOG.md`; the new display goldens carry `_unverified` in their names. | `docs/verify-logs/2026-10-28-g0.1-verify-octave.txt` |
| **O8** | Performance Contracts (LU, A+B, stats) | Master Prompt 5.G0.c | Three canonical release runs: LU 100x100 median **0.274 ms** (<= 0.6 ms, MET); interpreter A+B 1e6 median **3.766 ms** (<= 3 ms, **MISSED**, highly unstable); movmean **13.865 ms** (MISSED ~10 ms); cummax **9.916 ms** (median just under 10, one run 11.129; marginal); zscore **12.135 ms** (MISSED). Raw: `bench-results/g0.1/bench-run[1-3].txt`; summary: `docs/verify-logs/2026-10-28-g0.1-verify-bench-summary.txt`. | `bench-results/g0.1/bench-run[1-3].txt` |
| **O9** | Error ID Standardization (`RUSTLAB:`) | Master Prompt 5.G0.d | RustLab-only errors standardized to `RUSTLAB:` prefix across `rustlab-data` (`csv_io`, `stats`). A new source-scan test rejects any future `MATLAB:` prefix in those modules unless the identifier is a documented generic MATLAB id. | `crates/rustlab-data/src/*.rs`, `crates/rustlab-data/tests/g01_verify_tests.rs` |
| **O10** | Core Semantics & OmitNaN | Master Prompt 5.G0.d | Verified that `omitnan` is supported in core stdlib reductions (`sum`, `mean`, `min`, `max`) without requiring `--features data`; new tests run in default, all-features, and no-default test configurations. | `crates/rustlab-stdlib/src/stats.rs`, `crates/rustlab-stdlib/tests/g01_verify_tests.rs` |
| **O11** | CSV Version Specification | Master Prompt 5.G0.d | Pinned `csv = "1.4"` in `crates/rustlab-data/Cargo.toml` and documented `csv = "1.4.0"` in specifications. | `crates/rustlab-data/Cargo.toml`, `docs/STATE.md` |
| **O12** | REA R-Check Integration | Master Prompt 3, 5.G0.e | **SKIPPED in this session (OpenCode)**: `opencode mcp list` returned `No MCP servers configured`; the probe returned `MCP server not found: rea`. The prior G0.1 REA rows are historical Antigravity claims, not calls made here. The shell SHA-256 of `re/targets/rustlab.exe` matches `78A31A97...`; the canonical final build is separately recorded as `80E6AF24...` in `re/targets/rustlab-g0.1-verified.exe`. No new REA session hash or disassembly was produced. | `docs/re-notes/CALL-LOG.md`, `docs/re-notes/g0.1-add-path.md`, `docs/verify-logs/2026-10-28-g0.1-verify-rea-mcp-list.txt` |

## 2. `bench-results/a312/` File-by-File Purpose (O1)

The 13 files are a frozen record of the A3.1.2 verification pass. Sizes and
SHA-256s are re-read in `docs/verify-logs/2026-10-28-g0.1-verify-a312-inventory.txt`.

1. `extract-a2.ps1` (2,347 B): extracts and compares stdlib A2 linear-algebra
   test names from the verification log.
2. `extract-verify.ps1` (4,430 B): parses cargo-test output headers, normalizes
   runner executable names into crate units, and totals counts.
3. `flamegraph_ab_1e6.folded` (337 B): folded stack samples for the interpreter
   `A + B` workload on 1,000,000 doubles.
4. `flamegraph_ab_1e6.svg` (2,249 B): rendered SVG of the same folded profile.
5. `verify-a2-diff.txt` (2,704 B): test-name diff for `stdlib_a2_tests.rs`
   across verification stages.
6. `verify-a312-final.log` (107,710 B): original full transcript of the final
   A3.1.2 verification pass (the raw runner log).
7. `verify-a312-final.utf8.log` (53,857 B): UTF-8 conversion of the raw log.
8. `verify-counts.txt` (6,128 B): step-by-step execution totals and per-unit
   breakdowns extracted from the log.
9. `verify-extract.txt` (38,476 B): extracted test list per crate unit.
10. `verify-ordered-units.txt` (4,089 B): ordered sequence of cargo-test suite
    executions.
11. `verify-stepdiff.txt` (514 B): difference between Step 4 (default features)
    and Step 6 (no-default-features) test runs.
12. `verify-totals.txt` (114 B): summary totals (142/142/138 for A3.1.2).
13. `verify-units.txt` (1,986 B): unique test execution units recognized by the
    parser.

These are historical A3.1.2 artifacts, not G0.1-VERIFY output. No file was
rewritten or re-labeled; G0.1-VERIFY only read sizes and hashes.

## 3. Exact `Cargo.toml` Changes (O2)

`git show 2b7ea43` (raw:
`docs/verify-logs/2026-10-28-g0.1-verify-cargo-oneliners.txt`) shows:

- `crates/rustlab-cli/Cargo.toml`: one line added,
  `license = "MIT OR Apache-2.0"`.
- `crates/rustlab-matio/Cargo.toml`: one line added,
  `license = "MIT OR Apache-2.0"`.
- `crates/rustlab-stdlib/Cargo.toml`: one line added,
  `license = "MIT OR Apache-2.0"`.
- The workspace root `Cargo.toml` did **not** receive a one-line addition. It
  removed `panic = "unwind"` from `[profile.bench]`, added a three-line comment
  above `[profile.release]`, and shortened the benchmark comment. This is a
  multi-line profile/comment edit, not a license line.
- `95dcbca` added two lines to `crates/rustlab-data/Cargo.toml`: a
  `description` and `license = "MIT OR Apache-2.0"` (raw:
  `docs/verify-logs/2026-10-28-g0.1-verify-data-license-line.txt`).
