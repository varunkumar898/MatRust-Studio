# Continuous State Transition Matrix (expm)

**Area**: Linear Algebra  
**Script**: `examples/math/linear_algebra/matrix_exponential.m`  

## Mathematical Background

Calculates the continuous dynamic state transition matrix expm(A*t) for a 2-DOF damped oscillator state matrix.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Underdamped harmonic decay with natural frequency omega_0 = 2 rad/s and decay envelope exp(-0.25*t)
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
