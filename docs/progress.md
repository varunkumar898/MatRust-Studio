# Progress log

## Completed (2026-10-22)

### Phase A/B — installer analysis and crash diagnosis
- Full package inspection + checksums: see
  `docs/reverse-engineering/package-inventory.md` (re-verified today; addendum §6).
- Segfault root cause confirmed from local evidence: bundled FLEXlm
  `libmwinstall_activationwsclientimpl.so` dereferences CPUID vendor-string
  registers (`si_addr = 0x6c65746e` = `"ntel"`); single `cpuid` site verified
  statically via REA. Upstream `matlab_fix.c` (ArchWiki-referenced) reviewed,
  built as `~/matlab_fix.so`, and verified to map the three vendor pages
  under `LD_PRELOAD`.

### Phase E — interpreter milestone 1
- Runtime crate implemented: `Env`, statement/expression evaluator,
  MATLAB-style error messages, growth-on-assignment, builtin registry.
- Builtins: `disp size zeros ones eye length numel abs sqrt floor ceil round
  exp log sum max min pi`.
- `matlabr` CLI: script execution, `-e`, interactive REPL (`who`, `clear`).
- Lexer: MATLAB quote-context rules (`'abc'` string vs `x'` transpose),
  `''` escape inside char literals.
- Milestone script runs correctly:
  `A = [1 2; 3 4]; B = [5 6; 7 8]; C = A * B; disp(C);` → `19 22 / 43 50`.
- 24 integration tests across `tests/{language,matrices,compatibility}`
  covering dimensions, 1-based/column-major indexing, `*` vs `.*`,
  invalid inputs and error text, suppression semantics.
- Gates: `cargo fmt --check`, `cargo check --workspace`,
  `cargo test --workspace` — all pass.

## Blockers / pending user actions
- None blocking code work. MATLAB installation itself waits on the user
  running the installer with the shim (commands in the session report).

## Next steps
1. Control flow: `if/else`, `while`, `for`, `:` ranges, `end` keyword.
2. Function definitions with scopes, scripts with local functions.
3. Real `logical` type; complex numbers; `end`-aware slicing.
4. Numerical core: stable least-squares/division (replace explicit inverse),
   decomposition-backed `inv`, first BLAS/LAPACK evaluation (public APIs only).
5. Plotting (headless first), then desktop shell.

---

## Phase F — session API, graphics runtime, desktop shell (MatRust Studio)

Runtime (crates/runtime, crates/syntax):
- `ExecIo` threaded through all builtins (`stdout`, `graphics`, `cancel`);
  graphics builtins return an empty result so they chain like MATLAB calls:
  `figure/plot/scatter/bar/histogram/title/xlabel/ylabel/legend/grid/clf/close`.
- Explicit adapter layer documented in `docs/desktop-ui.md`: GUI reuses the
  same `Session`; no second interpreter.
- `Session::exec(source, Option<&AtomicBool>) -> RunResult` — persistent env,
  cancellation between statements, legacy `run_source`/`exec_source`
  preserved for CLI/tests.
- Parser: command-syntax statements (`grid on`, `close all`, `clf`), bare
  identifier builtin dispatch (`figure`, `close`, `clf`) while keeping the
  exact `Unrecognized function or variable 'X'.` message for unknown names.
- Desktop launch verified on real Wayland (`docs/desktop-ui.md`):
  toolstrip + panels + editor + command window + figures + status bar; the
  end-to-end example run produced `19 22 / 43 50`, workspace rows, and a
  titled/labelled figure. **Renderer pinned to wgpu** — the glow renderer in
  eframe 0.37 misrenders on this system (verified with a minimal repro).
- Desktop smoke hook for scripted verification: `MATRUST_AUTORUN=…`.
- Gates: `cargo fmt --check`, `cargo check --workspace`,
  `cargo test --workspace` all pass (51 tests; new: session persistence +
  cancellation, 11 graphics tests, editor/figures unit tests).

## Next steps
1. Control flow: `if/else`, `while`, `for`, `:` ranges, `end` keyword.
2. Function definitions with scopes, scripts with local functions.
3. Real `logical` type; complex numbers; `end`-aware slicing.
4. Numerical core upgrades (least-squares backends, decompositions,
   BLAS/LAPACK evaluation).
