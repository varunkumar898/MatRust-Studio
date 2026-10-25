# RustLab Numerical Display & Formatting Specification

This document specifies the exact formatting and display rules implemented in RustLab, matching MATLAB `format short` display conventions.

---

## 1. Scope & Primary Principles

RustLab adheres to the following principles derived directly from official MathWorks MATLAB documentation:
- **MATLAB Documentation: Set Output Display Format (`format`)**: [https://www.mathworks.com/help/matlab/ref/format.html](https://www.mathworks.com/help/matlab/ref/format.html)
- **MATLAB Documentation: Formatting Strings**: [https://www.mathworks.com/help/matlab/matlab_prog/formatting-strings.html](https://www.mathworks.com/help/matlab/matlab_prog/formatting-strings.html)

### Scale Factor Threshold Rules (MathWorks Citation)
As documented by MathWorks for `format short` and `format long`:
> *"If the largest element of a matrix is larger than 1000 or smaller than 0.001, MATLAB applies a common scale factor for the short and long formats."*

Specifically:
1. **Format Short Range**: Fixed-point format with 4 decimal places is used for values where the maximum magnitude is in the interval $[10^{-3}, 10^{3})$ (or up to $10^{4}$ for single scalar values without scientific exponent).
2. **Lower Threshold ($< 10^{-3}$)**: Values strictly smaller than $0.001$ ($10^{-3}$) switch to scientific notation (e.g. `1e-16` displays as `1.0000e-16`, `5e-4` displays as `5.0000e-04`).
3. **Upper Threshold ($\ge 10^{3}$ for matrices, $\ge 10^{4}$ for non-integer scalars)**: When values exceed this threshold, a common scale factor is factored out for the entire matrix (e.g. `1.0e+04 *`), or non-integer scalars display in signed 2-digit zero-padded scientific notation (`1.2346e+04`). Exact integer doubles below $10^{10}$ display full digits without decimal points.

---

## 2. Exponent Layout and Number Conventions

1. **Exponent Layout**:
   - The exponent is always signed (`+` or `-`) and padded to at least 2 digits (e.g. `5.0000e-04`, `1.2346e+04`).
   - Small values such as `1e-16` print as `1.0000e-16` (not `0.0000`).

2. **Matrix Formatting & Scale Factor Invariant**:
   - **Unified Column Width**: All columns within a given matrix share a uniform column width to guarantee aligned vertical columns.
   - **Common Scale Factor**: When an array contains values exceeding $10^3$ or requiring scaling, a single common scale factor is factored out for the **entire matrix** (never per column), printed as:
     ```matlab
       1.0e+04 *

         1.2346    0.0001
     ```
   - In mixed matrices where the maximum magnitude is within $[10^{-3}, 10^3)$, no scale factor is extracted; tiny components display as `0.0000` (e.g. `[1e-16 1]` prints as `0.0000       1`).
   - Pure integer matrices (e.g. `[1 1000000]`) print with integer columns aligned without decimal points.
   - For string conversions `num2str`: Row vectors join entries with two spaces (`"10000  25000"`), and multi-row matrices right-justify columns separated by two spaces, matching MATLAB `num2str` row formatting.

3. **Negative Numbers & Sign Alignment**:
   - Leading space gutters accommodate minus signs so negative numbers align cleanly with positive entries in the same column.

4. **Special Values**:
   - `NaN` prints as `NaN`.
   - `Inf` prints as `Inf`, `-Inf` prints as `-Inf`.

---

## 3. Diagnostics & Caret Placement

Binary operator syntax errors (such as type mismatch in `a + b` or dimension mismatch in `A * B`) span the full binary expression `left op right`, placing the CLI caret `^` across the exact span of the operator and its operands.

---

## 4. Verified Golden Test Matrix

| Expression | Expected Output | Citation / Rule |
| :--- | :--- | :--- |
| `1e-16` | `   1.0000e-16` | Tiny value < 1e-3 switches to scientific |
| `5e-4` | `   5.0000e-04` | 2-digit zero-padded signed exponent |
| `12345.678` | `   1.2346e+04` | Value $\ge 10^4$ switches to scientific |
| `100000` | `    100000` | Integer below $10^{10}$ preserves full digits |
| `1e10` | `   1.0000e+10` | Value $\ge 10^{10}$ switches to scientific |
| `[1e-16 1]` | `    0.0000       1` | Common scale factor rule across whole matrix |
| `[1 1000000]` | `          1  1000000` | Integer vector alignment |
| `[12345.678 1]` | `  1.0e+04 *\n\n    1.2346  0.0001` | Factored common scale factor |
