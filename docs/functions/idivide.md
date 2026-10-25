# `idivide`

Integer division with directed rounding modes.

## Syntax
```matlab
C = idivide(A, B)
C = idivide(A, B, opt)
```

## Description
Divides integer array `A` by integer array `B` using the rounding option `opt`:
- `'fix'`: Round toward zero (default).
- `'floor'`: Round toward $-\infty$.
- `'ceil'`: Round toward $+\infty$.
- `'round'`: Round to nearest integer, ties rounded away from zero.

## Examples
```matlab
idivide(int32(7), int32(2), 'fix')    % returns 3
idivide(int32(7), int32(2), 'ceil')   % returns 4
idivide(int32(-7), int32(2), 'floor') % returns -4
```