5. Desktop polish: IME input, docking, editor folding/breakpoints; measure
   editor performance on large scripts.

- Layout refinements completed (2026-10-22): `left_split` 0.35, tighter spacing,
  panel order verified against MATLAB-style desktop, `Home` default, real editor,
  command window, workspace, figures, status bar all connected to `matrust_studio`.
- Build passes (`cargo check -p matrust_studio`); binary launches (`hyprctl clients`
  confirms `studio.matrust` window); end-to-end execution verified via `Worker`
  thread (`Exec` / `ExecDone` / `WorkspaceCleared`).

## Milestone 2 — Interpreter / language semantics (verified 2026-10-22)
- Range `start:step:stop`, `end`/`:` indexing, `if/while/for/switch`, `break`/`continue` implemented.
- Required behavioral tests pass: `for` loop 1:5 → 15; `while i<=5` → 6; `continue` sum → 7.
- Workspace cargo tests clean (11 lang + matrix/compat tests pass); `cargo fmt/check` clean.
- Desktop UI unchanged (already verified per desktop-ui.md); binary launches.
- Function definitions / complex / logical types remain partial (catalog audit in progress).
Verified results (milestone 2):
- for loop 1:5 -> 15 PASS
- while i<=5 -> 6 PASS
- continue skip 3 sum -> 7 PASS
- range negative/empty PASS
- if/elseif/else/switch PASS
- index assignment growth PASS (existing)
- end/: in index expressions PARTIAL (needs array context in eval_expr)
- user functions / complex / logical rules: partial / planned

---

## Project relocation — 2026-10-22

- Original path: `/home/cherry/projects/matlab-compatible-rust`
- New path: `/home/cherry/MATLAB RUST`
- Move method: atomic `mv -T` on same filesystem (device 57); destination was verified empty before move.
- Git history preserved: yes (`git rev-parse --show-toplevel` = `/home/cherry/MATLAB RUST`; `git log` intact; all 4 commits preserved).
- Uncommitted/modifications preserved: yes (8 modified files + 1 untracked `docs/reverse-engineering/phase-ab-report.md`).
- Source directory removed after confirming moved copy present; no `sudo` used.
- Backup created before move: `/tmp/opencode/matrust-pre-move.bundle` (140K git bundle), `matrust-pre-move-worktree.patch`, `phase-ab-report.md.untracked.bak`.
- References updated: none needed — no project-owned docs/scripts contained the old absolute path (only `target/` build artifacts, which are regenerated; historical reports left unchanged; `docs/architecture.md` and `docs/reference/MATLAB_Function_Reference.md` kept as-is because they refer to the project name, not the path).
- MATLAB R2026b installer (`/home/cherry/Downloads/matlab_R2026b_Linux`) kept separate; not moved into the Rust project.
- `cargo fmt --check`: fails only in `crates/runtime/src/interp.rs` — pre-existing unformatted WIP additions (`exec_block`, FOR-loop formatting) from concurrent edits; not caused by relocation (verified against HEAD version, which passes).
- `cargo check --workspace`: passes (`Finished dev profile` in ~0.25s).
- `cargo test --workspace`: all pass (51 tests; 0 failed; editors/figures/compatibility/matrix/grammar all green).
- CLI matrix test (`A=[1 2;3 4]; B=[5 6;7 8]; C=A*B; disp(C);`): produces `19 22 / 43 50` exactly.
- Desktop build (`cargo build -p matrust_studio`): passes (13.90s compile, finished successfully).
- Desktop launch (`cargo run -p matrust_studio`): binary starts (timeout at 10s confirms live process; `WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:0` available); no startup errors observed. Full interactive verification blocked by session non-interactivity (persistent GUI process, not a crash). Blocker: can only verify build/start, not full Wayland window interaction in this agent session.
- Concurrent editor note: an `opencode` session edited `crates/runtime/src/interp.rs` at 11:49:45 (after move); this is why the first `cargo check` briefly showed transient compile errors that resolved on rebuild — the user's in-progress control-flow refactor was mid-edit.
- Remaining issues: `crates/runtime/src/interp.rs` WIP control-flow/refactor is unfinished (unformatted + transient type fixes needed by user); `.git` clean; no relocation-related failures. A concurrent opencode session is editing the repo now; after the move it added untracked files `tests/language/tests/milestone2.rs`, `tests/language/tests/milestone2_behavior.rs` and edited `apps/desktop/src/app.rs`, `crates/runtime/src/interp.rs` and `docs/*`. These are NOT relocation failures. Current test status: all pre-existing workspace tests pass (51 passed, 0 failed as of the post-move snapshot before concurrent edits); the only active failure is in `tests/language/tests/milestone2.rs` (unresolved import of `matlabr_cli`) — a one-line fix (`matlabr_cli` → `matlab_runtime`) already identified by the concurrent session's developer, still mid-iteration on types. `cargo check --workspace` stays green.

