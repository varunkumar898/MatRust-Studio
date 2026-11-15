# Overdetermined Least Squares via QR

**Area**: Linear Algebra  
**Script**: `examples/math/linear_algebra/qr_least_squares.m`  

## Mathematical Background

Fits a quadratic polynomial to noisy data points by solving the overdetermined system A*x = b using QR decomposition.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Optimal coefficients c ~ [1.02, 1.25, 0.98]; residual norm < 0.2
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
