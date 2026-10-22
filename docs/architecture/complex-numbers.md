# Complex Number Value Model — Architectural Specification

## Status: Fully Implemented (Batch 10 Verified)

## 1. Executive Summary

MatRust Studio currently operates exclusively on real-valued IEEE-754 double-precision floating-point numbers (`f64`). When mathematical operations encounter domain boundaries that yield complex numbers in standard MATLAB (such as `sqrt(-1)` or `asin(2)`), the current runtime either yields `NaN` or raises a domain error.

Introducing genuine complex-number support requires extending the runtime value system, linear algebra kernels, lexer/parser, and display formatting while strictly preserving real-valued performance and backwards compatibility.

This document establishes the formal design for complex-number representations and semantics across the MatRust architecture.

---

## 2. Core Value Representation & Memory Layout

### 2.1 The Value Enum

The top-level `Value` enum in `matlab_values` will be updated to distinguish real matrices from complex matrices:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Matrix(DenseMatrix),                 // Real-valued double matrix (f64)
    ComplexMatrix(ComplexDenseMatrix),   // Complex-valued double matrix (Complex64)
    String(String),                      // Character vector
    Boolean(bool),                       // Logical scalar
}
```

### 2.2 Memory Layout: Interleaved Complex

Standard numerical libraries (BLAS/LAPACK `DoubleComplex`, modern MATLAB R2018a+, and NumPy) use **interleaved storage** rather than split real/imaginary arrays.

We define `Complex64` as a contiguous memory block representing `[real: f64, imag: f64]`:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64,
}

impl Complex64 {
    #[inline(always)]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline(always)]
    pub fn is_real(&self) -> bool {
        self.im == 0.0 && !self.im.is_sign_negative()
    }
}
```

And `ComplexDenseMatrix`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexDenseMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Complex64>,
}
```

#### Advantages of Interleaved Layout
- **BLAS/LAPACK ABI Compliance**: `Vec<Complex64>` can be passed directly as `*const c_double_complex` to OpenBLAS, Intel MKL, or Apple Accelerate without copying.
- **Cache Locality**: Complex arithmetic (multiplication, division) accesses both real and imaginary components simultaneously; contiguous pairs avoid cache misses.
- **Unified 2-D Stride**: Indexing is identical to `DenseMatrix` with `index = row * cols + col`.

---

## 3. Complex Scalars vs. Complex Arrays

Following MATLAB semantics:
1. There is no distinct `ComplexScalar` type. A scalar complex number is simply a `ComplexDenseMatrix` with `rows == 1` and `cols == 1`.
2. Scalar expansion rules apply identically: an operation between a scalar complex number and a complex matrix (or real matrix) broadcasts the scalar across all elements.

---

## 4. Real and Imaginary Components & Primitives

The core complex primitive functions operate element-wise:

| Builtin | Mathematical Definition | Return Type | MATLAB Semantics |
|---|---|---|---|
| `real(Z)` | $\text{Re}(z)$ | `Value::Matrix` | Extracts real part; identity if input is already real |
| `imag(Z)` | $\text{Im}(z)$ | `Value::Matrix` | Extracts imaginary part; returns zeros if real |
| `conj(Z)` | $\text{Re}(z) - i\,\text{Im}(z)$ | `Value::ComplexMatrix` | Complex conjugate; identity if real |
| `angle(Z)`| $\text{atan2}(\text{Im}(z), \text{Re}(z))$ | `Value::Matrix` | Phase angle in radians $[-\pi, \pi]$ |
| `abs(Z)`  | $\sqrt{\text{Re}(z)^2 + \text{Im}(z)^2}$ | `Value::Matrix` | Complex modulus / magnitude via `hypot` |
| `isreal(Z)` | $\forall z \in Z, \text{Im}(z) == 0$ | `Value::Boolean` | `true` for `Value::Matrix`, or `ComplexMatrix` with all zero imaginary |

---

## 5. Complex Arithmetic & Operator Semantics

### 5.1 Addition & Subtraction (`+`, `-`)
$$(a + bi) \pm (c + di) = (a \pm c) + (b \pm d)i$$
Real-complex mixed operations promote the real operand by setting $\text{im} = 0.0$.

### 5.2 Multiplication (`*`, `.*`)
- **Element-wise Multiplication (`.*`)**:
  $$(a + bi)(c + di) = (ac - bd) + (ad + bc)i$$
- **Matrix Multiplication (`*`)**:
  $$\mathbf{C}_{ij} = \sum_{k} \mathbf{A}_{ik} \mathbf{B}_{kj}$$
  using complex multiplication in the accumulation loop.

### 5.3 Division (`/`, `./`, `\`, `.\`)
- **Element-wise Division (`./`)**:
  $$\frac{a + bi}{c + di} = \frac{(ac + bd) + (bc - ad)i}{c^2 + d^2}$$
  Implemented using Smith's method or Morrison's algorithm to prevent premature overflow/underflow during $c^2 + d^2$.
- **Matrix Left/Right Division (`/`, `\`)**:
  LU or QR decomposition with complex pivoting:
  $$\mathbf{A} \mathbf{X} = \mathbf{B} \implies \mathbf{X} = \mathbf{A} \setminus \mathbf{B}$$

### 5.4 Power (`^`, `.^`)
- For $z^w = e^{w \ln z}$, where $\ln z = \ln |z| + i \arg(z)$.
- Negative base with fractional exponent:
  `(-1) .^ 0.5` evaluates to `0 + 1i` (unlike real `powf` which gives `NaN`).

---

## 6. Complex Comparison Restrictions

In MATLAB, relational comparisons on complex numbers are subject to strict rules:
1. **Equality (`==`) and Inequality (`~=`)**:
   $$z_1 == z_2 \iff \text{Re}(z_1) == \text{Re}(z_2) \;\land\; \text{Im}(z_1) == \text{Im}(z_2)$$
2. **Order Comparisons (`<`, `<=`, `>`, `>=`)**:
   MATLAB documentation specifies that order comparisons compare **only the real parts**:
   $$z_1 < z_2 \iff \text{Re}(z_1) < \text{Re}(z_2)$$
   *(If both real parts are equal, MATLAB returns false for `<` regardless of imaginary parts).*
   A diagnostic warning or exact compatibility option should be configured.

---

## 7. Transpose Semantics: Conjugate (`'`) vs. Non-Conjugate (`.'`)

This is one of the most fundamental distinctions in MATLAB:
- **`A'` (Hermitian / Conjugate Transpose)**:
  $$\mathbf{A}^H_{ij} = \overline{\mathbf{A}_{ji}} = \text{Re}(\mathbf{A}_{ji}) - i\,\text{Im}(\mathbf{A}_{ji})$$
