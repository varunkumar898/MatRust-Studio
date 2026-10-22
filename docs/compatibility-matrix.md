# Compatibility matrix (milestone 1)

Status marks: **implemented** (tested), *partial* (works, documented divergences),
**not implemented** (future work). "MATLAB" column describes observed/documented
R2026b behavior this row tracks; this matrix makes no claim beyond that.

| Feature | Status | Notes |
|---|---|---|
| Array construction: `zeros` `ones` `eye` | implemented | 1-based, 2-D; negative/fractional dims rejected |
| `linspace` `logspace` `diag` `repmat` `magic` | implemented | column-major order; magic: Siamese/doubly-even/n=2; singly-even errors |
| `rand` `randn` `randi` `randperm` `rng(seed)` | implemented | session xoshiro256++; Box–Muller randn; Fisher–Yates randperm; deterministic seed |
| `size` `length` `numel` `ndims` `isempty` | implemented | `ndims` always 2 (2-D model) |
| `reshape` `transpose` `flip` `fliplr` `flipud` `rot90` `circshift` `triu` `tril` | implemented | 2-D only; N-D overloads rejected |
| Concatenation: `vertcat` `horzcat` `cat` `blkdiag` | implemented | dim 1/2 only; dimension consistency checked; block-diagonal zero fill |
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
| `if/else/elseif`, `for`, `while`, `switch`, `break`/`continue` | implemented | strict scalar conditions; `break`/`continue` error outside loop; `for` iterates columns; `end` block matching verified; empty/negative ranges work |
| `:` ranges, `end` keyword | implemented | `start:step:stop`; `end` in indices reserved (context required) |
| Function definitions, scopes, scripts+functions | partial | `function` keyword parsed; local scopes not fully enforced |
| Function definitions, scopes, scripts+functions | not implemented | next milestone |
| N-D arrays, sparse, cell/table types | not implemented | |
| Display formatting `format short` etc. | *partial* | own formatter; integer/fixed-4 approximates MATLAB layout |
| JITAOT/JIT, plotting, desktop | not implemented | later phases |

## Batch 4 additions (2026-10-22)
| `sort` `sortrows` | implemented | column/row sort; default first non-singleton dim; NaN sorts last |
| `unique` | implemented | sorted unique values (column vector) |
| `ismember` `setdiff` `union` `intersect` | implemented | sorted set operations; 0/1 membership; column vectors |
| `find` | implemented | linear indices (column-major); multi-output [r,c] and [r,c,v] |
| `any` `all` | implemented | per-column reduction; 1xN output; logical as 0/1 doubles |
| `cumsum` `cumprod` | implemented | column-wise cumulative; preserves dimensions |
| `diff` | impl-ptest | single difference along columns; n-times and dim not yet supported |
| Colon operator `a:b` `a:step:b` | implemented | positive/negative steps; empty ranges; `1:0` -> empty |
| Colon indexing `A(i,:)` `A(:,j)` `A(end,:)` | implemented | returns row/column vectors; `end` keyword supported |

## Batch 6 & 7 additions (2026-10-22)
| Feature | Status | Notes |
|---|---|---|
| Base conversions: `dec2bin`, `bin2dec`, `dec2hex`, `hex2dec` | implemented | scalar and array inputs; 32-bit two's complement for negative integers (padded with 1/F); floor semantics for fractional inputs; width padding; multi-row text round-trips to numeric column vectors |
| Signum: `sign` | implemented | element-wise sign extraction returning -1, 0, 1, or NaN; scalar and array inputs |
| Degree inverse trigonometry: `asind`, `acosd`, `atand` | implemented | degree outputs; domain boundaries `[-1, 1]`; out-of-domain produces NaN; scalar and array inputs |
| Exact degree trigonometry: `sind`, `cosd`, `tand` | implemented | exact zeros for multiples of 90/180 degrees; standard IEEE-754 for other angles |
| Prime factorization: `factor` | implemented | returns 1xN row vector of prime factors; reconstructs input via multiplication; `factor(1)==[1]`; invalid/non-integer/non-positive inputs rejected |
| Real nth root: `nthroot` | implemented | real nth root; odd roots of negative numbers supported (`nthroot(-8,3)==-2`); even roots of negative numbers error; scalar expansion |
| Modulus and Remainder: `mod`, `rem` | implemented | `mod(x,0)==x`, `rem(x,0)==NaN`; sign conventions preserved (`mod` follows divisor, `rem` follows dividend); scalar expansion |
| Combinatorics: `factorial`, `nchoosek` | implemented | float accumulation avoiding early integer overflow; `nchoosek` uses symmetry; `factorial(171)==Inf` |
| Bitwise operations: `bitand`, `bitor`, `bitxor`, `bitshift` | implemented | safe shift logic preventing overflow panics; 3-arg `bitshift(A,k,n)` mask; scalar expansion; integer validation |
| Complex number model | designed | architectural design specification in `docs/architecture/complex-numbers.md` |

## Batch 8 additions (2026-10-22)
| Feature | Status | Notes |
|---|---|---|
| Central tendency: `mean`, `median`, `mode` | implemented | default dimension reduction (dim 1 for matrices, dim 2 for row vectors); explicit dimension argument; `'all'` global reduction; `mode` resolves ties to smallest value; empty inputs return NaN |
| Measures of spread: `std`, `var` | implemented | normalization weights `w=0` (sample, $N-1$) and `w=1` (population, $N$); dimension and `'all'` options; constant data returns 0.0; singletons return 0.0; empty arrays return NaN |
| Measures of spread: `range`, `iqr` | implemented | `range = max - min`; `iqr` uses empirical quantile linear interpolation ($r = 0.5 + N \times p$) matching MATLAB; dimension and `'all'` support; empty inputs return NaN |
| Products and Sums: `prod`, `sum` | implemented | default, explicit dim, and `'all'` reduction; empty `sum([]) == 0.0`; empty `prod([]) == 1.0` |
| Minima and Maxima: `min`, `max` | implemented | 1-arg reduction along dim or `'all'`; 2-arg element-wise pairwise `min(A,B)` / `max(A,B)` with scalar expansion; 2-output `[M, I] = min/max(...)` returning 1-based column-major indices |
