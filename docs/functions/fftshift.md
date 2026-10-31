# `fftshift`

Shift zero-frequency component to center of spectrum.

## Syntax

```matlab
Y = fftshift(X)
Y = fftshift(X, dim)
```

## Description

- `Y = fftshift(X)`: Rearranges outputs of fft, fft2, or fftn by moving zero-frequency to the center.
- Implemented in RustLab following clean-room MATLAB-compatible specifications.

## References

1. MATLAB Documentation: `fftshift`
2. GNU Octave Documentation: `fftshift`
