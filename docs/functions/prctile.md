# `prctile`

Percentiles of an array.

## Syntax

```matlab
Y = prctile(X, p)
```

## Description

- `Y = prctile(X, p)` computes percentiles for percentages `p` in [0, 100].
- Uses linear interpolation between order statistics ($h = (N-1)q + 1$).
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `prctile`
2. NIST Engineering Statistics Handbook: Percentiles
