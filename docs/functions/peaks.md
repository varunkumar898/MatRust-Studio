# `peaks`

Evaluate the peaks function of two variables.

## Syntax

```matlab
z = peaks
z = peaks(n)
z = peaks(x, y)
```

## Description

- `peaks` returns `peaks(49)`, a 49-by-49 matrix.
- `peaks(n)` returns an n-by-n matrix evaluated on the grid from
  `meshgrid(linspace(-3, 3, n))`.
- `peaks(x, y)` evaluates the peaks function on the grid defined by
  vectors `x` (columns) and `y` (rows).

The peaks function is:

z = 3*(1-x).^2.*exp(-(x.^2)-(y+1).^2)
  - 10*(x/5 - x.^3 - y.^5).*exp(-x.^2-y.^2)
  - (1/3)*exp(-(x+1).^2-y.^2)

## Examples

```matlab
z = peaks(0, 0)      % z = 0.9810
z = peaks(49)        % 49-by-49 matrix
[X, Y] = meshgrid(-3:0.2:3);
Z = peaks(X, Y);     % 31-by-31 matrix
```

## References

1. MATLAB Documentation: `peaks`
2. GNU Octave Documentation: `peaks`
