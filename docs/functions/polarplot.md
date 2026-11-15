# polarplot

Plot lines in polar coordinates.

## Syntax

`matlab
polarplot(theta, rho)
`

## Description

- polarplot(theta, rho) plots lines in polar coordinates with theta in radians.

## Example

`matlab
theta = linspace(0, 2*pi, 100);
r = 1 + cos(theta);
polarplot(theta, r);
`

## References

1. MATLAB Documentation: polarplot
2. GNU Octave Documentation: polarplot
