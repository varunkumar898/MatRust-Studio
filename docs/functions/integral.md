# `integral`

Adaptive numerical integration of single-variable function.

## Syntax

```matlab
q = integral(fun, a, b)
```

## Description

- `q = integral(fun, a, b)` approximates the integral of function `fun` from `a` to `b` using adaptive Simpson's quadrature.
- Recursively refines intervals where curvature or function error is significant until target error tolerances are satisfied.
