# Mandelbrot Escape-Time Fractal Set

**Area**: Fractals & Chaos  
**Script**: `examples/math/fractals/mandelbrot_set.m`  

## Mathematical Background

Computes the classical Mandelbrot fractal escape-time field z_{n+1} = z_n^2 + c over the complex plane and renders with scaled colormap.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Boundary of main cardioid at Re(c) = 0.25, period-2 bulb at Re(c) = -0.75 cleanly mapped
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
