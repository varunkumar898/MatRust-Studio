# Van der Pol Relaxation Oscillator (ode45)

**Area**: ODE  
**Script**: `examples/math/ode/van_der_pol.m`  

## Mathematical Background

Simulates the non-linear Van der Pol oscillator d2y/dt2 - mu*(1 - y^2)*dy/dt + y = 0 with mu = 1.0 using the adaptive Runge-Kutta Dormand-Prince ode45 solver.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Converges to limit cycle with amplitude ~ 2.0 and period ~ 6.66 s
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
