# `pchip`

Piecewise Cubic Hermite Interpolating Polynomial.

## Syntax

```matlab
yy = pchip(x, y, xx)
```

## Description

- `yy = pchip(x, y, xx)` interpolates values `y` given at distinct grid locations `x` at query points `xx` using shape-preserving cubic Hermite polynomials (Fritsch & Carlson 1980).
- Guarantees monotonicity preservation and prevents overshoot between data points.
