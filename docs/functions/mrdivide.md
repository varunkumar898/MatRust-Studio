# `mrdivide`

Matrix right divide.

## Syntax

```matlab
x = mrdivide(B, A)
x = B / A
```

## Description

- `x = mrdivide(B, A)`: Solves linear system x*A = B. Equivalent to (A' \ B')'.
- Implemented in RustLab following clean-room MATLAB-compatible specifications.

## References

1. MATLAB Documentation: `mrdivide`
2. GNU Octave Documentation: `mrdivide`
