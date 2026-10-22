# Complex Number Value Model — Design Document

## Status: Design Phase (not yet implemented)

## Current State
- `Value` enum has `Matrix(DenseMatrix)`, `String(String)`, `Boolean(bool)`.
- `DenseMatrix` stores `Vec<f64>` in row-major order.
- No complex type exists. `sqrt(-1)` returns NaN (MATLAB: `0 + 1i`).
- `real`, `imag`, `conj`, `angle` are not implemented.

## Design

### 1. Value Representation
Add a new variant:
```rust
pub enum Value {
    Matrix(DenseMatrix),      // real-valued (existing)
    ComplexMatrix(ComplexDenseMatrix),  // complex-valued (new)
    String(String),
    Boolean(bool),
}
```

### 2. ComplexDenseMatrix
```rust
pub struct ComplexDenseMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Complex<f64>>,  // or (f64, f64) pairs
}
```

### 3. Complex Type
Use `num_complex::Complex<f64>` or a custom struct:
```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}
```

### 4. Arithmetic
- Complex + Complex: (a+bi) + (c+di) = (a+c) + (b+d)i
- Complex * Complex: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
- Complex / Complex: (a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c²+d²)
- Real op Complex: promote real to complex (im=0)

### 5. Matrix Operations
- Complex matmul: use complex arithmetic in the inner loop
- Complex transpose: conjugate transpose (') vs non-conjugate (.')
- Complex inverse: Gauss-Jordan with complex pivoting
- Eigenvalues: complex Schur (requires complex arithmetic)

### 6. Display Formatting
- `3 + 4i` for complex scalars
- `3.0000 + 4.0000i` for matrices
- Real-valued complex (im=0) displayed as real

### 7. Conversion
- `complex(A)` or `A + 0i` promotes real to complex
- `real(C)` extracts real part
- `imag(C)` extracts imaginary part
- `conj(C)` conjugates
- `abs(C)` returns magnitude (real-valued result)

### 8. Interoperability
- Real operations on complex inputs: promote to complex
- Complex operations on real inputs: promote real to complex
- Mixed-type binary ops: promote real operand to complex

### 9. Migration Path
1. Add Complex type and ComplexDenseMatrix
2. Add complex arithmetic to linalg
3. Update builtins to handle both Matrix and ComplexMatrix
4. Update parser for `1i`, `1j` imaginary literals
5. Update formatter for complex display
6. Add complex tests

### 10. Dependencies
- `num-complex` crate (MIT/Apache-2.0) for Complex type
- Or custom implementation to avoid dependency

### 11. Limitations
- No complex support in current implementation
- `sqrt(-1)` returns NaN (MATLAB: 0+1i)
- `real`, `imag`, `conj`, `angle` not yet implemented
- Eigenvalues and SVD require complex support
EOF
echo "created"