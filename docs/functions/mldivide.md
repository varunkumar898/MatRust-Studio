# `mldivide`

Matrix left divide or linear system solve.

## Syntax

```matlab
x = mldivide(A, B)
x = A \ B
```

## Description

- `x = mldivide(A, B)`: Solves linear system A*x = B using LU factorization with partial pivoting.
- Implemented in RustLab following clean-room MATLAB-compatible specifications.

## References

1. MATLAB Documentation: `mldivide`
2. GNU Octave Documentation: `mldivide`
