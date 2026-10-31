# PLAN-B8: MEX C API Compatibility Layer (Roadmap, Analyst Voice)

**Milestone**: B8 (planning document; implementation scheduled "later").
**Author role**: ANALYST (Clean Room 1) — this file records binary-interface facts
and an implementation plan for the IMPLEMENTER (Clean Room 2).

---

## 1. Goal

Provide a drop-in **MEX C API compatibility layer** so that legacy `.mex*a*`
numerical extensions (and their source trees) can be either:

1. **shimmed**: compiled against RustLab's MEX header/ABI and linked into a
   RustLab host process (`mxArray*` shims implemented in Rust, C ABI at the
   boundary), or
2. **ported**: source-level reuse of MEX C code against a RustLab-native Rust API
   with a thin `#[no_mangle] pub extern "C"` adapter.

RustLab itself must **not** load foreign `.mexw64`/`.mexa64` blobs in B8; that is a
future, explicitly out-of-scope capability (F-propagation of proprietary binaries).

---

## 2. Boundary surfaces to implement

### 2.1 `mxArray*` object model (backward-facing)

RustLab value map (Rust side) must be mirrored, not shared, across the C ABI:

| MEX type macro / class | RustLab `Value` variant | Notes |
| :--- | :--- | :--- |
| `mxDOUBLE_CLASS` | `Value::Double(DenseArray<f64>)` | row-major vs col-major must be normalized at the shim |
| complex doubles | `Value::ComplexDouble` | `mxGetPr`/`mxGetPi` return two separate buffers from one view |
| `mxSINGLE_CLASS` | `Value::Single(f32)` | |
| `mxINT8..INT64 / UINT8..UINT64` | `Value::Int8..UInt64` | exact 64-bit preserved (no double round-trip) |
| `mxCHAR_CLASS` | `Value::Char` | UTF-16 conversion at the boundary |
| `mxLOGICAL` | `Value::Logical` | `mxIsLogical` true; scalar `true/false` interop |
| `mxCELL_CLASS` | `Value::Cell` | `mxGetCell` returns `mxArray*` handles; per-cell reference counting |
| `mxSTRUCT_CLASS` | `Value::Struct` | `mxGetFieldNumber/mxGetFieldByNumber` |
| `mxSPARSE_CLASS` | `Value::Sparse` (CSC f64) | `mxGetJc/mxGetIr/mxGetData` map directly |

**Implementation vehicle**: an arena of opaque handles (`*mut mxArray` = index into a
thread-local slab) so C code never touches Rust memory management; `mexAtExit`-style
cleanup drains the slab per call.

### 2.2 C ABI surface (monotone subset, Phase-ordered)

Phase B8.1 (readers only): `mxCreateDoubleMatrix`, `mxGetM`, `mxGetN`, `mxGetPr`,
`mxGetPi`, `mxGetNumberOfElements`, `mxGetSizeof*` family, `mxDuplicateArray`,
`mxDestroyArray`.
Phase B8.2 (writers): `mxCreateNumericArray`, `mxCreateCellArray`,
`mxCreateStructArray`, field access family, `mxSetPr/mxSetData`.
Phase B8.3 (control): `mexFunction` dispatch table, `mexErrMsgIdAndTxt`,
`mexWarnMsgIdAndTxt`, `mexPrintf` (routed to REPL `output_log`), `mexCallMATLAB`,
`mexEvalString`, `mexIsGlobal`, `mexAtExit`.
Phase B8.4 ("later"): `mxGetProperty/setProperty` for handle classes, GPU-class
entry points (`mxGPUCopyFromMxArray`), Fortran `mexgateway` conventions.

### 2.3 Host dispatch

`mex(name)` in-engine: argument marshaling `Vec<Value>` -> `mxArray**`,
call `mexFunction(nlhs, plhs, nrhs, prhs)`, unmarshal `plhs[0..nlhs)` and
convert to `Value`, then free the handle slab. Errors: MATLAB-compatible
identifiers (`MATLAB:mex:invalidInputs`) route through `EvalError::UserError`.

---

## 3. Data-layout invariants the IMPLEMENTER must guarantee

1. **Column-major everywhere**: `mxGetM` is rows; `DenseArray` is already
   column-major — no transposition needed; document `mxGetNzmax` semantics for the
   sparse case (`nnz` upper bound, not `values.len()`).
2. **No aliasing**: `mxDuplicateArray` deep-copies; `mxGetData` borrows with a
   lifetime bound to the call (slab prevents UAF).
3. **Integer exactness**: `int64` lanes must never float through `mxDOUBLE`;
   MEX sources that use `mxGetPr` on an int array get an explicit NaN-policy
   error (`MATLAB:mex:classMismatch`).
4. **Complex layout**: MEX expects interleaved `mxComplexDouble`? NO — MEX uses
   *two* real arrays (`mxGetPr`, `mxGetPi`); the shim must split
   `DenseArray<Complex64>` into two scratch buffers pinned for the call.

---

## 4. Verification plan (per phase)

1. **Golden C suite compiled against the shim** (samples from public MEX docs):
   `arrayProduct.c`, `timestwo.c`, `phonebook.c` (cell/struct/string), `mexatexit`.
   Expected outputs are pinned from public reference behavior, never from
   proprietary binary probing.
2. **Valgrind-class leak audit**: per-phase slab drain assertion in debug builds
   (`debug_assert_eq!(slab.len(), 0)` after `mexReturnValue`).
3. **Interop round-trip with the existing MAT path**: `save` struct -> `mxArray`
   handle view -> `save_mat` byte-identical to the scipy fixtures
   (byte-compare allowed on our own generated files).
4. **Differential vs. native RustLab calls**: `mex('timestwo')` and the native
   `2*x` must agree to the last bit for all numeric classes.

---

## 5. Out of scope (explicitly)

- Loading prebuilt third-party `.mex*` binaries (ABI risk, licensing).
- `gpuArray`/`mexLock`/`utInterp` entry points.
- Java/COM engine API wrappers.

Deliverable order: shim crate `rustlab-mex` (new workspace member) with
`rustlab-mex-c` (C headers + sample gateway) subdirectory, tests as
`crates/rustlab-mex/tests/mex_shim_tests.rs`.
