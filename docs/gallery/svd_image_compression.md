# SVD Low-Rank Approximation & Spectrum

**Area**: Linear Algebra  
**Script**: `examples/math/linear_algebra/svd_image_compression.m`  

## Mathematical Background

Decomposes a rank-deficient test matrix into singular values and vectors via svd, showing optimal low-rank truncated approximations.

## Expected Numerical Outcomes & Verification

- **Expected Results**: Dominant singular value sigma(1) ~ 25.1; condition number cond(A) ~ 1e3
- All figures render with informative titles, axis labels, and legends where appropriate.

## Parameters to Vary

- Modify grid resolution, time spans, or solver options to explore system dynamics and stability.
- Adjust parameters to inspect sensitivity, convergence rates, and bifurcation transitions.

## References

1. Burden, R. L., & Faires, J. D. Numerical Analysis.
2. Press, W. H., et al. Numerical Recipes: The Art of Scientific Computing.
3. GNU Octave / MATLAB Reference Documentation.
