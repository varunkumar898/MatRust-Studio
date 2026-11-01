# Safety Audit (Round 3 review)

Status: **Partial — audited, with residual risks listed at the bottom.**

This audit was produced at commit
`be5d46fb2f209129bd9b8366f9fe848252a937c8` on 2026-11-01.
It does not claim formal verification, Miri coverage, or fuzzing of the C ABI.

## 1. Commands and raw logs

| Check | Command | Raw log |
| :--- | :--- | :--- |
| Production panic/unsafe marker counts | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/count-safety-markers.ps1` | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-safety-marker-counts.log` |
| Every `unsafe` occurrence | `rg -n "\bunsafe\b" crates apps -g '*.rs'` | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-unsafe-sites.log` |
| Panic strategy and mutex poison handling | `rg -n "rust-version\|panic =\|catch_unwind\|lto =\|overflow-checks" Cargo.toml crates/mlapp-ffi/src/lib.rs apps/desktop/src/execution.rs` plus the poison-handling grep shown in the log | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-panic-unwind-mutex.log` |
| cbindgen header is current | `cbindgen --config cbindgen.toml --crate mlapp-ffi --output crates/mlapp-ffi/include/mlapp.h` then `git diff --exit-code -- crates/mlapp-ffi/include/mlapp.h` | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cbindgen-header-check.log` |

The marker script excludes integration tests under `tests/`, benchmarks under
`benches/`, and inline `#[cfg(test)]` modules. Its full output is:

```text
crate,production_unwrap,production_expect,production_panic,production_todo,production_unimplemented,production_unsafe
mlapp-cli,0,0,0,0,0,0
mlapp-container,0,0,0,0,0,0
mlapp-ffi,0,0,0,0,0,8
mlapp-host,0,0,0,0,0,0
mlapp-lang,0,0,0,0,0,0
mlapp-plot,0,0,0,0,0,0
mlapp-render,0,0,0,0,0,0
mlapp-runtime,0,0,0,0,0,0
mlapp-ui,0,0,0,0,0,0
rustlab-cli,0,0,0,0,0,0
rustlab-data,0,0,0,0,0,0
rustlab-eval,0,0,0,0,0,0
rustlab-math,0,0,0,0,0,1
rustlab-matio,0,0,0,0,0,0
rustlab-parse,0,27,0,0,0,0
rustlab-stdlib,0,0,0,0,0,0
desktop,0,2,1,0,0,0
```

The 27 `expect(` hits in `rustlab-parse` are calls to the parser's own
`Parser::expect(TokenKind)` method, not `Option::expect`/`Result::expect`
panics. The two real production `expect` calls are in
`apps/desktop/src/execution.rs`: `expect("just pushed")` after an immediate
`push`, and `expect("Failed to spawn background numerical execution worker
thread")` when starting the worker. The one production `panic!` is the
controlled test-injection arm described below.

## 2. Unsafe inventory

There are **nine production unsafe markers**:

| Location | Count | Why it is unsafe | Evidence / mitigation |
| :--- | ---: | :--- | :--- |
| `crates/mlapp-ffi/src/lib.rs` | 8 | `unsafe extern "C"` functions dereference caller-owned pointers and strings. | Every entry point null-checks or returns an error code, wraps its body in `catch_unwind(AssertUnwindSafe(...))`, and uses `u64` registry IDs rather than raw Rust object pointers. `ffi_tests.rs` covers null/invalid handles and the string ownership flow. |
| `crates/rustlab-math/src/ops.rs` | 1 | One `matrixmultiply::dgemm` call with raw pointers. | Shapes, strides, buffer lengths, zero dimensions, and the output-element cap are checked before the call. A `// SAFETY:` comment lists the invariants. Regression tests: `test_matmul_zero_dimensions_and_mismatch`, `test_matmul_huge_output_is_rejected_without_allocating`, and the matrixmultiply/fallback parity test. |

Two unsafe blocks in tests (`mlapp-ffi/tests/ffi_tests.rs` and
`rustlab-stdlib/tests/numeric_comparison_tests.rs`) are test-only and visible
in the unsafe-sites log.

### dgemm details

`matmul_2d` rejects:

- non-2D inputs;
- inner-dimension mismatch (`MATLAB:innerdim`);
- output sizes above `128 * 1024 * 1024` elements (`MATLAB:pmaxsize`);
- zero rows/columns before entering the kernel;
- `m` or `k` that do not fit in `isize`;
- input slices whose lengths do not match their declared shapes.

Only after those checks does it call `matrixmultiply::dgemm` with
column-major strides. The output vector has exactly `m*n` elements and the
pointer is not retained.

## 3. Panic policy

- `Cargo.toml` release profile uses `panic = "unwind"` (line printed in the
  raw log). This is required for `catch_unwind` in the worker and FFI.
- The desktop worker wraps each request in
  `std::panic::catch_unwind(AssertUnwindSafe(...))`. On a panic it resets the
  session and sends `WorkerResponse::WorkerPanic`; the regression test is
  `test_worker_panic_recovery_and_session_reset`.
- FFI entry points catch panics and convert them to error return codes so a
  Rust panic does not unwind across the C ABI.
- `WorkerRequest::InjectPanicForTesting` is the only deliberate production
  `panic!`. It is reached only through the public, `#[doc(hidden)]`
  `ExecutionClient::inject_panic_for_testing()` hook used by the integration
  test. It is not called by the UI, CLI, or FFI. This is documented rather
  than gated behind a feature because the integration test links the library
  without `cfg(test)`.

## 4. Mutex poisoning

`mlapp-host` and `mlapp-runtime` consistently recover poisoned locks with
`.unwrap_or_else(|poisoned| poisoned.into_inner())`. The desktop worker does
the same for its signal, plot, figure, and breakpoint locks. A few desktop
paths use `if let Ok(...)`, which skips the update if a mutex is poisoned
rather than panicking. No production path calls `.unwrap()` on a mutex lock.

Residual risk: recovering from a poisoned lock assumes the protected data is
still structurally valid. The audit did not prove that invariant for every
desktop lock.

## 5. C ABI header

`crates/mlapp-ffi/include/mlapp.h` is generated by cbindgen 0.29.0 from
`cbindgen.toml` (C language, `MLAPP_H` guard, Doxygen comments, C++ guard,
80-column wrapping). The CI `fmt` job regenerates it and runs
`git diff --exit-code`; a hand edit that is not reflected in the Rust source
fails the job. The alias header `mlapp_ffi.h` includes `mlapp.h`, and
`test_c_header_parity_and_declarations` checks the exported names and
`uint64_t` handle types.

## 6. Residual risks / not claimed

- No Miri, ThreadSanitizer, AddressSanitizer, or fuzzing of FFI pointers.
- No proof that every `unsafe extern` caller follows the documented pointer
  contract; the tests cover the repository's own callers only.
- The desktop worker's `expect` on thread spawn aborts if the OS refuses to
  create the thread; no recovery path exists for that startup failure.
- Mutex-poison recovery does not validate the data inside the lock.
- The controlled panic hook remains reachable from Rust code that can call the
  public `ExecutionClient` API; it is not a network or file-format attack
  surface.
