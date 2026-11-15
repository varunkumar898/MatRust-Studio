# `spectrogram`

Spectrogram using Short-Time Fourier Transform.

## Syntax

```matlab
[s, f, t] = spectrogram(x)
[s, f, t] = spectrogram(x, window, noverlap, nfft, fs)
```

## Description

- `[s, f, t] = spectrogram(x, window, noverlap, nfft, fs)` computes the short-time Fourier transform matrix `s`, frequency vector `f`, and time center vector `t` for input signal `x`.
- Divides `x` into overlapping windowed segments and applies discrete Fourier transforms across time frames.
