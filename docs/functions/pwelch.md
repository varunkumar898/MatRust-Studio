# `pwelch`

Welch's power spectral density estimate.

## Syntax

```matlab
[pxx, f] = pwelch(x)
[pxx, f] = pwelch(x, window)
[pxx, f] = pwelch(x, window, noverlap)
[pxx, f] = pwelch(x, window, noverlap, nfft)
[pxx, f] = pwelch(x, window, noverlap, nfft, fs)
```

## Description

- `[pxx, f] = pwelch(x, window, noverlap, nfft, fs)` estimates the power spectral density `pxx` of vector `x` using Welch's averaged modified periodogram method.
- Evaluates periodograms across overlapping windowed segments to reduce spectral variance.
