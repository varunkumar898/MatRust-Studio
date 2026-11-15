# stem

Plot discrete sequence data with stem and markers.

## Syntax

`matlab
stem(Y)
stem(X, Y)
`

## Description

- stem(Y) plots the data sequence Y as stems extending from the baseline.
- stem(X, Y) plots Y against X.

## Example

`matlab
x = 0:0.1:2*pi;
y = sin(x);
stem(x, y);
`

## References

1. MATLAB Documentation: stem
2. GNU Octave Documentation: stem
