# `odeset`

Create or alter ODE options structure.

## Syntax

```matlab
options = odeset()
options = odeset('Name1', Value1, 'Name2', Value2, ...)
options = odeset(oldopts, 'Name1', Value1, ...)
```

## Description

- `options = odeset()` creates an options structure with all field names initialized to empty defaults.
- `options = odeset('Name1', Value1, ...)` creates an options structure where specified parameters (`RelTol`, `AbsTol`, `InitialStep`, `MaxStep`, etc.) have the assigned values.
- `options = odeset(oldopts, 'Name1', Value1, ...)` updates existing options structure `oldopts` with new parameter values.
