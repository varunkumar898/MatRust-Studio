# `spline`

Cubic spline data interpolation.

## Syntax

```matlab
yy = spline(x, y, xx)
```

## Description

- `yy = spline(x, y, xx)` interpolates values `y` given at distinct grid locations `x` at query points `xx` using cubic splines.
- Constructs piecewise cubic polynomials with continuous second derivatives across interval knots.
