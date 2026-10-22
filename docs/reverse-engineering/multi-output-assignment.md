# Investigation Record: Multi-Output Assignment

## Feature
Multiple-output assignment (`[X, Y] = f(...)` syntax)

## Investigation question
How does MATLAB handle functions that return multiple values, and how can we implement this in Rust?

## Target binary or application
- MATLAB R2026b: NOT AVAILABLE on this system
- Installer package at `/home/cherry/Downloads/matlab_R2026b_Linux` contains only Network License Manager archives (4.2MB + 10.2MB), no MATLAB executable or numerical engine

## Target component or library
- N/A — MATLAB binary not available

## Target SHA-256
- N/A — target unavailable

## REA tools actually invoked
- `search` (execute mode) — verified tool availability: `open_binary`, `inspect_rust_binary`, `inspect_binary_layout`, `binary_overview`, `close_binary`, `inspect_analysis_view`
- `inspect_rust_binary` attempted on `/home/cherry/MATLAB RUST/target/debug/matrust-studio` (421MB) — BLOCKED by 256MB resource limit (resource_constraint error)

## Analysis providers
- None configured for native MATLAB analysis (no MATLAB target present)

## Evidence identifiers
- No evidence IDs collected (MATLAB target unavailable; REA binary inspection blocked by size limit)

## Observed behavior
- **Parser**: `try_parse_multi_assign_names()` already supported `[X, Y] = f(...)` syntax via `AssignTarget::Multi(Vec<(String, Span)>, Span)` (in `crates/syntax/src/parser.rs`)
- **Interpreter**: `AssignTarget::Multi` previously returned error "Multi-output assignment is not yet supported."
- **linalg kernels**: `meshgrid()` and `ndgrid()` in `crates/linalg/src/arrays.rs` already implemented and return `Vec<DenseMatrix>`
- **Builtins**: `meshgrid` and `ndgrid` not registered in `builtins.rs` before this batch

## Inferred behavior
- MATLAB returns first output when called in single-output context; multi-output when called with `[X, Y] = ...`
- nargout (number of outputs requested) determines how many values the function computes/returns
- For functions like `meshgrid`, MATLAB returns multiple outputs (X, Y, optionally Z)
- For `size(A)` with single output, returns [r, c]; with multi-output returns r, c separately

## Unknown behavior
- Exact MATLAB error messages for multi-output assignment mismatches
- MATLAB's behavior when fewer outputs are requested than the function returns (partial nargout)
- Whether MATLAB errors or silently ignores excess outputs
- MATLAB's `nargin`/`nargout` introspection within user functions

## Limitations
- No MATLAB reference available for differential testing
- Multi-output support currently only handles builtin function calls (not user-defined functions)
- nargout beyond what the function naturally returns is not fully handled
- Only `meshgrid`, `ndgrid`, and `size` have multi-output implementations so far

## Independent Rust implementation plan
1. Add `call_builtin_multi()` to `builtins.rs` returning `Vec<Value>` for multi-output functions
2. Modify `exec_statement()` in `interp.rs` to handle `AssignTarget::Multi` before evaluating the expression
3. Extract function name and args from `Expr::Call` (callee is a String in this AST)
4. Wire meshgrid/ndgrid into both `call_builtin` (single output = first) and `call_builtin_multi` (all outputs)
5. Add size multi-output support (returns r, c separately)
6. Fall back to single-output for non-multi functions (excess outputs get empty matrices)

## Required compatibility tests
- meshgrid_2d: verify X varies along columns, Y varies along rows, correct dimensions
- ndgrid_2d: verify X varies along rows, Y varies along columns
- meshgrid_single_output: verify single output returns first matrix
- multi_output_size: verify [r, c] = size(A) returns correct dimensions
- multi_output_invalid: verify non-call RHS errors, non-multi function fallback

## Confidence levels
- **Observed**: Parser already supported multi-output syntax (verified by code inspection)
- **Documented**: MATLAB multi-output behavior described in public documentation
- **Inferred**: nargout-based output selection (not directly verified against MATLAB)
- **Unknown**: Exact error messages, partial nargout handling
- **Unavailable**: MATLAB binary for differential testing
EOF
echo "created investigation record"