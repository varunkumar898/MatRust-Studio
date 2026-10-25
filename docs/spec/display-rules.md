# RustLab Numerical Display & Formatting Specification

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

The only sentence on those pages that directly fixes numeric thresholds is the `format` quote in
§2. Everything else below is either implemented from the observed MATLAB convention or marked
**unverified** where no primary source spells it out.

---

## 2. Matrix common-scale-factor rule (cited)

MathWorks `format` says:

> *"If the largest element of a matrix is larger than 1000 or smaller than 0.001, MATLAB applies a
> common scale factor for the short and long formats."*
>
> Source: <https://www.mathworks.com/help/matlab/ref/format.html>

Implementation:

1. The decision is made **once for the whole matrix**, never per column or per entry.
2. The comparison is strict: `max(|A|) > 1000`, or `0 < max(|A|) < 0.001`. Exactly `1000` or
   exactly `0.001` does **not** trigger scaling.
3. The scale exponent is `floor(log10(max(|A|)))`; the header is printed as `  1.0e+NN *`
   (signed, at least two exponent digits).
4. Boundary-edge behaviour (`[0.001 1000]`) is **unverified**; the strict-inequality reading above
   is a direct reading of the cited sentence.

## 3. Scalar display rules (`format short`)

| Rule | Status / source |
| :--- | :--- |
| Integer-valued doubles with `|x| < 1e10` print full digits with no decimal point (e.g. `100000`). | **unverified** (convention; `format` page does not state the 1e10 limit). |
| Non-integer scalars with `0 < |x| < 0.001` print scientific notation, 4 fractional digits, signed 2-digit exponent (`1.0000e-16`, `5.0000e-04`). | **unverified** (the page documents matrix scaling, not scalar cut-offs). |
| Non-integer scalars with `0.001 <= |x| < 1000` print fixed point with 4 decimals (`0.0010`, `0.0012`, `999.9000`). | Source for "4 decimal places": <https://www.mathworks.com/help/matlab/ref/format.html>; exact boundary handling **unverified**. |
| Non-integer scalars with `1000 <= |x| < 10000` print scientific notation (`1234.5678` -> `1.2346e+03`). | **unverified**. |
| Non-integer scalars with `|x| >= 10000` print scientific notation (`12345.678` -> `1.2346e+04`). | **unverified**; consistent with the cited matrix threshold. |
| Integral-valued doubles with `|x| >= 1e10` print scientific notation (`1e10` -> `1.0000e+10`). | **unverified**. |
| `NaN` / `Inf` / `-Inf` print as `NaN` / `Inf` / `-Inf`. | **unverified**. |
| Negative values keep the sign and share the positive gutter. | **unverified**. |

## 4. Matrix cell layout

| Rule | Status / source |
| :--- | :--- |
| All columns in one display share a single uniform width. | **unverified**. |
| If no scale factor is active, integer-valued cells print without a decimal point; non-integer cells print 4 decimals. | **unverified**. |
| `[1e-16 1]` does not extract a scale factor (max is 1) and clips the tiny cell to `0.0000`: `    0.0000       1`. | **unverified** (follows from §2 + the mixed-cell rule). |
| `[1 1000000]` is an all-integer matrix and prints full digits in one width: `          1  1000000`. | **unverified**. |
| `[12345.678 1]` extracts `1.0e+04` for the whole matrix: `  1.0e+04 *` then `    1.2346  0.0001`. | **unverified** (threshold is cited; the exact cell rounding is convention). |
| `[0.001 1000]` is exactly on both strict boundaries, so no scale factor is extracted: `    0.0010    1000`. | **unverified**. |

## 5. `num2str` conversion

| Rule | Status / source |
| :--- | :--- |
| Default `num2str` uses at most 5 significant digits (`num2str(pi)` -> `3.1416`). | Source: <https://www.mathworks.com/help/matlab/ref/num2str.html>. |
| Non-integer values follow C `%g`-style formatting: no trailing zeros, fixed notation for exponents in `[-4, 5)`, scientific otherwise (`1.234` -> `1.234`, `56.78` -> `56.78`, `0.001` -> `0.001`, `0.0012` -> `0.0012`, `1e-16` -> `1e-16`, `1234.5678` -> `1234.6`). | **unverified** (the reference says "maximum of 5 significant digits" but does not spell out the `%g` conversion). |
| Multi-row `num2str` right-justifies each column and separates columns by two spaces. | **unverified**. |
| Integer-valued doubles keep full digits (`num2str(100000)` -> `100000`). | **unverified**. |

## 6. Diagnostics and caret placement

Binary-operator errors span the full `left op right` expression; the CLI caret `^` covers that
span. CRLF and LF inputs produce the same line/column bookkeeping. Undefined symbols get a
single caret. These are RustLab diagnostics, not MATLAB display rules; source is the RustLab
parser/evaluator test suite (no external source claimed).

## 7. Golden test matrix

| Expression | Expected output | Rule / verification |
| :--- | :--- | :--- |
| `1e-16` | `   1.0000e-16` | scalar scientific; **unverified** |
| `5e-4` | `   5.0000e-04` | scalar scientific; **unverified** |
| `1234.5678` | `   1.2346e+03` | scalar `[1e3,1e4)` scientific; **unverified** |
| `12345.678` | `   1.2346e+04` | scalar `>=1e4` scientific; **unverified** |
| `0.001` | `    0.0010` | lower boundary fixed; **unverified** |
| `0.0012` | `    0.0012` | fixed; **unverified** |
| `100000` | `    100000` | integer full digits; **unverified** |
| `1e10` | `   1.0000e+10` | integer >=1e10 scientific; **unverified** |
| `[1e-16 1]` | `    0.0000       1` | no common scale; **unverified** |
| `[1 1000000]` | `          1  1000000` | all-integer alignment; **unverified** |
| `[12345.678 1]` | `  1.0e+04 *` + `    1.2346  0.0001` | cited common-scale threshold; cell rounding **unverified** |
| `[0.001 1000]` | `    0.0010    1000` | strict boundary, no scale; **unverified** |
| `num2str([1.234 56.78; 9.1 0.2])` | `1.234  56.78` / `  9.1    0.2` | 5 significant digits cited; `%g` layout **unverified** |
