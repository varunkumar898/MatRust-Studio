# `rank`

Matrix rank from singular values.

## Syntax

```matlab
k = rank(A)
k = rank(A, tol)
```

## Description

- `k = rank(A)` returns the number of linearly independent rows or columns of `A`.
- Evaluated as the number of singular values of `A` strictly greater than `tol`.
- Default tolerance:
  $$\text{tol} = \max(M, N) \cdot \text{eps}(\sigma_1)$$
