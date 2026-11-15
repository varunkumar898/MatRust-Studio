# lsqnonlin

Solve nonlinear least-squares problem.

## Syntax

```matlab
x = lsqnonlin(fun, x0)
[x, resnorm] = lsqnonlin(fun, x0)
[x, resnorm, residual, exitflag, output] = lsqnonlin(fun, x0)
```

## Description

`lsqnonlin(fun, x0)` solves nonlinear data-fitting and parameter estimation problems of the form $\min_x \sum_i F_i(x)^2$ starting from initial guess `x0` using the Levenberg-Marquardt algorithm.

## Examples

Find root of vector residual:

```matlab
f = @(x) [2*x(1) - 4; 3*x(2) - 9];
x = lsqnonlin(f, [0; 0]);
```

## See Also

[fsolve](fsolve.md), [lsqcurvefit](lsqcurvefit.md), [fminunc](fminunc.md), [optimset](optimset.md)
