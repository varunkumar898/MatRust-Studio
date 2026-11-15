# xis

Set axis limits and aspect ratio.

## Syntax

`matlab
axis auto
axis tight
axis equal
axis([xmin xmax ymin ymax])
`

## Description

- xis controls axis scaling, limits, and appearance.

## Example

`matlab
x = 0:0.1:10;
plot(x, sin(x));
axis([0 10 -1.5 1.5]);
`

## References

1. MATLAB Documentation: xis
2. GNU Octave Documentation: xis
