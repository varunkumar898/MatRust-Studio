# `orth`

Orthonormal basis for range (column space / image).

## Syntax

```matlab
Q = orth(A)
```

## Description

- `Q = orth(A)` returns an orthonormal basis for the range of `A`.
- Satisfies:
  - $Q^T Q = I$
  - $\text{size}(Q, 1) = M$ (number of rows of $A$)
  - $\text{size}(Q, 2) = \text{rank}(A)$
