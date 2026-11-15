# Logistic Map Period-Doubling Bifurcation

**Area**: Fractals & Chaos  
**Script**: `examples/math/fractals/bifurcation_diagram.m`  

## Mathematical Background

Maps the period-doubling route to chaos in the non-linear discrete logistic map x_{n+1} = r * x_n * (1 - x_n) for parameter r in [2.5, 4.0].

## Expected Numerical Outcomes & Verification

- **Expected Results**: First bifurcation at r=3.0, second at r=1+sqrt(6) ~ 3.449, onset of chaos at r_infinity ~ 3.5699
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
