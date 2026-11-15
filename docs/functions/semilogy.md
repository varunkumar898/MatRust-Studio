# semilogy

Semilogarithmic line plot with logarithmic y-axis.

## Syntax

`matlab
semilogy(X, Y)
`

## Description

- semilogy(X, Y) plots data with a base-10 logarithmic scale on the y-axis and linear scale on the x-axis.

## Example

`matlab
x = 0:10;
y = exp(x);
semilogy(x, y);
`

## References

1. MATLAB Documentation: semilogy
2. GNU Octave Documentation: semilogy
