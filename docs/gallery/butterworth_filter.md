# Butterworth Digital Filter & freqz Response

**Area**: Signal Processing  
**Script**: `examples/math/signal_processing/butterworth_filter.m`  

## Mathematical Background

Designs a 2nd-order Butterworth low-pass digital filter and computes its complex frequency response H(e^jw) using freqz.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Passband gain 0 dB at DC (w=0); -3 dB cutoff at w ~ 0.2*pi; monotonically decreasing magnitude
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
