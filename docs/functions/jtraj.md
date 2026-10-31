# `jtraj`

Compute a joint trajectory between two configurations.

## Syntax

```matlab
q = jtraj(q0, q1, n)
```

## Description

`q = jtraj(q0, q1, n)` generates a smooth quintic polynomial trajectory between initial joint coordinate `q0` and final joint coordinate `q1` over `n` steps with zero boundary velocities and accelerations.
