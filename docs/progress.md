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
