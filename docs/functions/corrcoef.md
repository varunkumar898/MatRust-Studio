# `corrcoef`

Correlation coefficients.

## Syntax

```matlab
R = corrcoef(A)
R = corrcoef(x, y)
```

## Description

- `R = corrcoef(A)` computes the Pearson correlation coefficient matrix between columns of `A`.
- Diagonal elements are 1.0, and off-diagonal entries are normalized covariance values.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `corrcoef`
2. GNU Octave Documentation: `corrcoef`
