# `histcounts`

Histogram bin counts.

## Syntax

```matlab
N = histcounts(X)
N = histcounts(X, edges)
```

## Description

- `N = histcounts(X)` partitions numerical array `X` into bins and returns element counts per bin.
- `N = histcounts(X, edges)` partitions `X` using specified strictly increasing bin edges. All bins except the last are half-open $[e_j, e_{j+1})$, with the final bin closed $[e_k, e_{k+1}]$.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `histcounts`
2. GNU Octave Documentation: `histcounts`
