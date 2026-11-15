# `ode23`

Solve non-stiff differential equations with low order method (Bogacki-Shampine 3(2)).

## Syntax

```matlab
[t, y] = ode23(odefun, tspan, y0)
```

## Description

- `[t, y] = ode23(odefun, tspan, y0)` integrates a system of ordinary differential equations $y' = f(t, y)$ from time `tspan(1)` to `tspan(end)` with initial conditions `y0`.
- Uses the adaptive Bogacki-Shampine (2,3) Runge-Kutta pair with step-size adjustment based on local error estimates.
