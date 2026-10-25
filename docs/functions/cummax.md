# `cummax`

Cumulative maximum.

## Syntax

```matlab
M = cummax(A)
M = cummax(A, dim)
```

## Description

- `M = cummax(A)` computes cumulative maximum along the first non-singleton dimension of `A`.
- Preserves propagation of NaN values according to MATLAB standard semantics.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `cummax`
2. GNU Octave Documentation: `cummax`
