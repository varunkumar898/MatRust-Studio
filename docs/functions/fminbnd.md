# fminbnd

Find minimum of single-variable function on fixed interval.

## Syntax

```matlab
x = fminbnd(fun, x1, x2)
[x, fval] = fminbnd(fun, x1, x2)
[x, fval, exitflag, output] = fminbnd(fun, x1, x2, options)
```

## Description

`fminbnd(fun, x1, x2)` returns a value `x` that is a local minimizer of the scalar function `fun` over the interval `[x1, x2]`. It uses Brent's method combining golden section search and parabolic interpolation.

## Examples

Find minimum of $f(x) = (x - 2)^2$:

```matlab
f = @(x) (x - 2)^2;
x = fminbnd(f, 0, 5);
```

## See Also

[fzero](fzero.md), [fminsearch](fminsearch.md), [fminunc](fminunc.md), [optimset](optimset.md)
