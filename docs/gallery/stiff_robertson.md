# Stiff Robertson Chemical Kinetics (ode15s)

**Area**: ODE  
**Script**: `examples/math/ode/stiff_robertson.m`  

## Mathematical Background

Integrates the stiff Robertson autocatalytic reaction system spanning decades from t=1e-5 to t=1e4 s using backward differentiation formula ode15s.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Mass conservation y(1)+y(2)+y(3) = 1.0 preserved within 1e-6 across all steps
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
