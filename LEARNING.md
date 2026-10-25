# RustLab Learning and Insights Ledger

This document records architectural, mathematical, and algorithmic learnings accumulated throughout the development of RustLab.

---

## 1. Summary Statistics & Order Statistics (Milestone B2.1)

- **Linear Percentile Interpolation ($h = (N-1)q + 1$)**:
  MATLAB's default `prctile` and `quantile` interpolation rule computes fractional index $h = (N - 1)q + 1$ (1-based), decomposing $h$ into integral floor $k = \lfloor h \rfloor$ and remainder $f = h - k$. The value is evaluated as $x_{(k)} + f \cdot (x_{(k+1)} - x_{(k)})$. For edge conditions $q = 0$ and $q = 1$, the exact boundary order statistics $x_{(1)}$ and $x_{(N)}$ are returned.
- **Mode Tie-Breaking Semantics**:
  When computing `mode(X)` on discrete or real arrays with equal highest frequencies, MATLAB breaks ties by returning the minimum value among the mode candidates. In RustLab's `mode_slice`, sorting values ascending and selecting candidate with `count > best_count || (count == best_count && val < best)` guarantees identical deterministic tie-breaking.
- **Zero-Allocation In-Place Strided Reducers**:
  For N-D array statistics (`median`, `movmean`, `cummax`, `zscore`), fiber extraction along an arbitrary active dimension $d \in \{1, \dots, D\}$ avoids cloning entire sub-arrays by iterating over projected fiber coordinates using column-major stride offsets.

---

## 2. Delimited Matrix I/O & Memory Efficiency (Milestone B2.1)

- **RFC 4180 Parsing & Header Handling**:
  The `csv` crate provides fast streaming parsing with RFC 4180 quote escaping. To properly support MATLAB `readmatrix` options (`'NumHeaderLines'`, `'Delimiter'`, BOM stripping), headers must be consumed at the byte stream level before passing the reader to `csv::ReaderBuilder`.
- **Shortest Round-Trip Decimal Formatting**:
  To guarantee exact round-trip fidelity between `writematrix` and `readmatrix`, floating-point values are rendered using `ryu` (Ryu/Dragonbox algorithm), emitting minimal decimal digits that round back to the exact identical IEEE 754 bit pattern upon re-parsing.
- **Ragged Row Error Diagnostics**:
  When reading delimited files with mismatched column lengths, returning the exact 1-based file line number (`Row L has C fields, expected E`) allows rapid debugging of corrupted data files.

---

## 3. Toolchain & Platform Specifics (Windows ARM64 GnuLLVM)

- **Dynamic Linking of LLVM Unwind Runtime**:
  When executing standalone binaries produced by `aarch64-pc-windows-gnullvm`, `libunwind.dll` must be available on `PATH` if the binary is invoked outside `cargo run`. Prepending the rustlib toolchain bin directory resolves dynamic dependency resolution.
- **Panic = "abort" for Numerical Kernels**:
  Setting `panic = "abort"` in release profile eliminates landing pads and exception handling tables, optimizing register availability and instruction cache locality for tight linear algebra inner loops.
