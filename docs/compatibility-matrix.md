# Compatibility matrix (milestone 1)

Status marks: **implemented** (tested), *partial* (works, documented divergences),
**not implemented** (future work). "MATLAB" column describes observed/documented
R2026b behavior this row tracks; this matrix makes no claim beyond that.

| Feature | Status | Notes |
|---|---|---|
| Numeric literals, arithmetic precedence | implemented | `^` binds before `*`; right-associative power |
| Variables, assignment, `;` suppression | implemented | case-sensitive names |
| 2-D matrix literals `[1 2; 3 4]` | implemented | scalar elements only; no nested concat yet |
| Element access `A(i,j)`, `A(k)` | implemented | 1-based; linear index column-major |
| Index assignment with growth | implemented | 2-D grows zero-filled; `A(5)=v` on 2x2 grows to 1x5 |
| Element-wise `.*` `./` `.^` | implemented | never falls back to scalar broadcast |
| Matrix `*` `/` `^` | implemented | `/` via explicit inverse (Gauss-Jordan); NOT yet a stable solve |
| Comparisons `==` `~=` `<` `<=` `>` `>=` | implemented | result is 0/1 **double**, not logical type |
| Transpose `'` and `.'` | implemented | real values only (no conjugate) |
| `disp`, `size` (+dim), `zeros/ones/eye`, `length`, `numel` | implemented | |
| `abs sqrt floor ceil round exp log sum max min pi` | implemented | `sum/max/min` reduce along columns (MATLAB default) |
| `1./0` → `Inf`, IEEE zero division | implemented | matches MATLAB double semantics |
| Char literals `'abc'`, `''` escape | implemented | |
| Char arithmetic/indexing | not implemented | strings stored, ops rejected with diagnostic |
| logical type | not implemented | represented as 0/1 doubles (divergence) |
| Complex numbers | not implemented | `sqrt(-1)` → NaN (MATLAB: `0 + 1i`) |
| `:` ranges, `end` keyword | not implemented | colon token reserved, no evaluation |
| `if/else`, `for`, `while`, `switch` | not implemented | next milestone |
| Function definitions, scopes, scripts+functions | not implemented | next milestone |
| N-D arrays, sparse, cell/table types | not implemented | |
| Display formatting `format short` etc. | *partial* | own formatter; integer/fixed-4 approximates MATLAB layout |
| JITAOT/JIT, plotting, desktop | not implemented | later phases |
