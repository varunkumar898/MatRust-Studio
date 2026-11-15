# `deconv`

Deconvolution and polynomial division.

## Syntax

```matlab
[q, r] = deconv(b, a)
```

## Description

- `[q, r] = deconv(b, a)` deconvolves vector `a` out of vector `b` using polynomial long division.
- Solves for quotient `q` and remainder `r` satisfying $b = \text{conv}(a, q) + r$.
