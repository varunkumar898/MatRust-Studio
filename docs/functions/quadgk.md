# `quadgk`

Adaptive Gauss-Kronrod numerical quadrature.

## Syntax

```matlab
q = quadgk(fun, a, b)
```

## Description

- `q = quadgk(fun, a, b)` approximates the integral of function `fun` from `a` to `b` using adaptive numerical quadrature.
- Designed for robust handling of smooth, oscillatory, or moderately singular integrands over finite intervals.
