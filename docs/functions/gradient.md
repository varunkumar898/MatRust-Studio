# `gradient`

Numerical gradient.

## Syntax

```matlab
g = gradient(f)
g = gradient(f, h)
```

## Description

- `g = gradient(f)` calculates the central difference numerical gradient along vector `f` assuming unit point spacing.
- `g = gradient(f, h)` calculates the gradient using point spacing `h`.

## Implementation Details

- Endpoint derivatives use forward/backward differences:
  $$g_1 = \frac{f_2 - f_1}{h}, \quad g_N = \frac{f_N - f_{N-1}}{h}$$
- Interior points use second-order central differences:
  $$g_i = \frac{f_{i+1} - f_{i-1}}{2 h}$$
