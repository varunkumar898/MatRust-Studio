# Nonlinear System Equilibrium (fsolve)

**Area**: Optimization  
**Script**: `examples/math/optimization/nonlinear_fsolve.m`  

## Mathematical Background

Solves a system of coupled non-linear equations [x1^2 + x2^2 - 4; exp(x1) + x2 - 1] = 0 using the Levenberg-Marquardt fsolve algorithm.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Roots satisfy ||f(x)|| < 1e-6 with x1 ~ 0.887, x2 ~ -1.428
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
