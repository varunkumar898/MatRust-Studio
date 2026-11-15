# 2D Adaptive Quadrature Integral

**Area**: Calculus  
**Script**: `examples/math/calculus/adaptive_quadrature.m`  

## Mathematical Background

Computes the 2D surface integral of f(x, y) = x*y*exp(-x^2 - y^2) over the unit square [0, 1] x [0, 1] using integral2.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Exact analytical value I = (1 - exp(-1))^2 / 4 ~ 0.1000305; error < 1e-6
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
