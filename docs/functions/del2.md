# `del2`

Discrete Laplacian of array.

## Syntax

```matlab
L = del2(U)
L = del2(U, h)
L = del2(U, hx, hy)
```

## Description

- `L = del2(U)` calculates a discrete approximation of Laplace's differential operator applied to array `U`.
- In 1D, `del2(u)` calculates $\frac{1}{2} \frac{d^2 u}{dx^2}$.
- In 2D, `del2(U)` calculates $\frac{1}{4} \nabla^2 U = \frac{1}{4} \left(\frac{\partial^2 U}{\partial x^2} + \frac{\partial^2 U}{\partial y^2}\right)$.
- `L = del2(U, h)` specifies equal grid spacing `h`.
- `L = del2(U, hx, hy)` specifies grid spacing `hx` along columns (x) and `hy` along rows (y).

## Implementation Details

- Uses standard central second differences for interior points and second-order forward/backward differences at boundary points.
