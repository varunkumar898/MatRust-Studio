# `mode`

Most frequent values in an array.

## Syntax

```matlab
M = mode(A)
M = mode(A, dim)
M = mode(..., 'omitnan')
```

## Description

- `M = mode(A)` returns the sample mode of `A` along the first non-singleton dimension.
- When multiple values have the same maximum frequency, the smallest value is selected.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `mode`
2. GNU Octave Documentation: `mode`
