# `surf`

Create a 3D surface plot.

## Syntax

```matlab
surf(X, Y, Z)
surf(Z)
```

## Description

- `surf(X, Y, Z)` creates a 3D surface plot from grid matrices X, Y, Z of the
  same size. X varies along columns, Y varies along rows, Z is the height.
- `surf(Z)` uses implicit X = 1:cols(Z), Y = 1:rows(Z).
- A colorbar is added automatically showing the Z scale.

## Example

```matlab
[X, Y] = meshgrid(-3:0.2:3);
Z = sin(sqrt(X.^2 + Y.^2));
surf(X, Y, Z);
```

## References

1. MATLAB Documentation: `surf`
2. GNU Octave Documentation: `surf`
