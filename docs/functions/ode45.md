# `ode45`

Solve non-stiff differential equations with medium order method (Dormand-Prince 5(4)).

## Syntax

```matlab
[t, y] = ode45(odefun, tspan, y0)
```

## Description

- `[t, y] = ode45(odefun, tspan, y0)` integrates a system of ordinary differential equations $y' = f(t, y)$ from time `tspan(1)` to `tspan(end)` with initial conditions `y0`.
- `odefun` is a function handle `@(t, y) ...` returning a column vector of derivatives $y'$.
- Uses the adaptive Dormand-Prince (4,5) Runge-Kutta pair with local truncation error estimation and adaptive step-size selection.
