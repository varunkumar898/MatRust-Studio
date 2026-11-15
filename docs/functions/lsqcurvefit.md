# lsqcurvefit

Solve nonlinear curve-fitting problem in least-squares sense.

## Syntax

```matlab
x = lsqcurvefit(fun, x0, xdata, ydata)
[x, resnorm] = lsqcurvefit(fun, x0, xdata, ydata)
[x, resnorm, residual, exitflag, output] = lsqcurvefit(fun, x0, xdata, ydata)
```

## Description

`lsqcurvefit(fun, x0, xdata, ydata)` fits nonlinear function model `fun(x, xdata)` to observed data `ydata` using the Levenberg-Marquardt algorithm.

## Examples

Fit an exponential model $y = a e^{b x}$:

```matlab
xdata = [0, 1, 2, 3];
ydata = [1.0, 2.7, 7.3, 20.0];
model = @(p, x) p(1) * exp(p(2) * x);
p_fit = lsqcurvefit(model, [1.0; 0.5], xdata, ydata);
```

## See Also

[lsqnonlin](lsqnonlin.md), [fsolve](fsolve.md), [polyfit](polyfit.md), [optimset](optimset.md)
