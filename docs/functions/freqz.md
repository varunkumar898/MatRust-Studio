# `freqz`

Frequency response of digital filter.

## Syntax

```matlab
[h, w] = freqz(b, a)
[h, w] = freqz(b, a, n)
[h, w] = freqz(b, a, n, 'whole')
```

## Description

- `[h, w] = freqz(b, a, n)` returns the complex frequency response vector `h` and frequency vector `w` (in radians/sample) of the digital filter specified by numerator `b` and denominator `a` over `n` frequencies.
- Evaluates transfer function ratio $H(e^{j\omega}) = B(e^{j\omega}) / A(e^{j\omega})$ along the unit circle.
