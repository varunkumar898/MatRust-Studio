# `trapz`

Trapezoidal numerical integration.

## Syntax

```matlab
z = trapz(y)
z = trapz(x, y)
```

## Description

- `z = trapz(y)` computes the approximate integral of `y` via the trapezoidal method with unit spacing ($dx = 1$).
- `z = trapz(x, y)` integrates `y` with respect to coordinates specified by `x`.

## Implementation Details

- Evaluates discrete sum:
  $$\int y dx \approx \sum_{i=1}^{N-1} \frac{y_i + y_{i+1}}{2} (x_{i+1} - x_i)$$
- Validates vector length equality between `x` and `y`.
