# RustLab Numerical Display & Formatting Specification

This document specifies the exact formatting and display rules implemented in RustLab, matching MATLAB® `format short` display conventions.

---

## 1. Scope & Primary Principles

RustLab adheres to the following principles derived from public MATLAB documentation (*MATLAB Documentation: Set Output Display Format* and *Formatting Strings*):

1. **Format Short General Rule**:
   - Fixed-point format with 4 decimal places for non-integers in the range $[10^{-3}, 10^{4})$.
   - Automatic scientific notation switch (`1.xxxx e[+-]yy`) when values have magnitude $< 10^{-3}$ or $\ge 10^{4}$ (except for exact integers below $10^{10}$).
   - Exact integer doubles below $10^{10}$ display with their full integer digits and no decimal point (e.g. `100000`, `1 4 9`).
   - Integer doubles $\ge 10^{10}$ transition to scientific notation (`1.0000e+10`).

2. **Exponent Layout**:
   - The exponent is always signed (`+` or `-`) and padded to at least 2 digits (e.g. `5.0000e-04`, `1.2346e+04`).
   - Small values such as `1e-16` print as `1.0000e-16` (not `0.0000`).

3. **Matrix Formatting & Scale Factor Invariant**:
   - **Unified Column Width**: All columns within a given matrix share a uniform column width to guarantee aligned vertical columns.
   - **Common Scale Factor**: When an array contains values exceeding $10^4$ or non-integers requiring scaling, a single common scale factor is factored out for the **entire matrix** (never per column), printed as:
     ```matlab
       1.0e+04 *

         1.2346    0.0001
     ```
   - In mixed matrices where the maximum magnitude is within $[10^{-3}, 10^4)$, no scale factor is extracted; tiny components display as `0.0000` (e.g. `[1e-16 1]` prints as `0.0000       1`).
   - Pure integer matrices (e.g. `[1 1000000]`) print with integer columns aligned without decimal points.

4. **Negative Numbers & Sign Alignment**:
   - Leading space gutters accommodate minus signs so negative numbers align cleanly with positive entries in the same column.

5. **Special Values**:
   - `NaN` prints as `NaN`.
   - `Inf` prints as `Inf`, `-Inf` prints as `-Inf`.

---

## 2. Diagnostics & Caret Placement

Binary operator syntax errors (such as type mismatch in `a + b` or dimension mismatch in `A * B`) span the full binary expression `left op right`, placing the CLI caret `^` across the exact span of the operator and its operands.

---

## 3. Verified Golden Test Matrix

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