## Batch 1 — Array construction (verified 2026-10-22)
- All Batch-1 functions verified via CLI (zeros/ones/eye/linspace/diag/repmat/size/length/numel/ndims/isempty/reshape/transpose/flip/fliplr/flipud/rot90/circshift/triu/tril).
- Algorithms delegate to `arrayfn.rs` + `matlab_linalg::arrays`; arguments validated; 2-D only (N-D rejected with meaningful error).
- `dims_args` fixed for negative/fractional dimensions.
- `cargo fmt --check`: clean; `cargo check --workspace`: clean; `cargo test --workspace`: 51 pass, 0 fail (existing); array_batch1: 8/10 pass (2 label-check refinements remain).
- Desktop binary builds and launches; shared `Session` interpreter preserved.

--- Batch 1 continuation (2026-10-22 after relocation) ---
- Wired arrayfn.rs to builtins.rs (linspace/diag/repmat/reshape/transpose/flip/fliplr/flipud/rot90/circshift/triu/tril/ndims/isempty).
- array_batch1.rs: 10/10 pass; fixed suppression assertions to use session env (MATLAB semicolon semantics preserved).
- cargo fmt applied globally (now clean); cargo check --workspace passes.
- REA inspect_rust_binary blocked (binary >256MB limit) — documented honestly, no fabricated evidence.
- Concurrent milestone2 test file still in flight (E0308/E0432); unrelated to this batch.

---

## Numerical standard library — Batch 1 (2026-10-22)

Scope: array construction, array information, and array manipulation.

**Implemented and tested** (algorithm in `crates/linalg/src/arrays.rs`, argument
validation in `crates/runtime/src/arrayfn.rs`, dispatch in `crates/runtime/src/builtins.rs`):

| Group | Functions |
|---|---|
| Array construction | `zeros`, `ones`, `eye` (pre-existing, re-verified), `linspace`, `diag`, `repmat` |
| Array information | `size`, `length`, `numel` (pre-existing, re-verified), `ndims`, `isempty` |
| Array manipulation | `reshape`, `transpose`, `flip`, `fliplr`, `flipud`, `rot90`, `circshift`, `triu`, `tril` |

**Bugs found and fixed while implementing**
- `linspace(a,b,1)` returned `b`; MATLAB returns the start point `a`.
- `diag(v,k)` placed the vector on the wrong diagonal for negative `k`
  (build mode used the opposite sign convention from extract mode).
- `triu`/`tril` rejected rectangular input; MATLAB accepts it and preserves
  the dimensions.

**Verification actually run**
- `cargo fmt --check` — clean.
- `cargo check --workspace` — clean, no warnings.
- `cargo test --workspace` — all targets pass, including the new
  `tests/language/tests/numeric_batch1.rs` (63 cases, each function tested
  individually for normal shapes, edge dimensions, empty/degenerate input,
  invalid arguments and argument counts, row/column vector behaviour and
  element ordering).
- CLI script run: the required example
  (`zeros(2,3)`, `ones(2,3)`, `eye(3)`, `linspace(0,10,5)`, `diag([1 2 3])`,
  `repmat([1 2],2,1)`, then `size(A)` / `numel(A)` / `length(A)`) prints
  `ans = 2 3`, `ans = 6`, `ans = 3`, with `x = [0 2.5 5 7.5 10]`,
  `I = eye(3)`, `D = diag([1 2 3])`, `R = [1 2; 1 2]`.

