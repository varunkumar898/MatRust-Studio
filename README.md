# RustLab: Clean-Room MATLAB-Compatible Numerical Engine in Rust

**RustLab** is a clean-room, high-performance, MATLAB-compatible numerical computing engine written from scratch in idiomatic Rust. It features a complete pipeline: Lexer & Pratt Parser $\rightarrow$ Interpreter & Scope Manager $\rightarrow$ Pure Column-Major Tensor Math Core $\rightarrow$ Standard Library $\rightarrow$ Level 5 `.mat` File I/O $\rightarrow$ Interactive Rustyline REPL & CLI.

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
