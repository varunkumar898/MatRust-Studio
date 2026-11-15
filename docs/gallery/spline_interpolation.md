# Runge Interpolation: Spline vs PCHIP

**Area**: Signal Processing  
**Script**: `examples/math/signal_processing/spline_interpolation.m`  

## Mathematical Background

Compares cubic spline interpolation against shape-preserving piecewise cubic Hermite interpolation (pchip) on Runge's test function.

## Expected Numerical Outcomes & Verification

- **Expected Results**: PCHIP preserves monotonicity without overshoot; Spline achieves C2 continuity
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
