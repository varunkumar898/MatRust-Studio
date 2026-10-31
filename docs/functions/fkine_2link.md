# `fkine_2link`

Forward kinematics for 2-link planar arm.

## Syntax

```matlab
[x, y] = fkine_2link(l1, l2, q1, q2)
```

## Description

`[x, y] = fkine_2link(l1, l2, q1, q2)` computes the Cartesian end-effector coordinates $(x, y)$ of a planar 2-link robot arm with link lengths `l1`, `l2` and joint angles `q1`, `q2`:
$$x = l_1 \cos(q_1) + l_2 \cos(q_1 + q_2)$$
$$y = l_1 \sin(q_1) + l_2 \sin(q_1 + q_2)$$
