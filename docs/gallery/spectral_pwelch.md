# Welch Power Spectral Density Estimation

**Area**: Signal Processing  
**Script**: `examples/math/signal_processing/spectral_pwelch.m`  

## Mathematical Background

Estimates the power spectral density (PSD) of a multi-tone signal embedded in Gaussian white noise using pwelch.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Distinct spectral peaks detected at 100 Hz and 250 Hz rising > 15 dB above noise floor
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
