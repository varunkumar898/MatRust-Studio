# `meshgrid`

2-D and 3-D grids for plotting and evaluations.

## Syntax
```matlab
[X, Y] = meshgrid(x, y)
[X, Y] = meshgrid(x)
[X, Y, Z] = meshgrid(x, y, z)
```

## Description
Generates 2-D or 3-D Cartesian coordinate arrays.
For vectors `x` (length $N$) and `y` (length $M$), `X` and `Y` have dimensions $M \times N$.
- `X` contains copies of `x` across rows.
- `Y` contains copies of `y` down columns.
