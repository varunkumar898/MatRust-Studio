# contour

2-D contour plot of matrix.

## Syntax

`matlab
contour(Z)
contour(X, Y, Z)
`

## Description

- contour(Z) draws isolines of matrix Z.
- contour(X, Y, Z) specifies coordinates X and Y for Z.

## Example

`matlab
[X, Y] = meshgrid(-2:0.1:2);
Z = X.^2 + Y.^2;
contour(X, Y, Z);
`

## References

1. MATLAB Documentation: contour
2. GNU Octave Documentation: contour
