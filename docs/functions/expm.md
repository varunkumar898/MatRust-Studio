# `expm`

Matrix exponential.

## Syntax

```matlab
Y = expm(X)
```

## Description

- `Y = expm(X)` computes the matrix exponential $e^X = \sum_{k=0}^\infty \frac{X^k}{k!}$.
- Implemented natively using Higham's scaling and squaring method with [6/6] Padé approximation and Gauss-Jordan partial pivoting linear system solver.
- Preserves properties including $e^0 = I$, $e^{\text{diag}(d)} = \text{diag}(e^d)$, nilpotent matrix exponentials $e^{[0\ 1; 0\ 0]} = [1\ 1; 0\ 1]$, and rotation generation $e^{[0\ -t; t\ 0]} = R(t)$.

## References

1. Higham, N. J. (2005). *The scaling and squaring method for the matrix exponential revisited*. SIAM Journal on Matrix Analysis and Applications, 26(4), 1179-1193.
2. Moler, C., & Van Loan, C. (2003). *Nineteen dubious ways to compute the exponential of a matrix, twenty-five years later*. SIAM Review, 45(1), 3-49.
