# `fminsearch`

Multidimensional unconstrained nonlinear minimization.

## Syntax

```matlab
x = fminsearch(fun, x0)
[x, fval] = fminsearch(fun, x0)
```

## Description

- `x = fminsearch(fun, x0)` finds the minimum of an unconstrained multivariable function using the derivative-free Nelder-Mead simplex algorithm.
- Iteratively applies reflection, expansion, contraction, and shrink transformations to converge on the local minimum.
