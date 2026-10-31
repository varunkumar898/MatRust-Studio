# `pinv`

Moore-Penrose pseudoinverse of rectangular or rank-deficient matrices.

## Syntax

```matlab
B = pinv(A)
B = pinv(A, tol)
```

## Description

- `B = pinv(A)` calculates the pseudoinverse $A^+$ satisfying the four Moore-Penrose equations:
  1. $A B A = A$
  2. $B A B = B$
  3. $(A B)^T = A B$
  4. $(B A)^T = B A$
- Singular values $\sigma_i \le \text{tol}$ are truncated to zero during inverse calculation.
- The default tolerance is:
  $$\text{tol} = \max(M, N) \cdot \text{eps}(\sigma_1)$$
  where $\text{eps}(x)$ is the distance from $|x|$ to the next floating-point number.
