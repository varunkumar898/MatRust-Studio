# Helmholtz Potential & Vector Gradient

**Area**: Calculus  
**Script**: `examples/math/calculus/helmholtz_gradient.m`  

## Mathematical Background

Calculates the gradient vector field and equipotential contour lines of a 2D Gaussian potential Phi(x, y) = exp(-x^2 - y^2).

## Expected Numerical Outcomes & Verification

- **Expected Results**: U, V match central differences within 1e-12; max gradient norm at r=1/sqrt(2) ~ 0.8578
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
