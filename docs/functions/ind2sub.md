# `ind2sub`

Multiple subscripts from linear index.

## Syntax

```matlab
[I, J] = ind2sub(sz, IND)
[I1, I2, ..., In] = ind2sub(sz, IND)
```

## Description

- `[I, J] = ind2sub(sz, IND)` determines the equivalent row and column subscript values corresponding to given linear index `IND` for an array of size `sz`.
- `[I1, I2, ..., In] = ind2sub(sz, IND)` determines the subscripts for an n-dimensional array.
- Column-major (1-based) indexing is used consistent with standard MATLAB semantics.
