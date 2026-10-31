# `cat`

Concatenate arrays along specified dimension.

## Syntax
```matlab
C = cat(dim, A, B, ...)
```

## Description
Concatenates arrays along dimension `dim` (1-based). All dimensions other than `dim` must agree.
- `cat(1, A, B)` is equivalent to `vertcat(A, B)` or `[A; B]`.
- `cat(2, A, B)` is equivalent to `horzcat(A, B)` or `[A, B]`.
- `cat(3, A, B)` concatenates along the 3rd dimension.
