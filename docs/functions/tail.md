# `tail`

Return last rows of table or matrix.

## Syntax

```matlab
tail(T)
tail(T, k)
```

## Description

- `tail(T)` returns the bottom 8 rows of table or matrix `T`.
- `tail(T, k)` returns the bottom `k` rows of table or matrix `T`.

## Implementation Details

- If `T` is a table structure, each column variable is sliced to the last `min(k, NumRows)` elements.
- If `T` is a numeric matrix, slices rows `max(1, end - k + 1):end`.
