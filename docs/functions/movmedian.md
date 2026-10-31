# `movmedian`

Moving median.

## Syntax

```matlab
M = movmedian(A, k)
M = movmedian(A, k, dim)
M = movmedian(..., 'omitnan')
```

## Description

- `M = movmedian(A, k)` computes centered moving median with window length `k`.
- Shrinks window at endpoints to available elements.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `movmedian`
2. GNU Octave Documentation: `movmedian`
