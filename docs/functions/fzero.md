# `fzero`

Root of single-variable nonlinear function.

## Syntax

```matlab
x = fzero(fun, x0)
[x, fval] = fzero(fun, x0)
```

## Description

- `x = fzero(fun, x0)` finds a zero of function `fun` near initial point `x0` or within interval `[a, b]`.
- Uses Brent's method combining bisection, secant method, and inverse quadratic interpolation for guaranteed convergence.
