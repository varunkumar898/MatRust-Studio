# `ode15s`

Solve stiff differential equations and DAEs, variable order method.

## Syntax

```matlab
[t, y] = ode15s(odefun, tspan, y0)
[t, y] = ode15s(odefun, tspan, y0, options)
```

## Description

- `[t, y] = ode15s(odefun, tspan, y0)` integrates a system of stiff ordinary differential equations $y' = f(t, y)$ from `tspan(1)` to `tspan(end)` with initial conditions `y0`.
- Employs variable-step implicit numerical differentiation formulas (BDF) with modified Newton iteration and numerical Jacobian evaluation for stiff dynamical systems.
- `[t, y] = ode15s(odefun, tspan, y0, options)` integrates using parameters specified in `options` (created with `odeset`).
