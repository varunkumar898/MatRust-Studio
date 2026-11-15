# `integral2`

Numerically evaluate double integral over rectangular region.

## Syntax

```matlab
q = integral2(fun, xmin, xmax, ymin, ymax)
```

## Description

- `q = integral2(fun, xmin, xmax, ymin, ymax)` approximates the double integral of function `fun(x, y)` over the planar rectangular region $[xmin, xmax] \times [ymin, ymax]$.
- Uses iterated numerical quadrature with adaptive subdivision along spatial dimensions to ensure high numerical accuracy.
