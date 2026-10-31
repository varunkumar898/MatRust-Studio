# MatRust Studio (RustLab)

[![CI](https://github.com/varunkumar898/MatRust-Studio/actions/workflows/ci.yml/badge.svg)](https://github.com/varunkumar898/MatRust-Studio/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> **Notice**: MatRust Studio is an independent clean-room implementation and is not affiliated with, endorsed by, or sponsored by The MathWorks, Inc. MATLAB® is a registered trademark of The MathWorks, Inc.

**MatRust Studio** (powered by the **RustLab** engine) is a clean-room, high-performance, MATLAB-compatible numerical computing engine and App Designer runtime written from scratch in idiomatic Rust. It features a complete pipeline: Lexer & Pratt Parser $\rightarrow$ Interpreter & Scope Manager $\rightarrow$ Pure Column-Major Tensor Math Core $\rightarrow$ Standard Library $\rightarrow$ Level 5 `.mat` File I/O $\rightarrow$ Interactive REPL & CLI $\rightarrow$ App Designer (`.mlapp`) Container & UI Host.

---

## Workspace Architecture

The project is structured as a modular Cargo workspace of 6 crates with zero cyclic dependencies:

```
rustlab/
├── crates/
│   ├── rustlab-parse/    # Lexer & Pratt Parser (recursive descent, 12 precedence levels, dynamic end)
│   ├── rustlab-math/     # Pure column-major DenseArray<T>, broadcasting, 1-based indexing, Value enum
│   ├── rustlab-eval/     # Tree-walking AST interpreter, lexical environment, multiple outputs, control flow
│   ├── rustlab-stdlib/   # 8 tiers of built-in functions (linear algebra, FFT, statistics, containers)
│   ├── rustlab-matio/    # Clean-room MAT-file Level 5 reader & writer (zlib compression, SDE, complex)
│   └── rustlab-cli/      # Interactive REPL (rustyline), diagnostic printer, script runner, benchmarks
└── docs/
    ├── architecture.md   # Architectural design choices and crate justifications
    ├── re-notes/         # Interoperability specifications (.mat v5/v7 format, MEX C API)
    └── spec/             # Formal grammar, type system, broadcasting rules, error catalogue
```

---

## Key Technical Highlights

1. **Clean-Room Development**: Designed and implemented independently based strictly on mathematical definitions, official documentation, and public file format specifications. Zero copied proprietary or GPL code.
2. **Column-Major Multi-Dimensional Storage (`DenseArray<T>`)**: Memory layout exactly matches Fortran and MATLAB, enabling zero-copy matrix operations and high cache locality.
3. **Zero-Copy Coordinate Broadcasting**: Implicit dimension expansion supports arbitrary dimensional broadcasting using stride-0 coordinate wrapping.
4. **1-Based Indexing with Dynamic `end` Resolution**: Expressions like `v(2:end-1)` resolve contextually based on the active tensor dimension.
5. **Level 5 & v7 `.mat` Serialization**: Full binary deserializer and serializer supporting 128-byte headers, Small Data Elements (SDE), zlib compression, and IEEE-754 complex double matrices.
6. **Compiler-Grade Error Diagnostics**: Syntax and runtime errors report exact line, column, and caret pointers:
   ```
   Error: Undefined function or variable 'unknown_var'.
     --> Line 1, Column 17:
      |
     1 | x = 10; y = x + unknown_var;
      |                 ^^^^^^^^^^^
   ```

---

## Verification & Test Suite

All 59 unit, integration, and property tests pass across all workspace crates with zero warnings:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

---

## CLI Usage

### Interactive REPL
```bash
cargo run --bin rustlab
```

```matlab
>> A = [1, 2; 3, 4]
A =

    1.0000    2.0000
    3.0000    4.0000

>> b = [5; 6]
b =

    5.0000
    6.0000

>> x = A \ b
x =

   -4.0000
    4.5000

>> whos
  Name             Size             Bytes        Class           
  ------------------------------------------------------------
  A                2x2              32           double          
  ans              2x1              16           double          
  b                2x1              16           double          
  x                2x1              16           double          
```

### Inline Evaluation
```bash
cargo run --bin rustlab -- -e "A = magic(3); disp = A * 2"
```

### Execute MATLAB Script
```bash
cargo run --bin rustlab -- script.m
```

### Run Performance Benchmarks
```bash
cargo run --release --bin rustlab -- bench
```

**Measured Benchmark Throughput (Release Mode, Windows ARM64):**
- **Matrix Multiplication (2D Gemm):** **11.82 GFLOPS** (128x128 in 0.355 ms)
- **Linear System Solve ($A \setminus b$):** 100x100 system in **0.487 ms**
- **Fast Fourier Transform (1D FFT):** 4096 points in **936 µs**
- **Elementwise Operations (1M elements):** **45 - 73 Million elements/sec**

---

## MATLAB App Designer Runtime (`mlapp-rs`) Feature Matrix

> [!NOTE]
> **Clean-Room & Real-App Status**: 0 real apps tested directly in repository fixtures to preserve strict clean-room legal isolation. When external apps are supplied via the `MLAPP_REAL_APPS_DIR` environment variable, `mlapp-cli` will inspect and run them. All repository fixture packages (`r2018a_sample.mlapp`, `r2020b_sample.mlapp`, `r2021a_sample.mlapp`, `r2023b_sample.mlapp`, `hazardous_sample.mlapp`) are clean-room synthesized test packages conforming strictly to **ISO/IEC 29500-2:2021 (Open Packaging Conventions)** and public component hierarchies documented in the MathWorks Help Center ("App Designer Components", https://www.mathworks.com/help/matlab/creating_guis/).
>
> **Explicitly Unimplemented Features**:
> The following components are **NOT IMPLEMENTED**:
> 1. **Octave Bridge**: Subprocess bridge to `octave-cli` is NOT implemented.
> 2. **Python Bindings (PyO3)**: Native PyO3 extension is NOT implemented (C ABI with integer handles available via `mlapp-ffi`).
> 3. **Fuzz Testing (`cargo-fuzz`)**: LibFuzzer targets are NOT implemented.

| Capability Area | Feature | Status | Proving Test | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Container & Security** | Zip-Slip path sanitization | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_zip_slip_protection` | Rejects `..`, root `/`, and drive prefixes |
| **Container & Security** | Zip-Bomb compression ratio caps | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_zip_bomb_protection` | Rejects compression ratios > threshold |
| **Container & Security** | Actual decompressed byte accounting | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_decompressed_bytes_limit_with_forged_header` | Enforces `.take(limit)` on stream; catches forged header sizes |
| **Container & Security** | Maximum archive file quota | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_file_count_limit` | Default 2,048 file cap |
| **Container & Security** | Entry role & `.mat` detection | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_inspect_mlapp_structure` | Flags binary data as `unsupported-binary` |
| **Container & Security** | Installer `.mlappinstall` discovery | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_inspect_mlappinstall_manifest` | Extracts title, author, version, toolboxes |
| **Container & Security** | Opt-in Real-App Validation | **Implemented** | [container_tests.rs](crates/mlapp-container/tests/container_tests.rs) `test_opt_in_real_apps_validation` | Discovers and validates all `.mlapp` files when `MLAPP_REAL_APPS_DIR` is set |
| **UI Model** | App Designer XML parser | **Implemented** | [ui_tests.rs](crates/mlapp-ui/tests/ui_tests.rs) `test_parse_app_xml_component_tree` | Parses `UIFigure`, `Panel`, `Button`, `UIAxes` |
| **UI Model** | XML nesting depth limit | **Implemented** | [ui_tests.rs](crates/mlapp-ui/tests/ui_tests.rs) `test_xml_nesting_depth_limit` | Rejects > 1,000 nested XML elements with `MaxDepthExceeded` |
| **UI Model** | Bottom-Left to Top-Left geometry | **Implemented** | [ui_tests.rs](crates/mlapp-ui/tests/ui_tests.rs) `test_coordinate_transformation_bidirectional` | Tested bidirectional round-trip |
| **UI Model** | Top-level UIFigure geometry | **Implemented** | [ui_tests.rs](crates/mlapp-ui/tests/ui_tests.rs) `test_top_level_figure_geometry_exact_values` | Exact window-local `(0,0,800,600)` & desktop display coordinates |
| **UI Model** | Property read/write & validation | **Implemented** | [ui_tests.rs](crates/mlapp-ui/tests/ui_tests.rs) `test_parse_app_xml_component_tree` | Type mismatch checking on properties |
| **Language Frontend** | CDATA MATLAB code extraction | **Implemented** | [lang_tests.rs](crates/mlapp-lang/tests/lang_tests.rs) `test_extract_code_from_xml` | Extracts classdef from `matlab/document.xml` |
| **Language Frontend** | Classdef properties/methods AST | **Implemented** | [lang_tests.rs](crates/mlapp-lang/tests/lang_tests.rs) `test_parse_app_classdef_structure` | Extracts `properties`, `methods`, `startupFcn` |
| **Language Frontend** | Static compatibility analyzer | **Implemented** | [lang_tests.rs](crates/mlapp-lang/tests/lang_tests.rs) `test_static_compatibility_scanner` | Flags `eval`, `system`, computes 0-100 score |
| **Language Frontend** | Deterministic check-compat snapshot | **Implemented** | [lang_tests.rs](crates/mlapp-lang/tests/lang_tests.rs) `test_check_compat_deterministic_snapshot` | Sorted output verified with `insta` snapshot testing |
| **Language Frontend** | Parser nesting recursion limit | **Implemented** | [parser_tests.rs](crates/rustlab-parse/tests/parser_tests.rs) `test_parser_nesting_depth_limit` | Handles 100k nested parentheses without stack overflow |
| **Language Frontend** | Hazardous fixture test | **Implemented** | [fixture_tests.rs](crates/mlapp-container/tests/fixture_tests.rs) `test_hazardous_fixture_compatibility` | Verifies score <= 50 and hazardous warnings |
| **Language Frontend** | Real fixture callback inspection | **Implemented** | [fixture_tests.rs](crates/mlapp-container/tests/fixture_tests.rs) `test_fixtures_generation_and_inspection` | Verifies functions: `linspace`, `sin`, `plot`, `disp` |
| **Runtime & Execution**| `app` handle object semantics | **Implemented** | [runtime_tests.rs](crates/mlapp-runtime/tests/runtime_tests.rs) `test_app_runtime_lifecycle_and_mutations` | Mutates `app.RunButton.Text`, syncs to handle |
| **Runtime & Execution**| UI mutation event draining | **Implemented** | [runtime_tests.rs](crates/mlapp-runtime/tests/runtime_tests.rs) `test_app_runtime_lifecycle_and_mutations` | Drains `UiMutation::Alert`, `UiMutation::SetProperty` |
| **Runtime & Execution**| Fail-loud unsupported diagnostics | **Implemented** | [runtime_tests.rs](crates/mlapp-runtime/tests/runtime_tests.rs) `test_unsupported_function_diagnostics` | Reports unsupported function name with line/column diagnostic |
| **Runtime & Execution**| Comprehensive sandbox policies | **Implemented** | [runtime_tests.rs](crates/mlapp-runtime/tests/runtime_tests.rs) `test_comprehensive_runtime_sandbox_policies` | Intercepts `system`, `eval`, `feval('system')`, `str2func`, `fopen`, `rmdir`, `urlread`, `java`, `python` |
| **Runtime & Execution**| Cooperative cancellation & timeout | **Implemented** | [runtime_tests.rs](crates/mlapp-runtime/tests/runtime_tests.rs) `test_cancellation_while_loop_and_long_builtin` | Interrupts infinite `while true` loops and long builtins |
| **Runtime & Execution**| End-to-end callback execution | **Implemented** | [host_tests.rs](crates/mlapp-host/tests/host_tests.rs) `test_end_to_end_callback_with_advanced_matlab_features` | Runs for loop, struct, cell, closure, `cellfun`, `x(end+1)=5`, `A(2,:)=[]`, transpose |
| **Runtime & Execution**| Full statement interpreter | **Partial** | [stdlib_tests.rs](crates/rustlab-stdlib/tests/stdlib_tests.rs) `test_reductions_and_statistics` | Evaluates core statements; complex OOP/listeners missing |
| **Runtime & Execution**| N-D arrays | **Partial** | [stdlib_a3_tests.rs](crates/rustlab-stdlib/tests/stdlib_a3_tests.rs) `test_a3_nd_operations_cat_permute_meshgrid_reshape` | Multidimensional numerical core works; UITable N-D display partial |
| **Runtime & Execution**| Cells & Structs | **Partial** | [stdlib_tests.rs](crates/rustlab-stdlib/tests/stdlib_tests.rs) `test_cell_arrays`, `test_struct_arrays` | Struct dot access works; full cellfun/containers.Map partial |
| **Runtime & Execution**| Anonymous functions | **Partial** | [eval_tests.rs](crates/rustlab-eval/tests/eval_tests.rs) `test_anonymous_functions_and_closures` | Environment capture verified; App Designer event-driven `@(src,evt)` partial |
| **Runtime & Execution**| Try / Catch with MException | **Partial** | [eval_tests.rs](crates/rustlab-eval/tests/eval_tests.rs) `test_try_catch_statements` | `try ... catch ME` syntax works; `MException` object properties missing |
| **Plotting & 2D Graphics**| Renderer-agnostic `AxesModel` | **Implemented** | [plot_tests.rs](crates/mlapp-plot/tests/plot_tests.rs) `test_axes_model_plotting_and_svg_export` | Series primitives (line, scatter, bar), titles, limits |
| **Plotting & 2D Graphics**| Deterministic SVG renderer | **Implemented** | [plot_tests.rs](crates/mlapp-plot/tests/plot_tests.rs) `test_axes_model_plotting_and_svg_export` | Headless SVG output tested |
| **Host & Threading** | Worker thread lifecycle & channels | **Implemented** | [host_tests.rs](crates/mlapp-host/tests/host_tests.rs) `test_app_host_session_lifecycle` | Worker evaluates callbacks, dispatches notifications |
| **Rendering Backends** | Pluggable `RenderBackend` trait | **Implemented** | [render_tests.rs](crates/mlapp-render/tests/render_tests.rs) `test_headless_renderer_tree_walk_and_event_dispatch` | Headless backend tested; egui backend behind feature |
| **Embedding C ABI** | C API headers, null safety & handle IDs | **Implemented** | [ffi_tests.rs](crates/mlapp-ffi/tests/ffi_tests.rs) `test_c_abi_package_and_session_flow_with_handle_ids` | `mlapp.h` header parity, integer IDs, null checks, `catch_unwind` |
| **CLI Tooling** | `inspect`, `extract`, `check-compat`, `run` | **Implemented** | [cli_tests.rs](crates/mlapp-cli/tests/cli_tests.rs) `test_cli_run_command`, `test_cli_run_gui_flag` | Verified inspection, compatibility, execution, and `--gui` flag |
| **Property Testing** | Property-based tests (`proptest`) | **Implemented** | [proptest_math.rs](crates/rustlab-math/tests/proptest_math.rs) `prop_addition_is_commutative` | Tensor addition commutativity and transpose involution properties |
| **Octave Bridge** | Subprocess bridge to `octave-cli` | **Missing** | N/A | Subprocess bridge not implemented |
| **Python Bindings** | PyO3 Python package | **Missing** | N/A | C ABI is available via `mlapp-ffi`; native PyO3 module not implemented |
| **Fuzz Testing** | `cargo-fuzz` targets | **Missing** | N/A | Format validation unit tests present; libFuzzer harness not implemented |
| **Criterion Benches**| Criterion benchmarks for numerical engine | **Implemented** | [a312.rs](crates/rustlab-stdlib/benches/a312.rs) `bench_elementwise_add` | Microbenchmarks for GEMM, FFT, and A+B; container benchmarks missing |
