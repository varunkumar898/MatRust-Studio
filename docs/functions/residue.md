# `residue`

Partial fraction expansion (residues, poles, and direct term).

## Syntax

```matlab
[r, p, k] = residue(b, a)
```

## Description

- `[r, p, k] = residue(b, a)` finds the partial fraction expansion of the ratio of polynomials $B(s) / A(s)$.
- Returns residues `r`, poles `p`, and polynomial direct term quotient `k`.