- **`A.'` (Array / Non-Conjugate Transpose)**:
  $$\mathbf{A}^T_{ij} = \mathbf{A}_{ji} = \text{Re}(\mathbf{A}_{ji}) + i\,\text{Im}(\mathbf{A}_{ji})$$

For real matrices, `A'` and `A.'` produce identical results. When `ComplexMatrix` lands, parser and interpreter must distinguish between `Expr::Transpose` (conjugate) and `Expr::DotTranspose` (non-conjugate).

---

## 8. Display & Formatting

In MATLAB, complex numbers format with an explicit sign and `i` (or `j`):
- Scalar: `3.0000 + 4.0000i`
- Negative imaginary: `1.5000 - 2.2500i`
- Pure imaginary: `0.0000 + 5.0000i` or `0 + 5i`
- Column alignment: the real and imaginary parts align on decimal points in tabular matrix display.

```rust
impl Complex64 {
    pub fn format_matlab(&self) -> String {
        let sign = if self.im < 0.0 { "-" } else { "+" };
        format!("{:10.4} {} {:10.4}i", self.re, sign, self.im.abs())
    }
}
```

---

## 9. Conversion & Interoperability

1. **Automatic Promotion**: Any binary operation between a `Matrix` (real) and a `ComplexMatrix` automatically promotes the real matrix to `ComplexMatrix` without precision loss.
2. **Demotion / Simplification**: Functions like `real()`, `imag()`, and `abs()` return real `Matrix`.
3. **Domain Error Transition**:
   - `sqrt(x)` for $x < 0$: currently returns `NaN`. When complex mode is enabled, it returns `ComplexMatrix` with $\text{Re} = 0, \text{Im} = \sqrt{-x}$.
   - `log(x)` for $x < 0$: returns $\ln(-x) + \pi i$.

---

## 10. Implementation Roadmap

| Milestone | Deliverables | Status |
|---|---|---|
| **Phase 1: Foundations** | `Complex64` struct in `crates/values`; `ComplexDenseMatrix`; `Value::ComplexMatrix` variant; formatting | **Completed & Verified (Batch 10)** |
| **Phase 2: Linalg Kernels** | Complex addition, multiplication, elementwise ops, conjugate transpose | **Completed & Verified (Batch 10)** |
| **Phase 3: Parser & Literals** | Support for imaginary literals (`1i`, `2.5j`, `1e-3i`) in lexer/parser | **Completed & Verified (Batch 10)** |
| **Phase 4: Runtime Integration**| Dual dispatch in interpreter; automatic promotion on domain boundary; complex builtins | **Completed & Verified (Batch 10)** |
| **Phase 5: Matrix Decompositions** | Complex LU, QR, Cholesky, and SVD | Planned (Future Batch) |

This staged approach ensures that real-valued computations remain 100% stable, fast, and unaffected while complex capabilities are introduced progressively.