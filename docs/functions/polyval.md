# `polyval`

Evaluate polynomial.

## Syntax

```matlab
y = polyval(p, x)
```

## Description

`y = polyval(p, x)` evaluates the polynomial `p` at each point in array `x` using Horner's method.

The polynomial is given by:
$$p(x) = p_1 x^N + p_2 x^{N-1} + \dots + p_N x + p_{N+1}$$

## Implementation Details

- Evaluates polynomial iteratively using Horner's rule:
  $$y = (\dots((p_1 x + p_2) x + p_3) x + \dots) + p_{N+1}$$
- Maintains array shapes and supports single/double numeric precision.
