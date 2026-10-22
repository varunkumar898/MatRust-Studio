# Complex Number Value Model — Architectural Specification

## Status: Fully Implemented (Batches 10 & 11 Verified)

## 1. Executive Summary

MatRust Studio provides full complex-number support across its runtime, syntax, linear algebra kernels, and desktop environment, preserving real-valued fast paths and backwards compatibility.

This document establishes the formal design for complex-number representations and semantics across the MatRust architecture, including the Batch 11 storage semantics hardening and complex linear algebra extensions.

---

## 2. Core Value Representation & Memory Layout

### 2.1 The Value Enum

The top-level `Value` enum in `matlab_values` distinguishes real matrices from complex matrices:

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

## 4. Real and Imaginary Components & Storage Semantics

The core complex primitive functions operate element-wise:

| Builtin | Mathematical Definition | Return Type | MATLAB Semantics |
|---|---|---|---|
| `real(Z)` | $\text{Re}(z)$ | `Value::Matrix` | Extracts real part; identity if input is already real |
| `imag(Z)` | $\text{Im}(z)$ | `Value::Matrix` | Extracts imaginary part; returns zeros if real |
| `conj(Z)` | $\text{Re}(z) - i\,\text{Im}(z)$ | `Value::ComplexMatrix` (or `Value::Matrix` if simplified) | Complex conjugate; identity if real |
| `angle(Z)`| $\text{atan2}(\text{Im}(z), \text{Re}(z))$ | `Value::Matrix` | Phase angle in radians $[-\pi, \pi]$ |
| `abs(Z)`  | $\sqrt{\text{Re}(z)^2 + \text{Im}(z)^2}$ | `Value::Matrix` | Complex modulus / magnitude via `hypot` |
| `isreal(Z)` | Value storage check | `Value::Boolean` | `false` for `Value::ComplexMatrix`, `true` for `Value::Matrix` (even if imaginary parts of `ComplexMatrix` are 0, e.g. `isreal(complex(12)) == false`) |


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
2. **Arithmetic Demotion / Cancellation Simplification**:
   Arithmetic operations (`+`, `-`, `*`, `.*`, `./`, `.^`, `\`, `/`) wrap results in `from_complex_or_simplified`. When an arithmetic calculation cancels all imaginary parts to exact zero ($\forall z \in Z, \text{Im}(z) == 0.0 \land !\text{Im}(z).\text{is\_sign\_negative}()$), it demotes to real `Value::Matrix(DenseMatrix)`.
   - `(3 + 4i) + (5 - 4i)` produces real `8`, so `isreal(...)` is `true`.
   - In contrast, explicit complex storage constructors such as `complex(12)` preserve `Value::ComplexMatrix`, so `isreal(complex(12))` remains `false`.
3. **Extraction Builtins**: Functions `real()`, `imag()`, `abs()`, and `angle()` return real `Matrix`.
4. **Domain Error Transition**:
   - `sqrt(x)` for $x < 0$: returns `ComplexMatrix` with $\text{Re} = 0, \text{Im} = \sqrt{-x}$.
   - `log(x)` for $x < 0$: returns $\ln(-x) + \pi i$.

---

## 10. Complex Linear Algebra Architecture (Batch 11)

### 10.1 Complex One-Sided Jacobi SVD
- **Formulation**: $A = U \Sigma V^H$, where $U \in \mathbb{C}^{m \times m}$ is unitary ($U^H U = I_m$), $V \in \mathbb{C}^{n \times n}$ is unitary ($V^H V = I_n$), and $\Sigma$ contains non-negative singular values sorted in descending order.
- **Algorithm**: One-sided Jacobi (Hestenes method) operating on column pairs $(b_i, b_j)$ of accumulator matrix $B = A V$:
  $$\gamma = b_i^H b_j = \sum_{r=1}^m \overline{B_{ri}} B_{rj} = |\gamma| e^{i\theta}$$
  $$\alpha = \|b_i\|^2, \quad \beta = \|b_j\|^2, \quad \zeta = \frac{\beta - \alpha}{2 |\gamma|}$$
  $$t = \frac{\text{sign}(\zeta)}{|\zeta| + \sqrt{1 + \zeta^2}}, \quad c = \frac{1}{\sqrt{1 + t^2}}, \quad s = c t$$
  $$b_i' = c b_i - (s e^{-i\theta}) b_j, \quad b_j' = (s e^{i\theta}) b_i + c b_j$$
  Accumulating column rotations into $V$ preserves exact unitarity ($V^H V = I_n$).
- **Singular Values**: $\sigma_i = \|b_i\|$. Normalized columns form $U_1 = B \Sigma^{-1}$.
- **Unitarity Completion**: For rank-deficient or rectangular $m > n$ matrices, orthonormal completion for $U$ uses two-pass modified Gram-Schmidt projection against coordinate basis vectors.

### 10.2 Moore–Penrose Pseudoinverse & Numerical Rank
- **Pseudoinverse**: $A^+ = V \Sigma^+ U^H$, with $\Sigma^+_{ii} = 1/\sigma_i$ for $\sigma_i > \text{tol}$.
  Validated against all four Moore–Penrose conditions:
  1. $A A^+ A \approx A$
  2. $A^+ A A^+ \approx A^+$
  3. $(A A^+)^H \approx A A^+$
  4. $(A^+ A)^H \approx A^+ A$
- **Rank**: Number of singular values exceeding tolerance $\text{tol} = \max(m, n) \times \sigma_1 \times \epsilon$.

### 10.3 Complex Solvers: $A \setminus B$ and $B / A$
- **Square Systems**: Complex pivoted LU decomposition ($P A = L U$) with partial pivoting using modulus $|A_{ik}|$ as pivot selection criterion, followed by complex forward and backward substitution.
- **Overdetermined Least-Squares Systems ($m > n$)**: Complex Householder QR decomposition ($A = Q R$). Computes $X = R_1^{-1} (Q_1^H B)$ minimizing $\|A X - B\|_2$.
- **Underdetermined Systems ($m < n$)**: Minimum-norm solution via complex pseudoinverse $X = A^+ B$.
- **Right Division ($B / A$)**: Solved via $(A^H \setminus B^H)^H$.

### 10.4 Determinant, Inverse, Trace, and Norms
- `det(A)`: Computed from complex pivoted LU as $(-1)^{\text{swaps}} \prod U_{ii}$.
- `inv(A)`: Computes $A^{-1}$ via complex pivoted LU solve against identity matrix $I_n$.
- `trace(A)`: Sum of main diagonal elements $\sum A_{ii}$.
- `norm(A, [p])`:
  - Complex vector norms: $p=1$ ($\sum |z_i|$), $p=2$ ($\sqrt{\sum |z_i|^2}$), $p=\infty$ ($\max |z_i|$), general $p$ ($(\sum |z_i|^p)^{1/p}$).
  - Complex matrix norms: 1-norm (max column sum of moduli), $\infty$-norm (max row sum of moduli), Frobenius norm ($\sqrt{\sum |A_{ij}|^2}$), 2-norm (largest singular value $\sigma_1$ via complex SVD).

---

## 11. Implementation Roadmap & Verification Status

| Milestone | Deliverables | Status |
|---|---|---|
| **Phase 1: Foundations** | `Complex64` struct in `crates/values`; `ComplexDenseMatrix`; `Value::ComplexMatrix` variant; formatting | **Completed & Verified (Batch 10)** |
| **Phase 2: Linalg Kernels** | Complex addition, multiplication, elementwise ops, conjugate transpose | **Completed & Verified (Batch 10)** |
| **Phase 3: Parser & Literals** | Support for imaginary literals (`1i`, `2.5j`, `1e-3i`) in lexer/parser | **Completed & Verified (Batch 10)** |
| **Phase 4: Runtime Integration**| Dual dispatch in interpreter; automatic promotion on domain boundary; complex builtins | **Completed & Verified (Batch 10)** |
| **Phase 5: Storage Semantics Hardening** | Storage-based `isreal`, arithmetic cancellation to real storage | **Completed & Verified (Batch 11)** |
| **Phase 6: Matrix Decompositions & Solvers** | Complex SVD, pinv, rank, LU, QR least squares, mldivide, mrdivide, det, inv, trace, norms | **Completed & Verified (Batch 11)** |