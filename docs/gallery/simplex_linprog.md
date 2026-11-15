# Two-Phase Simplex Linear Program (linprog)

**Area**: Optimization  
**Script**: `examples/math/optimization/simplex_linprog.m`  

## Mathematical Background

Optimizes manufacturing resource allocation: maximizes profit 3*x1 + 5*x2 subject to labor, material, and demand constraints using two-phase simplex linprog.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Optimal solution x* = [2.0; 6.0] with maximum objective value = 36.0
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
