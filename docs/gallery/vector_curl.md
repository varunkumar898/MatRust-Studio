# 2D Vortex Flow Field & Curl

**Area**: Calculus  
**Script**: `examples/math/calculus/vector_curl.m`  

## Mathematical Background

Models a circulating vortex velocity field (u, v) = (-y/(r^2+0.1), x/(r^2+0.1)) and visualizes velocity streamlines.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Vortex velocity norm peaks at r ~ sqrt(0.1) ~ 0.316 with magnitude ~ 1.581
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
