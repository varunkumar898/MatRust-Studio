# quiver

Quiver plot of 2-D velocity vectors.

## Syntax

`matlab
quiver(u, v)
quiver(x, y, u, v)
`

## Description

- quiver(u, v) plots velocity vectors as arrows at equally spaced points in the x-y plane.
- quiver(x, y, u, v) plots vectors (u, v) at locations (x, y).

## Example

`matlab
[x, y] = meshgrid(-2:0.5:2, -2:0.5:2);
u = -y;
v = x;
quiver(x, y, u, v);
`

## References

1. MATLAB Documentation: quiver
2. GNU Octave Documentation: quiver
