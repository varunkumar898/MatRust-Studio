# semilogx

Semilogarithmic line plot with logarithmic x-axis.

## Syntax

`matlab
semilogx(X, Y)
`

## Description

- semilogx(X, Y) plots data with a base-10 logarithmic scale on the x-axis and linear scale on the y-axis.

## Example

`matlab
x = 1:100;
y = log(x);
semilogx(x, y);
`

## References

1. MATLAB Documentation: semilogx
2. GNU Octave Documentation: semilogx
