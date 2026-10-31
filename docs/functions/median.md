# `median`

Median value of an array.

## Syntax

```matlab
M = median(A)
M = median(A, dim)
M = median(..., 'omitnan')
```

## Description

- `M = median(A)` computes the median value of array `A` along the first non-singleton dimension.
- If the number of elements is even, returns the average of the two middle values.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `median`
2. GNU Octave Documentation: `median`
