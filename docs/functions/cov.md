# `cov`

Covariance matrix.

## Syntax

```matlab
C = cov(A)
C = cov(x, y)
```

## Description

- `C = cov(A)` computes the sample covariance matrix for 2-D array `A` where columns are random variables and rows are observations.
- Normalizes by $N - 1$.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `cov`
2. GNU Octave Documentation: `cov`
