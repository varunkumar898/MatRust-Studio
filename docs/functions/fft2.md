# `fft2` and `ifft2`

Two-dimensional discrete Fourier transform and its inverse.

## Syntax

```matlab
Y = fft2(X)
X = ifft2(Y)
```

## Description

- `Y = fft2(X)` computes the two-dimensional discrete Fourier transform of matrix `X` using separable 1D Fourier transforms along columns then rows.
- `X = ifft2(Y)` computes the two-dimensional inverse Fourier transform, scaled by $\frac{1}{M \cdot N}$.
- Powered by `rustfft` supporting both power-of-2 and arbitrary non-power-of-2 matrix shapes.
