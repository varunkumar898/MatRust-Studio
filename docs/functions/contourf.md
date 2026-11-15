# contourf

Filled 2-D contour plot.

## Syntax

`matlab
contourf(Z)
contourf(X, Y, Z)
`

## Description

- contourf(Z) draws filled contours of matrix Z.
- contourf(X, Y, Z) specifies coordinates X and Y for Z.

## Example

`matlab
[X, Y] = meshgrid(-2:0.1:2);
Z = sin(X) .* cos(Y);
contourf(X, Y, Z);
`

## References

1. MATLAB Documentation: contourf
2. GNU Octave Documentation: contourf
