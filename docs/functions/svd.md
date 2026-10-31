# `svd`

Singular value decomposition of $M \times N$ matrices.

## Syntax

```matlab
s = svd(A)
[U, S, V] = svd(A)
[U, S, V] = svd(A, 'econ')
```

## Description

- `s = svd(A)` returns the singular values of matrix `A` in descending order as a $\min(M, N) \times 1$ column vector.
- `[U, S, V] = svd(A)` produces a diagonal matrix `S` of dimension $M \times N$ with non-negative diagonal elements in decreasing order, and unitary matrices `U` ($M \times M$) and `V` ($N \times N$) satisfying:
  $$A = U S V^T$$
- `[U, S, V] = svd(A, 'econ')` computes the economy-sized decomposition where $K = \min(M, N)$, `U` is $M \times K$, `S` is $K \times K$, and `V` is $N \times K$.

## Implementation Details

- Feature-gated behind `faer`. Returns an informative error when compiled with `--no-default-features`.
- Returns exact dimensions for empty $0 \times 0$, $M \times 0$, and $0 \times N$ matrices without invocation of external solvers.
