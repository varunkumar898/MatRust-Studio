# `head`

Return first rows of table or matrix.

## Syntax

```matlab
head(T)
head(T, k)
```

## Description

- `head(T)` returns the top 8 rows of table or matrix `T`.
- `head(T, k)` returns the top `k` rows of table or matrix `T`.

## Implementation Details

- If `T` is a table structure, each column variable is sliced to the first `min(k, NumRows)` elements.
- If `T` is a numeric matrix, slices rows `1:min(k, size(T, 1))`.
