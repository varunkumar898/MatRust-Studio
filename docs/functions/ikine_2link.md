# `ikine_2link`

Inverse kinematics for 2-link planar arm.

## Syntax

```matlab
[q1, q2] = ikine_2link(l1, l2, x, y)
```

## Description

`[q1, q2] = ikine_2link(l1, l2, x, y)` computes the joint angles `q1` and `q2` required for a planar 2-link robot arm with link lengths `l1` and `l2` to reach Cartesian position $(x, y)$.
