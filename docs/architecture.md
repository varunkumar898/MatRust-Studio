# Architecture — matlab-compatible-rust

## Purpose

An independent, MATLAB-compatible numerical computing environment in Rust.
Release R2026b compatibility research is captured separately in
`docs/reverse-engineering/`. No MathWorks binaries or decompiler output are
used inside this repository; behavior is derived from public documentation,
independently authored test cases, and observable behavior.

## Crate graph

```
diagnostics      Diagnostic { message, span, severity, hint } + render
syntax           token / lexer / ast / parser          (depends on diagnostics)
values           Value { Matrix | String | Boolean } + DenseMatrix   (diagnostics)
linalg           numeric kernels on DenseMatrix        (values, diagnostics)
runtime          interp (evaluator) + builtins         (all of the above)
apps/cli         `matlabr` binary: script | -e | REPL  (runtime, diagnostics)
tests/{language,matrices,compatibility}  integration test crates
```

## Key decisions

- **Interpreter first.** A tree-walking interpreter (no JIT) while language
  semantics are still moving. Performance comes later.
- **`DenseMatrix` is row-major internally**, but all surfaces are 1-based,
  MATLAB-style: `A(i,j)`, `A(k)` where linear indices are resolved in
  **column-major** order (consistent with MATLAB).
- **Doubles only (f64).** No complex numbers yet — `sqrt(-1)` yields `NaN`
  (MATLAB returns `0 + 1i`); documented divergence.
- **Logical values are 0/1 doubles** in milestone 1, not a distinct type.
  Documented divergence until a `logical` type lands.
- **Element-wise vs matrix ops are distinct**: `*`, `/`, `^` are matrix
  operations (inner-dimension checks, Gauss-Jordan inverse), `.*`, `./`,
  `.^` are element-wise. `.* ` never falls back to scalar broadcasting.
- **Growth semantics**: assignments outside bounds grow the array
  (2-D: zero-filled rectangle; 1-D: grows to a 1-by-k row vector).
- **Variables shadow builtins.** Bare zero-argument constants (`pi`) resolve
  as identifiers when not shadowed.
- **Errors are `Diagnostic`s** (message + span + optional hint), rendered
  `line:col` with a source caret. Parser/lexer/evaluator never panic on
  bad language input.
- **No unsafe Rust. No workspace dependencies beyond std + thiserror.**

## Boundaries kept explicit

- `linalg` only touches `DenseMatrix` math and numeric error messages.
- `runtime` owns evaluation order, environments, and builtin dispatch.
- `apps/cli` owns I/O only — no evaluation logic.
- Black-box integration tests live under `tests/` packages so the language
  contract (not internals) is what they assert.

## Quality gates

`cargo fmt --check && cargo check --workspace && cargo test --workspace`
must pass before any commit.
