# fminunc

Find minimum of unconstrained multivariable function.

## Syntax

```matlab
x = fminunc(fun, x0)
[x, fval] = fminunc(fun, x0)
[x, fval, exitflag, output, grad, hessian] = fminunc(fun, x0, options)
```

## Description

`fminunc(fun, x0)` finds the local minimum of a multivariable objective function starting at initial point `x0`. Uses a clean-room BFGS quasi-Newton algorithm with Armijo backtracking line search and numerical gradients.

## Examples

Minimize 2D quadratic function:

```matlab
f = @(x) (x(1) - 3)^2 + (x(2) + 1)^2;
x = fminunc(f, [0; 0]);
```

## See Also

[fminsearch](fminsearch.md), [fminbnd](fminbnd.md), [fsolve](fsolve.md), [optimset](optimset.md)
