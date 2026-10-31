# `cumtrapz`

Cumulative trapezoidal numerical integration.

## Syntax

```matlab
z = cumtrapz(y)
z = cumtrapz(x, y)
```

## Description

- `z = cumtrapz(y)` computes the cumulative integral of `y` with unit spacing.
- `z = cumtrapz(x, y)` computes the cumulative integral with respect to coordinates `x`.

## Implementation Details

- Returns cumulative array of same length as `y`, starting with zero:
  $$z_1 = 0, \quad z_k = z_{k-1} + \frac{y_{k-1} + y_k}{2} (x_k - x_{k-1})$$
