# RustLab Project State (docs/STATE.md)

**Current Branch:** `wip/backend-expansion` (Backend expansion across 8 capability areas & MatRust Studio desktop IDE)  
**Baseline Tag:** `g1-verified` (commit `5667f83`), fix commit `ca3c325` (`g1-fix`)  
**Root Path:** `C:\Users\varun\Desktop\MATSTUDIO`  
**Date:** 2026-10-31  
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`), Rust 1.99.1 stable  
**Verification Status:** `scripts/verify.ps1` clean gate run passed; log: `docs/verify-logs/2026-10-31-verify.log` (SHA-256: `9662F445512731E237C167CEA3C0F30371648FD9F2B81430E96EC019DEECB133`).  
**Total Tests:**
- Default features: **240** passed (0 failed, across 24 test binaries)
- All features: **240** passed
- No-default features: **240** passed
- Test manifest integrity: all **212** historical manifest tests present and passing; 28 new tests added (14 desktop, 6 numerical, 5 extended capabilities, 3 tables).

> **Note on historical paths:** Paths recorded before milestone M0 refer to the old repository location (`C:\Users\varun\.antigravity\rustlab`). Live configuration, RE targets, and project roots are relocated to `C:\Users\varun\Desktop\MATSTUDIO`.

---

## 1. Crate Hierarchy & Layout

| Crate | Responsibility | Dependencies | Feature Flags |
| :--- | :--- | :--- | :--- |
| `crates/rustlab-parse` | Lexer, AST definitions with `Span`, recursive-descent parser | `thiserror` | None |
| `crates/rustlab-math` | Column-major `DenseArray`, `Value` enum, CSC sparse matrix, integer classes | `rustlab-parse`, `matrixmultiply`, `num-complex`, `num-traits` | `matrixmultiply` |
| `crates/rustlab-eval` | AST evaluator, runtime `Environment`, closure handles | `rustlab-parse`, `rustlab-math`, `thiserror` | None |
| `crates/rustlab-stdlib` | Standard library builtins (elementary math, linalg, FFT, string, IO, numerical methods, simulation, signal/image processing, ML, robotics) | `rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-matio`, `rustlab-data` (opt) | `faer`, `data` |
| `crates/rustlab-matio` | Level 5 MAT-file read/write | `rustlab-math`, `flate2` | None |
| `crates/rustlab-data` | Summary statistics, windowing, rankings, CSV matrix I/O, tabular datasets (`readtable`, `writetable`, `head`, `tail`, `summary`) | `rustlab-parse`, `rustlab-math`, `rustlab-eval`, `csv = "1.4"`, `ryu = "1.0"` | None |
| `crates/rustlab-cli` | REPL, line-editor, diagnostic error reporting, benchmark suite | `rustlab-eval`, `rustlab-stdlib`, `rustyline`, `clap` | `faer`, `data` |
| `apps/desktop` | MatRust Studio desktop IDE (custom DockTree, code folding, breakpoints, worker execution thread, interactive command window, workspace panel, figures viewer, status bar) | `rustlab-eval`, `rustlab-stdlib`, `rustlab-data`, `rustlab-matio`, `eframe = "0.31"`, `egui = "0.31"`, `egui_plot = "0.31"` | None |

---

## 2. Binary Hashes & REA Target

### G1 Reproducible Release Builds

Both intermediate commit `a4433f5` and tag `g1-verified` (`5667f83`) were proven **100% bit-for-bit reproducible** across independent directories ($0$ byte diff between two separate clone builds); all embedded host filesystem paths neutralized via `--remap-path-prefix` (0 occurrences of `varun`, `.cargo`, `.rustup`, `MATSTUDIO`). Unstripped for reverse-engineering inspection.
Build command: `powershell.exe -File scripts/build-release.ps1`

- **Intermediate commit `a4433f5` build:**
  - Size: **5,400,576 bytes**
  - SHA-256: `8036F5016640AAF9EA38E3EE3EBCA052C20780FBD7912ADE91A6527C49BEDDAC`
  - Preserved copy: `re/targets/rustlab-g1-verified.exe`
  - Measured benchmark runs: Three runs stored in `bench-results/g1/bench-run[1-3].txt`.
- **Tag `g1-verified` (`5667f83`) build:**
  - Size: **5,401,600 bytes**
  - SHA-256: `726520C91EE79810C84159E0B161F282686AC6F71979462C26DD9A8FB0A14728`
  - Difference (+1,024 B): Clippy warning resolutions (`as_chunks`, slice `.fill()`, direct indexing) committed in `5667f83`.
  - Proven bit-for-bit identical across two independent clones (`$env:TEMP\g1-a` and `$env:TEMP\g1-b`). Raw log: `docs/verify-logs/2026-10-30-g1-repro-byte-diff.txt`.

*Note on canonical hashes:* Neither hash is designated as an unconditional canonical hash without specifying its exact commit hash.

### Historical hashes

| Build | Size | SHA-256 | Notes |
| :--- | ---: | :--- | :--- |
| G1 tag `g1-verified` (`5667f83`) | 5,401,600 B | `726520C91EE79810C84159E0B161F282686AC6F71979462C26DD9A8FB0A14728` | Tagged release build. 100% reproducible (0-byte diff across clones). |
| G1 commit `a4433f5` build | 5,400,576 B | `8036F5016640AAF9EA38E3EE3EBCA052C20780FBD7912ADE91A6527C49BEDDAC` | Preserved as `re/targets/rustlab-g1-verified.exe`. Used for benchmark runs 1-3. |
| G0.1-VERIFY canonical build | 5,416,960 B | `80E6AF2442D97CED5E67145D0DF2B9961EF5957ED178049C689457B67CD5CA58` | Preserved as `re/targets/rustlab-g0.1-verified.exe`. Non-reproducible due to timestamp/paths. |
| G0.1-VERIFY pre-clean build | 5,416,960 B | `F6A302EB92DD04CDE4C3B54D9F90E3A7288341737D7089EC5F1341CDDCDEDAC0` | Preserved as `re/targets/rustlab-g0.1-verified-preclean-F6A302EB.exe`. |
| M0 relocation copy (historical) | 5,400,064 B | `02bbda0c68cafe87e01a33fa15f71c2bae96e7cd3f95dafcb82c712380f7460c` | Executable code from `24ac6e2` (`g0`); M0 commit `e84d542` changed docs/config only. Not the current `re/targets/rustlab.exe`. |
| G0.1 metadata rebuild, current `re/targets/rustlab.exe` | 5,400,064 B | `78a31a97b1e73696d5ec2ea434af9878da543bda43d4f058cf5c21a6eb42c3c7` | Built at `95dcbca`; only crate license metadata changed. |
| G0 closeout | 5,400,064 B | `683ea9cbc8bd2b025d93f1ead5281e3a36fbae0c01eae57365ff0681ed6be770` | `24ac6e2`, tag `g0`. |
| B2.1 | 5,393,408 B | `8fd8d28df96d1f36e9dfbd82c4566a26ee00cfee05add359cfb31fe191ff58d3` | Historical. |
| A3.1.4 intermediate | 5,177,856 B | `5F12F38994E80D2BC910CD8D36D611DE78CF4CD9004B2EC6DF759A11BFED0288` | Historical. |

### REA

- REA Project Root: `C:\Users\varun\Desktop\MATSTUDIO\re\projects`
- REA was executed in Antigravity on target `re/targets/rustlab-g1-verified.exe`:
  - `open_binary`: validated session SHA-256 matches shell hash `8036F501...`.
  - `inspect_pe_imports`: verified 15 system DLLs and 154 functions; zero networking imports (`ws2_32.dll` absent). Confirmed binary hygiene / build facts only (not compatibility evidence).
  - `close_binary`: target closed cleanly.
  - *Open R-check items:* Disassembly and decompiler inspection of hot loops (elementwise add, GEMM, FFT) and public `Value::add` vs interpreter fast path remain OPEN due to lack of ARM64 Ghidra provider support in REA.

---

## 3. Milestones Completed

- **Milestone 7**: Baseline foundation (68 tests)
- **Milestone A1**: Language, closures, string library (91 tests)
- **Milestone A2**: Native symmetric eig, svd, expm, fft (96 tests)
- **Milestone A3**: Integer classes, CSC sparse, N-D tensor ops (111 tests)
- **Milestone A3.1 - A3.1.4**: Corrective and integrity passes, zero unwrap policy, test manifest check (153 tests)
- **Milestone B2.1 - B2.1.1**: Clean-room statistics, RFC 4180 CSV matrix I/O, display goldens, MATLAB percentile rule, singular matrix warning + Inf (196 tests)
- **Milestone G0**: B2.1.1 closeout, performance contracts, error ID standardization (`RUSTLAB:`), omitnan core migration (196 tests)
- **Milestone M0**: Relocate project to Desktop\MATSTUDIO, verify 196/196/195 tests from cargo clean, configure re\targets and re\projects, record binary hash.
- **Milestone G0.1**: Unverified baseline only. Added owed-items review, a REA R-check claim, a REPL transcript at the wrong path, and a release benchmark triplet; rejected by the owner as not sufficient evidence.
- **Milestone R0**: Added `docs/ROADMAP.md` with status from git tags; recorded next milestone as G0.1. Documentation only; no feature code.
- **Milestone G0.1-VERIFY**: Verification-only closeout. Added nine missing regressions, settled the mask-count error id, corrected display rules and LU backlog notes, reconciled binary hashes, re-ran release benchmarks with a 1,000,000-row CSV and single-thread GEMM proof, regenerated the 19-prompt REPL transcript at `bench-results/g0.1/repl.txt`, and passed `scripts/verify.ps1` from `cargo clean` at 205/205/204. Tag: `g0.1-verified`.
- **Milestone G1**: Bit-for-bit reproducible release build (0-byte diff across clones), stats routine optimizations with 7 reference tests (212/212/211 tests), REA PE import hygiene audit, cargo-deny license verification, release benchmark triplet with detailed min/median/spread reporting. Benchmark results: LU MET, movmean MISSED (regressed to 16.739 ms), cummax UNSTABLE (16.8 ms spread), A+B MISSED on cross-run median (3.533 ms), zscore MARGINAL (10.197 ms). Misses logged to `docs/BACKLOG.md` §3. Tag: `g1-verified`, fix commit: `ca3c325` (`g1-fix`).
- **Branch `wip/backend-expansion`**: Unified Rust backend expansion across 8 scientific computing capability areas + MatRust Studio desktop IDE. Implemented 32 new clean-room builtins (`roots`, `polyval`, `polyfit` with QR decomposition, `trapz`, `cumtrapz`, `gradient`, `interp1`, `readtable`, `writetable`, `head`, `tail`, `summary`, `tf`, `step`, `filter`, `filtfilt`, `hamming`, `hann`, `blackman`, `xcorr`, `rgb2gray`, `imresize`, `imfilter`, `fitlm`, `predict`, `kmeans`, `confusionmat`, `rotx`, `roty`, `rotz`, `trotx`, `troty`, `trotz`, `transl`, `fkine_2link`, `ikine_2link`, `jtraj`). Added extensive architectural specifications under `docs/backend/` and `docs/compatibility/`. Total tests: 240/240/240. Manifest integrity: 212/212 preserved. Status: in progress on branch `wip/backend-expansion`.
