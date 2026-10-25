# `cummin`

Cumulative minimum.

## Syntax

```matlab
M = cummin(A)
M = cummin(A, dim)
```

## Description

- `M = cummin(A)` computes cumulative minimum along the first non-singleton dimension of `A`.
- Preserves propagation of NaN values according to MATLAB standard semantics.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `cummin`
2. GNU Octave Documentation: `cummin`
