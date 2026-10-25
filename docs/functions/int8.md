# `int8` (and `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`)

Convert numeric array to integer class with round half away from zero and saturating arithmetic.

## Syntax
```matlab
Y = int8(X)
Y = uint8(X)
Y = int32(X)
Y = int64(X)
```

## Description
Converts floating point or other numeric values into integer classes.
- Conversions round half away from zero (`int8(2.5) == 3`, `int8(-2.5) == -3`).
- Arithmetic saturates without overflow or wrapping (`uint8(250) + 10 == 255`, `uint8(3) - 5 == 0`).
- Unary negation saturates: `-int8(-128) == 127`.

## Examples
```matlab
u = uint8(250) + uint8(20);  % returns uint8(255)
```
