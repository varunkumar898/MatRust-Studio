# optimset

Create or modify optimization options structure.

## Syntax

```matlab
options = optimset()
options = optimset('solver_name')
options = optimset('param1', value1, 'param2', value2, ...)
options = optimset(oldopts, 'param1', value1, ...)
```

## Description

`optimset` creates or modifies an options structure controlling the execution of optimization algorithms such as `fminsearch`, `fminbnd`, `fminunc`, `fsolve`, and `linprog`.

## Examples

Create default options for `fminunc` with custom tolerance:

```matlab
opts = optimset('fminunc', 'TolFun', 1e-8, 'MaxIter', 1000);
```

## See Also

[fminsearch](fminsearch.md), [fminbnd](fminbnd.md), [fminunc](fminunc.md), [fsolve](fsolve.md), [odeset](odeset.md)
