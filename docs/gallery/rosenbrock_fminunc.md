# Rosenbrock Valley Minimization (fminunc)

**Area**: Optimization  
**Script**: `examples/math/optimization/rosenbrock_fminunc.m`  

## Mathematical Background

Finds the global minimum of the notorious banana-valley Rosenbrock objective f(x, y) = 100*(y - x^2)^2 + (1 - x)^2 starting from [-1.2, 1.0] using BFGS fminunc.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Global minimizer x* = [1.0, 1.0] with f(x*) = 0.0 within tolerance 1e-4
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
