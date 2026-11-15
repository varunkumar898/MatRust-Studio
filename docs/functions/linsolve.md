# `linsolve`

Solve linear system of equations.

## Syntax

```matlab
X = linsolve(A, B)
```

## Description

- `X = linsolve(A, B)` solves the linear system $A X = B$.
- For square matrices, it computes the exact solution via LU decomposition with partial pivoting.
- For rectangular overdetermined systems ($M > N$), it solves the least-squares problem via QR decomposition.
- For rectangular underdetermined systems ($M < N$), it computes the minimum-norm solution via pseudoinverse SVD.
