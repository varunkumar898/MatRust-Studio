# `zscore`

Standardized z-score.

## Syntax

```matlab
Z = zscore(X)
Z = zscore(X, dim)
```

## Description

- `Z = zscore(X)` computes centered and standardized z-scores $(X - \mu) / \sigma$ along the first non-singleton dimension.
- Normalizes by $N - 1$.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `zscore`
2. GNU Octave Documentation: `zscore`