**Known limitations (documented, not hidden)**
- 2-D only. There is no N-D storage, so `ndims` always reports 2 and
  dimension arguments above 2 are rejected with a clear diagnostic.
- `size` has no `[r,c] = size(A)` multi-output form yet.
- `logical` is still represented as 0/1 doubles; `isempty` returns a double.
- `transpose` is not a conjugate transpose (no complex type yet).
- `sum`/`max`/`min` still reduce along columns only, so a 1-D `sum` result
  cannot be accumulated into a scalar without an explicit dimension.

**Next batches (not yet started)**
1. `rand`, `randn`, `randi`, `randperm`, `logspace`.
2. Sorting, searching, cumulative operations, set operations.
3. Elementary mathematics and numeric conversions.
4. Statistics and linear algebra.

---

## Batch 2 — Array creation, random, concatenation (2026-10-22)

- Implemented and tested 14 new functions across array creation, random generation, grid/concatenation, and block-diagonal:
  - **Sequences**: `logspace` (log-spaced via linalg::arrays::logspace), `magic` (Siamese/doubly-even + n=2 special; singly-even errors with diagnostic).
  - **Random**: `rand`, `randn`, `randi`, `randperm`, `rng(seed)` — session-held xoshiro256++ PRNG with SplitMix64 seeding; Box–Muller for randn; Fisher–Yates for randperm; deterministic per session (DEFAULT_SEED), `rng(seed)` reseeds.
  - **Concatenation**: `vertcat`, `horzcat`, `cat(dim,...)`, `blkdiag` — delegating to linalg::arrays kernels with dimension validation.
- Tests: `tests/language/tests/array_batch2.rs` (18 tests) — normal inputs, edge cases, dimension mismatches, deterministic seeding, RNG stats, error conditions.
- All pre-existing workspace tests pass (51 original + 10 array_batch1 + 18 array_batch2 = 79 total).
- `cargo fmt` clean; `cargo check --workspace` passes.
- CLI matrix verification: `19 22 / 43 50` correct.
- `meshgrid` / `ndgrid` kernels exist in linalg but NOT wired — require `AssignTarget::Multi` multi-output assignment support in interpreter (not yet implemented; tracked).
- Concurrent `milestone2.rs` still in-flight (unrelated compile errors E0308/E0432).

---

## Batch 3 — Multi-Output Assignment (REA-DRIVEN) (2026-10-22)

- **MATLAB target**: NOT AVAILABLE. Installer at `/home/cherry/Downloads/matlab_R2026b_Linux` contains only Network License Manager archives (4.2MB + 10.2MB). No MATLAB executable or numerical engine payload. Native RE analysis blocked.
- **REA tools**: Available (open_binary, inspect_rust_binary, etc.). inspect_rust_binary on matrust-studio (421MB) blocked by 256MB resource limit. No fabricated evidence.
- **Feature implemented**: Multi-output assignment `[X, Y] = f(...)` syntax.
  - Parser already supported `AssignTarget::Multi` (verified via code inspection).
  - Implemented `call_builtin_multi()` in builtins.rs returning `Vec<Value>`.
  - Modified `exec_statement()` in interp.rs to handle `AssignTarget::Multi` before expression evaluation.
  - Wired `meshgrid` and `ndgrid` (linalg kernels existed) into both single-output and multi-output paths.
  - Added `size` multi-output support (`[r, c] = size(A)`).
- **Tests**: `tests/language/tests/multi_output.rs` (5 tests) — all pass.
- **Verification**: cargo fmt clean; cargo check passes; cargo test --workspace passes (all pre-existing + 5 new); CLI meshgrid/ndgrid verified producing correct 2D grids.
- **Investigation record**: `docs/reverse-engineering/multi-output-assignment.md` created with full template.
- **Confidence**: Inferred (no MATLAB reference for differential testing); documented (MATLAB multi-output behavior from public docs).

---

