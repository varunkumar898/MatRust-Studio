# Nonlinear Pendulum Phase Portrait (ode23)

**Area**: ODE  
**Script**: `examples/math/ode/damped_pendulum.m`  

## Mathematical Background

Solves large-angle nonlinear pendulum oscillations theta'' + 0.1*theta' + sin(theta) = 0 and tracks energy dissipation.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Damped spiral spiraling asymptotically to fixed point (0, 0)
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
