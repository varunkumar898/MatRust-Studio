# `integral3`

Numerically evaluate triple integral over rectangular cuboid volume.

## Syntax

```matlab
q = integral3(fun, xmin, xmax, ymin, ymax, zmin, zmax)
```

## Description

- `q = integral3(fun, xmin, xmax, ymin, ymax, zmin, zmax)` computes the triple volume integral of function `fun(x, y, z)` over the domain $[xmin, xmax] \times [ymin, ymax] \times [zmin, zmax]$.
- Uses nested iterated adaptive numerical quadrature across each coordinate axis.
