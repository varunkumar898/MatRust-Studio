# ill

Filled 2-D polygonal region.

## Syntax

`matlab
fill(X, Y, C)
`

## Description

- ill(X, Y, C) creates filled polygons from the data in X and Y with color C.

## Example

`matlab
t = (1/16:1/8:1)' * 2 * pi;
x = sin(t);
y = cos(t);
fill(x, y, 'r');
`

## References

1. MATLAB Documentation: ill
2. GNU Octave Documentation: ill
