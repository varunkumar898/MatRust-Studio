# Discrete Laplacian of Thermal Field

**Area**: Calculus  
**Script**: `examples/math/calculus/laplacian_del2.m`  

## Mathematical Background

Evaluates the discrete 2D Laplacian (del2) of a 2D sinusoidal temperature distribution T(x, y) = sin(pi*x)*sin(pi*y).

## Expected Numerical Outcomes & Verification

- **Expected Results**: L matches -0.5*pi^2*T at interior points within 1e-2
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
