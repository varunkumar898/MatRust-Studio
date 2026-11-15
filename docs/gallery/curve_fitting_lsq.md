# Nonlinear Exponential Decay Curve Fit

**Area**: Optimization  
**Script**: `examples/math/optimization/curve_fitting_lsq.m`  

## Mathematical Background

Fits experimental exponential radioactive decay data y(t) = a * exp(-b * t) + c to determine decay rate b using lsqcurvefit.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Optimal parameters p ~ [4.65, 0.95, 0.48]; R^2 goodness of fit > 0.99
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
