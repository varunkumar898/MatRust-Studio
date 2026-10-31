# Milestone A3 Design: Numeric Types, N-Dimensional Tensor Parity, and Sparse CSC Matrices

## 1. Architectural Overview

Milestone A3 completes the foundational numerical type system and tensor representation of **RustLab**, expanding beyond double-precision matrices into first-class numerical classes, true multidimensional array semantics, and compressed sparse storage.

```
                      +---------------------------------------+
                      |               Value Enum              |
                      +---------------------------------------+
                                          |
        +---------------------------------+---------------------------------+
        |                                 |                                 |
+-------------------+             +-------------------+             +-------------------+
|  Floating-Point   |             |   Integer Types   |             |    Sparse (CSC)   |
+-------------------+             +-------------------+             +-------------------+
| Double (f64)      |             | Int8, Int16       |             | CscMatrix<f64>    |
| Single (f32)      |             | Int32, Int64      |             | col_ptr: Vec<usize>
| ComplexDouble     |             | UInt8, UInt16     |             | row_ind: Vec<usize>
|                   |             | UInt32, UInt64    |             | values: Vec<f64>  |
+-------------------+             +-------------------+             +-------------------+
```

---

## 2. Numeric Classes & Saturating Arithmetic

MATLAB requires strict integer behaviors that differ fundamentally from standard C/Rust semantics:
1. **Saturating Arithmetic**:
   - Overflow and underflow clamp to `T::MIN` and `T::MAX` rather than wrapping or trapping (`uint8(250) + 10 == 255`, `uint8(3) - 5 == 0`, `int8(127) + 1 == 127`).
   - Unary negation saturates: `-int8(-128) == 127` and `abs(int8(-128)) == 127`.
2. **Round Half Away from Zero**:
   - Float-to-integer conversion rounds ties away from zero (`int32(2.5) == 3`, `int32(-2.5) == -3`).
3. **Exact 64-bit Integer Operations**:
   - `int64` and `uint64` preserve full 64-bit precision without conversion to double float (`intmax('int64') - 1 == 9223372036854775806`).
4. **Mixed-Class Rules**:
   - `integer + double -> integer` (with saturating cast).
   - `integer_A + integer_B -> Error: "Integers of different class cannot be combined."`
   - `single + double -> single`.
   - `logical / char + double -> double`.
5. **Integer Division (`idivide`)**:
   - Default division `/` between integers rounds to nearest integer.
   - Builtin `idivide(A, B, opt)` supports `'fix'` (round toward zero), `'floor'` (round toward $-\infty$), `'ceil'` (round toward $+\infty$), and `'round'` (round half away from zero).

---

## 3. N-Dimensional Tensor Parity

RustLab's `DenseArray<T>` represents multidimensional arrays in column-major order with explicit shape `[d1, d2, ..., dK]` and strides `[1, d1, d1*d2, ...]`.

Key parity features:
1. **Multi-Subscript Slicing & Assignment**:
   - Multi-index slicing $A(r_1, r_2, \dots, r_K)$ computes the Cartesian product of coordinate ranges in column-major order for arbitrary dimensions $K \ge 2$.
   - Sliced assignment supports both scalar filling and matching-shape tensor assignment.
   - Deletion: `A(row, :) = []` and `A(:, col) = []` shrinks matrix dimensions, while `A(mask) = []` flattens and deletes linear subsets.
2. **N-D Reductions Along Arbitrary Dim $k$**:
   - `sum`, `prod`, `mean`, `min`, `max`, `cumsum`, `any`, `all` operate along any dimension $k \ge 1$.
   - Integer inputs preserve their integer class and use saturating accumulation for `sum`, `prod`, and `cumsum`.
   - `mean` always returns `double` per MATLAB specification.
3. **Array Transformations**:
   - `cat(dim, A, B, ...)`: arbitrary dimension concatenation.
   - `permute(A, order)`: multidimensional generalized transpose.
   - `meshgrid` and `ndgrid`: Cartesian coordinate grid generators.
   - `reshape(A, d1, d2, ...)`: supports `[]` for automatic dimension deduction.

---

## 4. Compressed Sparse Column (CSC) Matrices

The `CscMatrix<f64>` struct provides standard Compressed Sparse Column format:
- `shape: [usize; 2]`: matrix dimensions $M \times N$.
- `col_ptr: Vec<usize>`: column start offsets (length $N + 1$).
- `row_ind: Vec<usize>`: 0-based row indices of non-zero entries.
- `values: Vec<f64>`: non-zero floating-point values.

Features:
- `sparse(i, j, v, m, n)`: constructs CSC matrix from coordinate triplets with duplicate coordinate accumulation.
- `full(S)`: converts sparse matrix into dense double matrix.
- `speye(m, n)`: generates sparse identity matrices.
- `spones(S)`: extracts binary sparsity pattern.
- Memory efficiency: for an $N \times N$ matrix with $K$ non-zeros, storage requires $(N + 1) \times 8 + K \times 16$ bytes, compared to $N^2 \times 8$ bytes for dense storage.
