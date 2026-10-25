# `ndgrid`

Rectangular grid in N-D space.

## Syntax
```matlab
[X1, X2, ...] = ndgrid(x1, x2, ...)
```

## Description
Generates N-dimensional coordinate grids where output $k$ varies along dimension $k$.
For two inputs `x1` (length $M$) and `x2` (length $N$), `X1` and `X2` have dimensions $M \times N$, transposed relative to `meshgrid`.