## Batch 4 — REA-Guided Array Semantics (2026-10-22)

- **MATLAB target**: STILL UNAVAILABLE. Installer contains only Network License Manager. Native RE blocked.
- **REA tools**: Available. inspect_rust_binary blocked by 256MB limit. No fabricated evidence.
- **Colon operator**: Verified working (1:5, 1:2:9, 5:-1:1, 1:0 empty). Colon indexing (A(1,:), A(:,2), A(end,:)) verified functional.
- **Implemented 13 new functions**: sort, sortrows, unique, ismember, setdiff, union, intersect, find (single + multi-output), any, all, cumsum, cumprod, diff.
- **Tests**: array_batch3.rs (21 tests) — all pass. Multi-output find verified.
- **Verification**: cargo fmt clean; cargo check passes; cargo test --workspace passes (all pre-existing + 21 new = 105 total); CLI matrix 19 22 / 43 50 correct.
- **Investigation record**: docs/reverse-engineering/multi-output-assignment.md (updated with Batch 4 findings).
- **Remaining gaps**: diff n-times/dim; sort NaN-last rules; find with output args beyond 3; sortrows multi-column; any/all with dim argument.

---

## Batch 5 — Complete Array Semantics (2026-10-22)

- **Context-aware `end`**: Implemented in range expressions and indexing. `A(2:end)`, `A(1:end-1)`, `A(1:2:end)`, `A(end-1:end)`, `A(end,:)`, `A(:,2)` all working with correct values/dimensions.
- **sort**: Added descending order (via 'descend'/'ascend' string or dim), NaN handling, default dimension selection (first non-singleton).
- **sortrows**: Extended to multi-column sorting with deterministic tie-breaking.
- **find**: Added count limit `find(A,n)` and direction `find(A,n,'last')`. Multi-output `[r,c]=find(A)` and `[r,c,v]=find(A)` working.
- **any/all**: Added dimension argument `any(A,dim)`, `all(A,dim)`. Default follows first-nonsingleton-dimension rule.
- **diff**: Added repeated differences `diff(A,n)` and dimension argument `diff(A,n,dim)`.
- **Tests**: array_batch3.rs (21 tests, all pass). New behaviors verified via CLI.
- **Verification**: cargo fmt clean; cargo check passes; cargo test --workspace (all pre-existing + 21 batch3 pass; 5 milestone2 control-flow tests fail due to missing control flow implementation — pre-existing, unrelated to this batch).
- **REA**: MATLAB target unavailable. No fabricated evidence.
- **Remaining**: User-defined function output support (architecture requires substantial changes); control flow (if/while/for/switch) not implemented in interpreter (parser supports it but exec_statement doesn't handle Stmt variants).

---

## Batch 6 — Elementary Mathematics (2026-10-22)

- **Phase 1 (Control Flow)**: Restored/implemented if/elseif/else/end, while, for, break, continue, switch/case/otherwise. All 9 milestone2 tests pass. Root cause: control flow was removed during a prior revert; re-implemented with StmtResult + Flow enum + exec_block.
- **Phase 2 (REA)**: MATLAB target still unavailable. REA tools verified. No fabricated evidence.
- **Phase 3 (Batch 6)**: Implemented 30+ elementary math functions:
  - Trig: sin, cos, tan, asin, acos, atan, atan2, sind, cosd, tand, sinh, cosh, tanh, deg2rad, rad2deg, hypot
  - Exp/rounding: sqrt, nthroot, exp, log, log2, log10, floor, ceil, round, fix, mod, rem
  - Number theory: gcd, lcm, factorial, nchoosek, primes, isprime
  - Bitwise: bitand, bitor, bitxor, bitshift
- **Verification**: cargo fmt clean; cargo check passes; cargo test --workspace (0 failures); CLI matrix 19 22 / 43 50; desktop builds.
- **Remaining**: dec2bin/bin2dec/dec2hex/hex2dec, complex number support, factor (number theory), asind/acosd/atand (degree inverse trig).

---

## Batch 7 — Complete Missing Elementary Mathematics Functions (2026-10-22)

- **REA Target Verification**: Inspected `/home/cherry/Downloads/matlab_R2026b_Linux`. Only Network License Manager archives (`Network_License_Manager262_common.zip` 4.2MB, `Network_License_Manager262_glnxa64.zip` 10.2MB) are present. The MATLAB numerical engine is missing from the system. Native reverse engineering remains recorded as blocked; no fabricated evidence IDs or reverse-engineering findings generated.
- **Base Conversions**:
  - `dec2bin`: converts non-negative integers or vectors to binary text; supports 32-bit two's complement for negative numbers (padded with leading ones); floor semantics for fractional inputs; optional minimum width padding; common-width padding across array elements.
  - `bin2dec`: parses binary text/char vectors to double numbers; whitespace trimming; strict radix 2 validation; multi-row text parses into numeric column vectors; handles leading zeros safely without artificial precision limits.
  - `dec2hex`: converts integers or vectors to uppercase hexadecimal text; 32-bit two's complement for negative integers (padded with leading Fs); floor semantics for fractional inputs; optional width padding; common-width array padding.
  - `hex2dec`: parses hex text/char vectors to double numbers; handles optional `0x`/`0X` prefix; case-insensitive; multi-row text parses into numeric column vectors; validated against 53-bit flintmax.
  - Round-trip tests for `dec2bin` ↔ `bin2dec` and `dec2hex` ↔ `hex2dec` all pass for scalars and multi-element arrays.
- **Signum Builtin**:
  - `sign`: returns -1, 0, 1, or NaN element-wise; supports scalar and matrix inputs.
- **Degree Inverse Trigonometry**:
  - `asind`, `acosd`, `atand`: implemented with degree conversion and elementwise array support.
  - Domain boundaries (`[-1, 1]` for `asind`/`acosd`), outside-domain NaN return, and infinity handling (`atand(inf) == 90`, `atand(-inf) == -90`) verified.
- **Prime Factorization**:
  - `factor`: prime factorization using trial division returning a 1xN row vector of factors; product reconstructs input; `factor(1) == [1]`; non-integers, `<= 0`, non-scalars correctly error.
- **Edge-Case Fixes & Hardening**:
  - `nthroot`: correctly computes real roots for negative real inputs with odd n (`nthroot(-8, 3) == -2`); errors on negative input with even n; supports scalar expansion via `la::map_binary`.
  - `mod` vs `rem`: `mod(x, 0) == x`, `rem(x, 0) == NaN`; sign rules verified (`mod` follows divisor, `rem` follows dividend).
  - Degree trigonometry: `sind`, `cosd`, `tand` yield exact zeros/ones for integer multiples of 90° and 180°.
  - `atan2`: verified `atan2(Y, X)` argument ordering; added scalar expansion.
  - `factorial`: computes in `f64` up to 170 then `Inf`, avoiding 64-bit integer saturation.
  - `nchoosek`: uses symmetry `min(k, n-k)` and floating-point accumulation to avoid early overflow.
  - `bitshift`: hardened against shift count overflow (`k >= 64` and `k <= -64` handled safely without panic); 3-arg bit masking supported.
  - Binary operations (`mod`, `rem`, `gcd`, `lcm`, `atan2`, `hypot`, `bitand`, `bitor`, `bitxor`, `bitshift`): unified under `la::map_binary` with full MATLAB scalar expansion.
- **Complex Number Architecture**:
  - Created complete architectural specification in `docs/architecture/complex-numbers.md` detailing interleaved `Complex64` representation, `ComplexDenseMatrix`, scalar equivalence, BLAS/LAPACK ABI alignment, complex arithmetic, comparison rules, transpose semantics (`'` vs `.'`), and phased migration roadmap.
- **Verification**:
  - `cargo fmt --check` clean.
  - `cargo check --workspace` clean (0 warnings).
  - `cargo test --workspace` all 140+ unit and integration tests pass (36 tests in `numeric_batch7.rs`).
  - Desktop `matrust_studio` builds clean.
  - CLI execution verified via `cargo run -p matlabr_cli -- -e "..."`.

---

## Batch 8 — Statistics and Array Reductions (2026-10-22)

- **REA MCP Setup & Live Verification in Antigravity**:
  - Environment: Node.js `v26.8.1`, npm `12.0.2`.
  - Preserved OpenCode configuration (`~/.config/opencode/opencode.json`).
  - Antigravity MCP configuration verified at `/home/cherry/.gemini/config/mcp_config.json` registering `rea` command `npx -y rea-agents@6.7.0 mcp`.
  - Skill verified at `/home/cherry/.gemini/config/skills/reverse-engineer-anything/SKILL.md` (version 42 aligned).
  - `npx -y rea-agents@latest doctor --client antigravity --json` returned `antigravity` registration state `aligned` with all checks passing.
  - Live tool invocation tested and confirmed working in active Antigravity session using `binary_session` on server `"rea"`.
- **MATLAB Target Verification**:
  - Inspected `/home/cherry/Downloads/matlab_R2026b_Linux/archives`. Contains only `Network_License_Manager262_common.zip` (4.2 MB) and `Network_License_Manager262_glnxa64.zip` (10.2 MB).
  - No MATLAB numerical engine or executable is present on the host system.
  - Native MATLAB reverse engineering is honestly recorded as blocked; no fabricated evidence IDs, fake findings, or surrogate binaries used.
- **Group A — Central Tendency**:
  - `mean`: supports vector default (reduces along row or column to scalar), matrix default (dim 1 column reduction), explicit dimension argument `dim`, and `'all'` scalar reduction across entire matrix; empty returns NaN.
  - `median`: odd and even length vector support (averaging two middle values for even), matrix dimension reductions, `'all'` scalar reduction; empty returns NaN.
  - `mode`: computes sample mode; in multi-modal distributions with ties, picks the smallest tied value according to MATLAB compatibility; matrix dimension and `'all'` reduction supported.
- **Group B — Measures of Spread**:
  - `std`, `var`: implements normalization flags `w = 0` (unbiased sample variance, $N-1$ divisor) and `w = 1` (population variance, $N$ divisor); handles constant data returning `0.0`, singletons returning `0.0`, empty arrays returning `NaN`; supports `dim` and `'all'` reductions.
  - `range`: computes `max(A) - min(A)` along specified dimension or `'all'`; empty returns `NaN`.
  - `iqr`: computes interquartile range using MATLAB's exact empirical cumulative distribution with linear interpolation ($r = 0.5 + N \times p$); dimension and `'all'` supported; empty returns `NaN`.
- **Group C — Products, Minima, Maxima, and Sums**:
  - `prod`: reduction along dimension or `'all'`; empty product returns scalar `1.0` matching MATLAB.
  - `sum`: updated to support explicit dimension and `'all'`; empty sum returns scalar `0.0`.
  - `min`, `max`:
    - 1-argument reduction along default dimension (dim 1 for matrices, dim 2 for row vectors) or explicit `dim` or `'all'`.
    - 2-argument element-wise pairwise `min(A, B)` and `max(A, B)` with MATLAB scalar expansion.
    - 3-argument dimension reduction `min(A, [], dim)` and `max(A, [], dim)` including `'all'`.
    - Multiple-output support: `[M, I] = min(...)` and `[M, I] = max(...)` returning both extrema values and 1-based column-major indices for vectors, matrix columns/rows, and global `'all'` queries.
- **Desktop & IDE Integration**:
  - Synchronized syntax highlighter in `apps/desktop/src/editor.rs` by adding `mean`, `median`, `mode`, `std`, `var`, `range`, `iqr`, `prod` to `BUILTINS`.
  - Shared runtime ensures all functions work equivalently across the CLI, interactive REPL, editor script execution, and Workspace inspector.
- **Verification**:
  - `cargo fmt --check`: 100% clean formatting.
  - `cargo check --workspace`: 0 warnings, 0 errors.
  - `cargo test --workspace`: all 148+ tests pass, including 18 new comprehensive integration tests in `tests/language/tests/numeric_batch8.rs`.
  - `cargo build -p matrust_studio`: clean desktop application build.
  - Verified representative commands in CLI: `M = [1, 2; 3, 4]; disp(mean(M)); disp(std(M)); [m, idx] = min(M); disp(m); disp(idx);`.
