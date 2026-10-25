# `eig`

Eigenvalues and eigenvectors of square matrices.

## Syntax

```matlab
d = eig(A)
[V, D] = eig(A)
```

## Description

- `d = eig(A)` returns a column vector containing the eigenvalues of square matrix `A`.
- `[V, D] = eig(A)` returns diagonal matrix `D` of eigenvalues and full matrix `V` whose columns are the corresponding right eigenvectors, satisfying:
  $$A V = V D$$

## Implementation Details

- **Real Symmetric Matrices**: Automatically detected using tolerance $10^{-12} \cdot \max(|A_{ij}|, |A_{ji}|) + 1.0$. Computed using native clean-room Householder tridiagonalization (`tred2`) followed by the implicit QL algorithm with Wilkinson shifts (`tql2`). Eigenvalues are sorted in ascending order.
- **Eigenvector Sign Normalization**: Each eigenvector column in `V` is normalized such that its component of maximum absolute value is strictly positive. If multiple components share the maximum magnitude, the lowest row index is chosen (tie-break). Note that this standardizes sign conventions and differs from LAPACK's non-standardized phase.
- **General Non-Symmetric Matrices**: Computed via Schur decomposition (delegated to `faer`).

## References

1. Golub, G. H., & Van Loan, C. F. (2013). *Matrix Computations* (4th ed.). Johns Hopkins University Press. §8.2.
2. Martin, R. S., Reinsch, C., & Wilkinson, J. H. (1968). *Householder's tridiagonalization of a symmetric matrix*. Numerische Mathematik, 11(3), 181-195.
