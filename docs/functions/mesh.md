# mesh

3D wireframe mesh plot.

## Syntax

`matlab
mesh(X, Y, Z)
mesh(Z)
`

## Description

- mesh(X, Y, Z) plots a 3D wireframe surface with colors determined by Z.
- mesh(Z) uses default grid coordinates.

## Example

`matlab
[X, Y] = meshgrid(-2:0.2:2);
Z = X .* exp(-X.^2 - Y.^2);
mesh(X, Y, Z);
`

## References

1. MATLAB Documentation: mesh
2. GNU Octave Documentation: mesh
