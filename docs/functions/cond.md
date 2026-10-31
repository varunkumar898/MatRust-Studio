# `cond`

Condition number with respect to inversion (2-norm).

## Syntax

```matlab
c = cond(A)
```

## Description

- `c = cond(A)` returns the 2-norm condition number, defined as the ratio of the largest to smallest singular value:
  $$c = \frac{\sigma_{\max}}{\sigma_{\min}}$$
- Returns `Inf` strictly when $\sigma_{\min} == 0.0$ (exact singularity). For ill-conditioned or numerically near-singular matrices where $\sigma_{\min} > 0$, `cond` returns a finite large floating point number.
