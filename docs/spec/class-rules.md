# RustLab Numeric Class Rules, Error Identifiers, and Citations

This document documents and cites the sources for RustLab's numeric class combination rules, error messages, and MException error identifiers.

---

## 1. Integer Arithmetic & Combination Rules

### Specification & MATLAB Behavior
- **Rule**: Integers cannot be mixed with integers of a different class (e.g. `int8 + int16` is rejected). Integers can only be combined with integers of the exact same class, or with scalar doubles.
- **Error Message**: `"Integers can only be combined with integers of the same class, or scalar doubles."`
- **Identifier**: `MATLAB:integerTypeMismatch`
- **Source Citation**:
  - *MATLAB Documentation: Arithmetic Operations on Integer Data*: "Integers can only be combined with integers of the same class, or scalar doubles. MATLAB produces an error if you attempt to perform arithmetic with integers of different classes."
  - Verified in GNU Octave: produces an analogous error under `Octave:mixed-type-arithmetic`.

---

## 2. Reshape Dimension Validation & Error Identifiers

| Operation | Identifier | Exact Error Message | Source / Verification |
| :--- | :--- | :--- | :--- |
| Negative dimension `reshape(A, -1, 3)` | `MATLAB:checkdim:badSizeArg` | `"Size arguments must be integer scalars or []."` | MATLAB `reshape` documentation (Verified) |
| Multiple unknown dims `reshape(A, [], [])` | `MATLAB:getReshapeDims:multipleNegative` | `"Only one dimension can be unknown ([])."` | MATLAB `reshape` syntax rules (Verified) |
| Numel mismatch `reshape(A, 2, 2)` for 6 elements | `MATLAB:getReshapeDims:notSameNumel` | `"To RESHAPE the number of elements must not change."` | MATLAB `reshape` documentation (Verified) |
| Zero dimension on non-empty array | `MATLAB:getReshapeDims:notSameNumel` | `"Dimensions must evaluate to the same number of elements."` | MATLAB documentation (Verified) |

---

## 3. Mask Assignment Rules

- **Rule**: When assigning to a logical mask with a non-scalar right-hand side `A(mask) = B`, the number of elements in `B` must equal the number of `true` values in `mask`.
- **Error Message**: `"In an assignment  A(I) = B, the number of elements in B and I must be the same."`
- **Identifier**: `MATLAB:subsassigndimmismatch`
- **Source Citation**: MATLAB Documentation: *Subscripted Assignment Dimension Mismatch*.
