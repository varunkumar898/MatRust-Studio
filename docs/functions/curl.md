# `curl`

Curl and angular velocity of a 2-D vector field.

## Syntax

```matlab
[cav, vor] = curl(X, Y, U, V)
[cav, vor] = curl(U, V)
cav = curl(X, Y, U, V)
```

## Description

- `[cav, vor] = curl(X, Y, U, V)` calculates the angular velocity `cav` and vorticity `vor` of a 2-D vector field $(U, V)$:
  $$\text{vor} = \frac{\partial V}{\partial x} - \frac{\partial U}{\partial y}$$
  $$\text{cav} = \frac{1}{2} \text{vor} = \frac{1}{2}\left(\frac{\partial V}{\partial x} - \frac{\partial U}{\partial y}\right)$$
- `[cav, vor] = curl(U, V)` assumes unit grid spacing.
- `cav = curl(...)` returns the angular velocity when a single output argument is requested.

## Implementation Details

- Clean-room numerical implementation evaluating central differences along coordinates $(X, Y)$.
