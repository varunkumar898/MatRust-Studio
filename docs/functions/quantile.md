# `quantile`

Quantiles of an array.

## Syntax

```matlab
Y = quantile(X, q)
```

## Description

- `Y = quantile(X, q)` computes quantiles for proportions `q` in [0, 1].
- Uses linear interpolation between order statistics ($h = (N-1)q + 1$).
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `quantile`
2. NIST Engineering Statistics Handbook: Quantiles
