# `interp1`

1-D data interpolation.

## Syntax

```matlab
yq = interp1(x, y, xq)
yq = interp1(x, y, xq, method)
```

## Description

`yq = interp1(x, y, xq)` returns interpolated values of a 1-D function at query points `xq`.
`method` can be `'linear'` (default) or `'nearest'`.

## Implementation Details

- Assumes sample points `x` are sorted monotonically.
- Queries outside the interval $[\min(x), \max(x)]$ return `NaN` (no extrapolation by default).
- Implements binary search bracket location followed by linear or nearest-neighbor interpolation.
