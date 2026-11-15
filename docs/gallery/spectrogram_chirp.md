# Linear Chirp Time-Frequency Spectrogram

**Area**: Signal Processing  
**Script**: `examples/math/signal_processing/spectrogram_chirp.m`  

## Mathematical Background

Computes the short-time Fourier transform (STFT) spectrogram of a frequency-modulated linear chirp signal sweeping from 20 Hz to 200 Hz.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Linear ridge with slope (200 - 20) / 1.0 = 180 Hz/s clearly resolved in time-frequency plane
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
