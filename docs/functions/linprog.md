# linprog

Solve linear programming problems.

## Syntax

```matlab
x = linprog(f, A, b)
x = linprog(f, A, b, Aeq, beq)
x = linprog(f, A, b, Aeq, beq, lb, ub)
[x, fval, exitflag, output] = linprog(f, A, b, Aeq, beq, lb, ub)
```

## Description

`linprog` solves linear programming problems of the form:

$$\min_x f^T x \quad \text{subject to} \quad A x \le b, \quad A_{eq} x = b_{eq}, \quad lb \le x \le ub$$

Uses a clean-room Two-Phase Simplex algorithm with Bland's anti-cycling pivot selection rule.

## Examples

Solve linear program:

```matlab
f = [-1; -2];
A = [1, 1; 2, 1];
b = [4; 5];
lb = [0; 0];
[x, fval] = linprog(f, A, b, [], [], lb);
```

## See Also

[fminunc](fminunc.md), [fminsearch](fminsearch.md), [optimset](optimset.md)
