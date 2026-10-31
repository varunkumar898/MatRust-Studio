# `single`

Convert numeric array to single-precision 32-bit floating-point format.

## Syntax
```matlab
Y = single(X)
```

## Description
`Y = single(X)` converts the numeric array `X` to single precision (`f32`). If `X` is already single precision, it returns `X` unchanged.

## Examples
```matlab
s = single([1.5, 2.7]);
class(s)  % returns 'single'
```
