# `quad`

Adaptive Simpson numerical quadrature.

## Syntax

```matlab
q = quad(fun, a, b)
q = quad(fun, a, b, tol)
```

## Description

- `q = quad(fun, a, b)` approximates the integral of function `fun` from `a` to `b` using recursive adaptive Simpson quadrature.
- Function `fun` is a function handle `@(x) ...`.

## Implementation Details

- Pure Rust clean-room recursive Simpson's rule with adaptive interval bisection and tolerance control.
