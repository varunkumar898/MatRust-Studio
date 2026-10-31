# MatRust Studio: MATLAB-Compatible Numerical Platform & App Designer Runtime

[![CI](https://github.com/varunkumar898/MatRust-Studio/actions/workflows/ci.yml/badge.svg)](https://github.com/varunkumar898/MatRust-Studio/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A high-performance, license-free, clean-room numerical computing platform and MATLAB App Designer (`.mlapp` / `.mlappinstall`) runtime implemented in pure, safe Rust.

> **NOTICE**: MatRust Studio and `mlapp-rs` are independent clean-room implementations developed without proprietary MathWorks code. MATLAB is a registered trademark of The MathWorks, Inc. This project is not affiliated with, sponsored by, or endorsed by The MathWorks, Inc.

---

## Architecture Overview

MatRust Studio is architected as a modular Cargo workspace decoupling the core numerical engine from UI execution and rendering:

- **`crates/rustlab-parse`**: Clean-room MATLAB lexical scanner, Pratt parser, and AST with source spans.
- **`crates/rustlab-math`**: Column-major multidimensional tensor storage (`DenseArray<T>`), implicit broadcasting, and unified runtime value model (`Value`).
- **`crates/rustlab-eval`**: Tree-walking AST interpreter, lexical environment, variable scoping, and function handle dispatch.
- **`crates/rustlab-stdlib`**: Standard mathematical library (linear algebra, matrix generators, elementary math, signal processing, machine learning, and robotics).
- **`crates/rustlab-data`**: Clean-room descriptive statistics and RFC 4180 CSV matrix I/O.
- **`crates/rustlab-matio`**: MAT-file (Level 5 / v7 binary) reader and writer.
- **`crates/mlapp-container`**: Secure Open Packaging Conventions (OPC) ZIP archive extractor with Zip-Slip protection, decompression bombs mitigations, and package manifest inspection.
- **`crates/mlapp-ui`**: Component tree hierarchy model, XML parser, and bidirectional Bottom-Left to Top-Left coordinate transformations.
- **`crates/mlapp-lang`**: App Designer classdef extractor, callback scanner, and deterministic compatibility analyzer.
- **`crates/mlapp-runtime`**: Thread-safe App Designer execution engine, `app` handle binding, callback dispatcher, and UI mutation queue.
- **`crates/mlapp-plot`**: Renderer-agnostic 2D plotting model (`AxesModel`), series management, and deterministic SVG generator.
- **`crates/mlapp-host`**: Threaded background worker lifecycle, event loop, and asynchronous callback execution.
- **`crates/mlapp-render`**: Pluggable UI rendering abstraction, egui renderer backend, and headless screenshot verification rasterizer.
- **`crates/mlapp-cli`**: Command-line interface for inspecting, validating, extracting, and executing MATLAB apps.
- **`crates/mlapp-ffi`**: C ABI embedding interface with integer handle safety, string ownership helpers, and `catch_unwind` error isolation.
- **`fuzz/`**: LibFuzzer targets for parser, XML, and ZIP containers with smoke tests.

---

## Key Technical Highlights

1. **Clean-Room Development**: Designed and implemented independently based strictly on mathematical definitions, official documentation, and public file format specifications. Zero copied proprietary or GPL code.
2. **Column-Major Multi-Dimensional Storage (`DenseArray<T>`)**: Memory layout exactly matches Fortran and MATLAB, enabling zero-copy matrix operations and high cache locality.
3. **Zero-Copy Coordinate Broadcasting**: Implicit dimension expansion supports arbitrary dimensional broadcasting using stride-0 coordinate wrapping.
4. **1-Based Indexing with Dynamic `end` Resolution**: Expressions like `v(2:end-1)` resolve contextually based on the active tensor dimension.
5. **Level 5 & v7 `.mat` Serialization**: Full binary deserializer and serializer supporting 128-byte headers, Small Data Elements (SDE), zlib compression, and IEEE-754 complex double matrices.
6. **Compiler-Grade Error Diagnostics**: Syntax and runtime errors report exact line, column, and caret pointers.

---

## Verification & Test Suite

All 309 unit, integration, property, and fuzz smoke tests pass across all workspace crates with zero warnings:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

## CLI Usage

### Inspect App Package
```bash
cargo run -p mlapp-cli -- inspect app.mlapp
```

### Static Compatibility Audit
```bash
cargo run -p mlapp-cli -- check-compat app.mlapp
```

### Run App
```bash
cargo run -p mlapp-cli -- run app.mlapp
cargo run -p mlapp-cli -- run --gui app.mlapp
```

---

## MATLAB App Designer Runtime (`mlapp-rs`) Feature Matrix

> [!NOTE]
> **Clean-Room & Real-App Status**: 0 real apps tested directly in repository fixtures to preserve strict clean-room legal isolation. When external apps are supplied via the `MLAPP_REAL_APPS_DIR` environment variable, `mlapp-cli` will inspect and run them. All repository fixture packages (`r2018a_sample.mlapp`, `r2020b_sample.mlapp`, `r2021a_sample.mlapp`, `r2023b_sample.mlapp`, `hazardous_sample.mlapp`) are clean-room synthesized test packages conforming strictly to **ISO/IEC 29500-2:2021 (Open Packaging Conventions)** and public component hierarchies documented in the MathWorks Help Center ("App Designer Components", https://www.mathworks.com/help/matlab/creating_guis/).
>
> **Explicitly Unimplemented Features**:
> The following components are **NOT IMPLEMENTED**:
> 1. **Octave Bridge**: Subprocess bridge to `octave-cli` is NOT implemented (documented stub).
> 2. **Python Bindings (PyO3)**: Native PyO3 extension is NOT implemented (C ABI with integer handles available via `mlapp-ffi`).

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
| **Rendering Backends** | Headless Screenshot Verification | **Implemented** | [render_tests.rs](crates/mlapp-render/tests/render_tests.rs) `test_headless_screenshot_render` | Rasterizes `UIFigure`, `Panel`, `Button`, `UIAxes` to RGBA buffer |
| **Embedding C ABI** | C API headers, null safety & handle IDs | **Implemented** | [ffi_tests.rs](crates/mlapp-ffi/tests/ffi_tests.rs) `test_c_abi_package_and_session_flow_with_handle_ids` | `mlapp.h` and `mlapp_ffi.h` parity, integer IDs, null checks, `catch_unwind` |
| **CLI Tooling** | `inspect`, `extract`, `check-compat`, `run` | **Implemented** | [cli_tests.rs](crates/mlapp-cli/tests/cli_tests.rs) `test_cli_run_command`, `test_cli_run_gui_flag` | Verified inspection, compatibility, execution, and `--gui` flag |
| **Property Testing** | Property-based tests (`proptest`) | **Implemented** | [proptest_math.rs](crates/rustlab-math/tests/proptest_math.rs) `prop_1_based_indexing_and_column_major_equivalence`, [proptest_parser.rs](crates/rustlab-parse/tests/proptest_parser.rs) `prop_never_panic_on_arbitrary_string` | Tensor addition commutativity, involution, 1-based indexing, and parser properties |
| **Fuzz Testing** | `cargo-fuzz` targets & smoke run | **Implemented** | [fuzz_smoke.rs](fuzz/tests/fuzz_smoke.rs) `test_fuzz_smoke_parser`, `test_fuzz_smoke_xml`, `test_fuzz_smoke_zip` | Targets for parser, XML, and ZIP containers with smoke tests |
| **Criterion Benches**| Criterion benchmarks for container, UI & interpreter | **Implemented** | [host_benches.rs](crates/mlapp-host/benches/host_benches.rs) `bench_container_open`, `bench_ui_xml_parse`, `bench_interpreter_callback` | Container open, UI XML parsing, and interpreter callbacks |
| **Octave Bridge** | Subprocess bridge to `octave-cli` | **Missing** | N/A | Subprocess bridge not implemented (documented stub) |
| **Python Bindings** | PyO3 Python package | **Missing** | N/A | C ABI is available via `mlapp-ffi`; native PyO3 module not implemented |
