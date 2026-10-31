# `polyfit`

Polynomial curve fitting.

## Syntax

```matlab
p = polyfit(x, y, n)
```

## Description

`p = polyfit(x, y, n)` finds the coefficients of a polynomial $p(x)$ of degree `n` that fits the data `y` best in a least-squares sense:
$$p(x) = p_1 x^n + p_2 x^{n-1} + \dots + p_n x + p_{n+1}$$

## Implementation Details

- Forms the Vandermonde system matrix $V_{i,j} = x_i^{n - j}$.
- Solves normal equations $(V^T V) p = V^T y$ using Gaussian elimination with partial pivoting.
- Requires $m \ge n + 1$ data points.
