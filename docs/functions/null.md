# `null`

Orthonormal basis for null space (kernel).

## Syntax

```matlab
Z = null(A)
```

## Description

- `Z = null(A)` returns an orthonormal basis for the null space of `A`.
- Satisfies:
  - $A Z \approx 0$
  - $Z^T Z = I$
  - $\text{size}(Z, 1) = N$ (number of columns of $A$)
  - $\text{size}(Z, 2) = N - \text{rank}(A)$
