# `sub2ind`

Linear index from multiple subscripts.

## Syntax

```matlab
IND = sub2ind(sz, I, J)
IND = sub2ind(sz, I1, I2, ..., In)
```

## Description

- `IND = sub2ind(sz, I, J)` returns the linear index equivalent to the row and column subscripts `I` and `J` for a matrix of size `sz`.
- `IND = sub2ind(sz, I1, I2, ..., In)` converts subscripts for an n-dimensional array into equivalent linear indices.
- Subscripts must be 1-based and within the bounds defined by `sz`.
