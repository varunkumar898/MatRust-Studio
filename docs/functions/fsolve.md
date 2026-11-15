# fsolve

Solve system of nonlinear equations.

## Syntax

```matlab
x = fsolve(fun, x0)
[x, fval] = fsolve(fun, x0)
[x, fval, exitflag, output, jacobian] = fsolve(fun, x0, options)
```

## Description

`fsolve(fun, x0)` solves a system of nonlinear equations $F(x) = 0$ starting at `x0`. Uses the Levenberg-Marquardt algorithm with numerical forward-difference Jacobians.

## Examples

Solve system of equations $x_1^2 + x_2^2 - 1 = 0$ and $x_1 - x_2 = 0$:

```matlab
f = @(x) [x(1)^2 + x(2)^2 - 1; x(1) - x(2)];
x = fsolve(f, [0.5; 0.5]);
```

## See Also

[fzero](fzero.md), [fminunc](fminunc.md), [lsqnonlin](lsqnonlin.md), [optimset](optimset.md)
