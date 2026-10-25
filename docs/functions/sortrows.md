# `sortrows`

Sort rows of a matrix.

## Syntax

```matlab
B = sortrows(A)
B = sortrows(A, cols)
[B, idx] = sortrows(A, ...)
```

## Description

- `B = sortrows(A)` sorts matrix rows lexicographically based on the first column, breaking ties with subsequent columns.
- `B = sortrows(A, cols)` sorts based on specified columns; negative indices indicate descending order.
- Clean-room implementation in `rustlab-data`.

## References

1. MATLAB Documentation: `sortrows`
2. GNU Octave Documentation: `sortrows`
