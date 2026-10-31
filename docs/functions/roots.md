# `roots`

Polynomial roots.

## Syntax

```matlab
r = roots(p)
```

## Description

`r = roots(p)` returns a column vector whose elements are the roots of the polynomial whose coefficients are the elements of vector `p`.

If `p` has $N+1$ elements, the polynomial is:
$$p(x) = p_1 x^N + p_2 x^{N-1} + \dots + p_N x + p_{N+1}$$

## Implementation Details

- Clean-room eigenvalue-based polynomial root calculation.
- Strips leading zeros.
- For degree 1 ($N=1$), computes exact root $-p_2 / p_1$.
- For degree 2 ($N=2$), computes quadratic formula with complex discriminant handling.
- For degree $N \ge 3$, constructs the companion matrix $C$ of size $N \times N$ and solves for its eigenvalues via `eig`.
