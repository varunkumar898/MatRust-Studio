# comet

Comet trajectory plot.

## Syntax

`matlab
comet(x, y)
`

## Description

- comet(x, y) displays an animated comet plot of vector y versus vector x.

## Example

`matlab
t = 0:pi/50:10*pi;
x = sin(t);
y = cos(t);
comet(x, y);
`

## References

1. MATLAB Documentation: comet
2. GNU Octave Documentation: comet
