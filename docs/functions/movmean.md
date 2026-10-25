# `movmean`

Moving mean.

## Syntax

```matlab
M = movmean(A, k)
M = movmean(A, k, dim)
M = movmean(..., 'omitnan')
```

## Description

- `M = movmean(A, k)` computes centered moving average with window length `k`.
- Shrinks window at endpoints to include only available elements within window boundaries.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `movmean`
2. GNU Octave Documentation: `movmean`
