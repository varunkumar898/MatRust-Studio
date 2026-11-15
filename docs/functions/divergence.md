# `divergence`

Divergence of a vector field.

## Syntax

```matlab
div = divergence(X, Y, U, V)
div = divergence(U, V)
```

## Description

- `div = divergence(X, Y, U, V)` calculates the divergence of a 2-D vector field with components `U` and `V` at grid coordinates `(X, Y)`:
  $$\nabla \cdot \mathbf{F} = \frac{\partial U}{\partial x} + \frac{\partial V}{\partial y}$$
- `div = divergence(U, V)` calculates the divergence assuming unit spacing ($h_x = 1, h_y = 1$).

## Implementation Details

- Evaluates partial derivatives along column ($x$) and row ($y$) directions using central differences in the interior and one-sided differences at boundaries.
