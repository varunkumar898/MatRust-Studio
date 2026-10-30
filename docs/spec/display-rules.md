# RustLab Numerical Display & Formatting Specification (B2.1.1)

This document specifies the exact formatting and display rules implemented in RustLab, matching
MATLAB `format short` display conventions.

**Verification key.** Every rule below carries either a source URL or the word **unverified**.
Tests that assert an unverified rule carry `_unverified` in their test name.

---

## 1. Source documents

- MathWorks, *Set Output Display Format* (`format`):
  <https://www.mathworks.com/help/matlab/ref/format.html>
- MathWorks, *Formatting Strings* (`num2str`, `sprintf`):
  <https://www.mathworks.com/help/matlab/matlab_prog/formatting-strings.html>
- MathWorks, `num2str` reference:
  <https://www.mathworks.com/help/matlab/ref/num2str.html>

The sentence on the `format` page directly fixing the scale threshold states:
> *"If the largest element of a matrix is larger than 1000 or smaller than 0.001, MATLAB applies a
> common scale factor for the short and long formats."*

---

## 2. Matrix Common Scale Factor vs Per-Element E-Notation Rules

For a 2D matrix of floating-point numbers:
1. **All-Integer Matrix (`is_all_int`)**:
   - If every finite element has zero fractional part (`v.fract() == 0.0`), all elements display as bare integers with uniform column width and right-alignment (e.g. `[1 1000000]` prints as `        1  1000000`).
   - No decimal points and no common scale factors are emitted.
   - Status: **unverified** (MATLAB integer matrix display convention).

2. **Mixed Matrix with Non-Integers (`!is_all_int`)**:
   Let `max_finite = max(|x|)` across all finite entries, and `min_finite_nonzero = min(|x|)` across all non-zero finite entries.
   The dynamic range ratio is defined as `dyn_range = max_finite / min_finite_nonzero`.

   - **Per-Element E-Notation**:
     When `dyn_range >= 1e9` (extreme dynamic range) OR when the matrix has no common scale factor (`max_finite < 1000.0`) and contains sub-1e-3 non-zero elements (`0 < min_finite_nonzero < 0.001`):
     A common scale factor would either cause small non-zero elements to underflow to `0.0000` or cannot represent the spread.
     Therefore, MATLAB formats **every entry in the matrix in scientific notation** (`format_short_scientific`).
     *Example*: `[1e-16 1]` prints as `    1.0000e-16  1.0000e+00`.
     Status: **unverified** (matches Octave / MATLAB output for high-dynamic-range arrays).

   - **Common Scale Factor (`1.0e+NN *`)**:
     When the dynamic range is within standard precision (`dyn_range < 1e9`), and the largest element satisfies `max_finite >= 1000.0` or `0 < max_finite <= 0.001`:
     A common multiplier `1.0e+NN *` (where `NN = floor(log10(max_finite))`) is printed above the matrix.
     Each element is scaled by `10^NN` and formatted.
     *Example*: `[0.001 1000]` has `max_finite = 1000.0`, triggering `  1.0e+03 *\n\n` followed by `    0.0000  1.0000` (one shared column width; two spaces between cells).
     Status: Source for threshold: <https://www.mathworks.com/help/matlab/ref/format.html>; boundary inclusive check `max >= 1000` is **unverified**. The exact column spacing is also **unverified** and is locked by `test_g01_display_scale_factor_0_001_1000_golden_unverified`.

   - **Fixed-Point Mixed Matrix**:
     When neither e-notation nor a scale factor is triggered:
     - Exact zero (`0.0` or `-0.0`) prints as bare `"0"`.
     - All other elements (including integer-valued numbers like `22.0`) print with 4 decimal places (`22.0000`).
     *Example*: `x = [1.5 22]` prints as `    1.5000   22.0000`.
     *Example*: `x = [1.5 0]` prints as `    1.5000        0`.
     Status: **unverified** (MATLAB fixed-point mixed-matrix alignment convention).

---

## 3. Scalar Display Rules (`format short`)

| Value Range | Output Format | Example | Status / Citation |
| :--- | :--- | :--- | :--- |
| Integral doubles with `\|x\| < 1e10` | Full integer digits, no decimal point | `100000` -> `    100000` | **unverified** |
| Integral doubles with `\|x\| >= 1e10` | Scientific notation | `1e10` -> `   1.0000e+10` | **unverified** |
| Non-integral scalars with `0 < \|x\| < 0.01` | Scientific notation (4 decimals, signed 2-digit exp) | `0.001` -> `   1.0000e-03`<br>`0.0012` -> `   1.2000e-03` | **unverified** |
| Non-integral scalars with `0.01 <= \|x\| < 1000` | Fixed point with 4 decimal places | `0.01` -> `    0.0100`<br>`3.14159` -> `    3.1416` | Source: <https://www.mathworks.com/help/matlab/ref/format.html> |
| Non-integral scalars with `1000 <= \|x\|` | Scientific notation | `1234.5678` -> `   1.2346e+03`<br>`12345.678` -> `   1.2346e+04` | **unverified** |
| `NaN` / `Inf` / `-Inf` | Standard literals | `NaN`, `Inf`, `-Inf` | **unverified** |

---

## 4. Vector and Array Reshaping & Orientation

- `unique(x)` for a row vector `1 x N` returns a row vector `1 x M` (`DenseArray::row_vector`).
  Status: MathWorks `unique`: *"If A is a row vector, then C is a row vector."* <https://www.mathworks.com/help/matlab/ref/unique.html>.

---

## 5. `num2str` Conversion

- Default `num2str` uses at most 5 significant digits (<https://www.mathworks.com/help/matlab/ref/num2str.html>).
- Follows C `%g` style: removes trailing zeros for non-integers, uses exponential notation outside `[-4, 5)`.
- Multi-row matrices right-align columns separated by 2 spaces.
- Status: **unverified** (%g specification details).

---

## 6. Golden Test Summary Table

| Expression | Expected Output | Rule / Verification |
| :--- | :--- | :--- |
| `0.001` | `   1.0000e-03` | scalar scientific sub-0.01; **unverified** |
| `0.0012` | `   1.2000e-03` | scalar scientific sub-0.01; **unverified** |
| `0.01` | `    0.0100` | scalar fixed point; **unverified** |
| `1234.5678` | `   1.2346e+03` | scalar scientific >=1000; **unverified** |
| `[1.5 22]` | `    1.5000   22.0000` | mixed matrix integer formatted with decimals; **unverified** |
| `[1.5 0]` | `    1.5000        0` | exact zero bare in mixed matrix; **unverified** |
| `[1e-16 1]` | `    1.0000e-16  1.0000e+00` | per-element e-notation due to extreme range; **unverified** |
| `[1 1000000]` | `        1  1000000` | all-integer matrix; **unverified** |
| `[0.001 1000]` | `  1.0e+03 *\n\n    0.0000  1.0000` | common scale factor `1.0e+03`; MathWorks threshold cited, exact layout **unverified** |
