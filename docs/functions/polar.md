# polar

Polar coordinate plot.

## Syntax

`matlab
polar(theta, rho)
`

## Description

- polar(theta, rho) plots a line in polar coordinates, with angle theta in radians and radius rho.

## Example

`matlab
theta = 0:0.01:2*pi;
rho = sin(2*theta);
polar(theta, rho);
`

## References

1. MATLAB Documentation: polar
2. GNU Octave Documentation: polar
