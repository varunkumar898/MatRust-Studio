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
| Transpose `'` and `.'` | implemented | `'` is Hermitian conjugate transpose; `.'` is non-conjugate transpose (agree on real matrices) |
| `disp`, `size` (+dim), `zeros/ones/eye`, `length`, `numel` | implemented | supports both real and complex matrices |
| `abs sqrt floor ceil round exp log sum max min pi` | implemented | `abs`, `sqrt`, `exp`, `log` support complex numbers; `sqrt(-4)` yields `0 + 2i` |
| `1./0` → `Inf`, IEEE zero division | implemented | matches MATLAB double semantics |
| Char literals `'abc'`, `''` escape | implemented | |
| Char arithmetic/indexing | not implemented | strings stored, ops rejected with diagnostic |
| logical type | not implemented | represented as 0/1 doubles (divergence) |
| Complex numbers | implemented | genuine `Complex64` and `ComplexDenseMatrix`; interleaved column-major storage; automatic real promotion |
| `:` ranges, `end` keyword | not implemented | colon token reserved, no evaluation |
| `if/else/elseif`, `for`, `while`, `switch`, `break`/`continue` | implemented | strict scalar conditions; `break`/`continue` error outside loop; `for` iterates columns; `end` block matching verified; empty/negative ranges work |
| `:` ranges, `end` keyword | implemented | `start:step:stop`; `end` in indices reserved (context required) |
| Function definitions, scopes, scripts+functions | partial | `function` keyword parsed; local scopes not fully enforced |
| Function definitions, scopes, scripts+functions | not implemented | next milestone |
| N-D arrays, sparse, cell/table types | not implemented | |
| Display formatting `format short` etc. | *partial* | own formatter; integer/fixed-4 approximates MATLAB layout |
| JITAOT/JIT, plotting, desktop | implemented (plotting infrastructure verified) | `GraphicsState`, `FigureState`, `PlotItem`, `egui_plot` renderer; plot/scatter builtins; plot stage 4 verified; desktop rendering `NOT TESTED` live (non-interactive) |

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

## Batch 9 additions (2026-10-22)
| Feature | Status | Notes |
|---|---|---|
| Matrix left division: `\`, `mldivide` | implemented | Solves linear system $A * X = B$. Square nonsingular systems solved via pivoted LU with forward/backward substitution; overdetermined systems solved via Householder QR least squares; underdetermined via SVD pseudoinverse; scalar broadcasting supported; strict error on singular matrices |
| Element-wise left division: `.\`, `ldivide` | implemented | Equivalent to `B ./ A`; supports scalar broadcasting |
| Matrix right division: `/`, `mrdivide` | implemented | Solves $X * B = A$ via $(B' \setminus A')'$; replaces explicit Gauss-Jordan inversion |
| LU factorization: `lu` | implemented | Partial row pivoting; supports 1-output compact $Y$, 2-output $[L, U]$ where $A = L * U$ ($L$ permuted), and 3-output $[L, U, P]$ where $P * A = L * U$ with unit lower triangular $L$ |
| Determinant: `det` | implemented | Computed via pivoted LU; accounts for permutation parity ($(-1)^{\text{swaps}}$) and diagonal pivot product; $0.0$ for singular matrices; $1.0$ for empty matrix |
| Matrix inverse: `inv` | implemented | Solves $A * X = I_n$ via pivoted LU forward/backward substitution; errors on singular matrices without fallback to pseudoinverse |
| Matrix trace: `trace` | implemented | Sum of main diagonal elements for square and rectangular matrices; returns $0.0$ for empty matrices |
| Singular value decomposition: `svd` | implemented | One-sided Jacobi (Hestenes method) with Gram-Schmidt orthonormal completion; 1-output $s$ (descending singular values column vector) and 3-output $[U, S, V]$ satisfying $U * S * V' \approx A$ with orthonormal $U$ and $V$ |
| Numerical rank: `rank` | implemented | Number of singular values greater than tolerance; default tolerance $\max(m,n) \times \sigma_{\max} \times \text{eps}$; explicit tolerance parameter supported |
| Moore–Penrose pseudoinverse: `pinv` | implemented | Computed via SVD ($V * S^+ * U'$); satisfies all 4 Moore–Penrose identities; handles rectangular and rank-deficient matrices; explicit tolerance supported |
| Vector & matrix norms: `norm` | implemented | Vector norms (1, 2, $\infty$, $-\infty$, general $p$); matrix norms (1 max column sum, 2 largest singular value via SVD, $\infty$ max row sum, 'fro' Frobenius norm) |
| Operators: `\` and `.\` | implemented | Lexer tokens `Backslash` and `DotBackslash`, AST `BinaryOp::LDiv` and `BinaryOp::DotLDiv`, parser infix bindings (precedence 60, 61) |

## Batch 10 additions (2026-10-22)
| Feature | Status | Notes |
|---|---|---|
| Complex value representation | implemented | `Complex64` struct (`#[repr(C)]` interleaved `re`, `im: f64`); `ComplexDenseMatrix` with column-major 2-D indexing |
| Runtime value variant | implemented | `Value::ComplexMatrix(ComplexDenseMatrix)` in `matlab_values`; scalar complex numbers represented as 1x1 complex matrices |
| Imaginary literals and constants | implemented | Lexer recognizes `2i`, `3.5j`, `1e-3i` as `TokenKind::ImaginaryNumber`; AST `Expr::Imaginary`; constants `i` and `j` evaluate to `0 + 1i` when unshadowed |
| Complex constructors | implemented | `complex(re)` and `complex(re, im)` with scalar broadcasting; matrix literal `[1 + 2i, 3; 4, 5 - 6i]` concatenation |
| Component extraction & manipulation | implemented | `real(Z)`, `imag(Z)`, `conj(Z)`, `angle(Z)`, `abs(Z)` (via `hypot`), `isreal(Z)` |
| Complex arithmetic | implemented | `+`, `-`, `*`, `.*`, `/`, `./`, `^`, `.^` with mixed real-complex promotion, robust Smith division, and scalar expansion |
| Transpose semantics | implemented | Conjugate transpose `A'` (Hermitian transpose conjugating elements and swapping dimensions) vs non-conjugate transpose `A.'` (swapping dimensions without conjugating) |
| Elementary function domain extensions | implemented | `sqrt(x)` promotes negative numbers to complex ($x < 0 \implies \sqrt{-x}\,i$); `(-1).^0.5` yields `0 + 1i`; `exp(1i*pi)` yields `-1 + 0i`; `log(-1)` yields $0 + \pi i$ |
| Complex relational comparisons | implemented | `==` and `~=` compare both real and imaginary components; `<` `<=` `>` `>=` compare strictly the real components per MATLAB specifications |
| Indexing and growth | implemented | Linear and 2-D indexing on complex matrices; out-of-bounds assignment growth with zero-filled complex numbers; promotion from real to complex on element assignment |
| Desktop & CLI integration | implemented | Complex matrix display formatting (`format_matlab`), desktop workspace viewer grid and inspector support, syntax highlighter for complex builtins |

## Batch 11 additions (2026-10-22)
| Feature | Status | Notes |
|---|---|---|
| Complex storage semantics (`isreal`) | implemented | `isreal(Z)` returns `false` for `Value::ComplexMatrix` regardless of whether imaginary parts are zero (`isreal(complex(12)) == false`); returns `true` for `Value::Matrix` |
| Arithmetic demotion / cancellation | implemented | Arithmetic operations (`+`, `-`, `*`, `.*`, `./`, `.^`, `\`, `/`) cancel to real storage (`Value::Matrix`) when all imaginary parts evaluate to exact zero (`(3+4i) + (5-4i) = 8`, `isreal == true`) |
| Complex SVD | implemented | One-sided Jacobi (Hestenes method) with complex Givens rotations $R^H R = I_2$, non-negative descending singular values, unitary $U, V$ ($U^H U = I, V^H V = I$), $A = U \Sigma V^H$; supports square, rectangular, and rank-deficient matrices |
| Complex rank & pseudoinverse | implemented | `rank(A, [tol])` with singular-value thresholding; `pinv(A, [tol])` satisfying all 4 Moore-Penrose conditions with conjugate transpose $A A^+ A \approx A$, $A^+ A A^+ \approx A^+$, $(A A^+)^H \approx A A^+$, $(A^+ A)^H \approx A^+ A$ |
| Complex linear solvers | implemented | Square systems solved via complex pivoted LU; overdetermined least-squares via complex Householder QR; underdetermined via complex pseudoinverse; right division via $(A^H \setminus B^H)^H$ |
| Eigenvalues: `eig(A)` | implemented | Symmetric real matrices use Jacobi rotations; general real and complex matrices use Hessenberg reduction plus shifted complex QR; complex-conjugate pairs are returned as a complex column vector and real spectra are simplified back to a real matrix; right eigenvectors use inverse iteration; NaN/Inf and non-convergence produce explicit diagnostics; `eig([0 -1; 1 0])` = 0±1i asserted; seeded probe over 20 random 4x4 real matrices: 20/20 converged, sum(λ)=trace and prod(λ)=det held |
| Complex LU factorization | implemented | Partial row pivoting using element modulus; 1-output $Y$, 2-output $[L, U]$, and 3-output $[L, U, P]$ with unit lower triangular $L$ |
| Complex det, inv, trace, norm | implemented | `det(A)` via complex LU with permutation parity; `inv(A)` via complex LU solve on $I_n$; `trace(A)` summing complex diagonal; `norm(A, [p])` for complex vector (1, 2, $\infty$, $p$) and matrix (1, 2, $\infty$, 'fro') |

## Apps framework (2026-10-23)
| Application | Status | Verification | Notes |
|---|---|---|---|
| Data Analysis | Ready | Unit-tested; Not reference-verified; live GUI NOT TESTED | CSV open (header/delimiter detection, quoted fields, missing/ragged rows), preview, column selection, mean/median/std/min/max/quantiles/correlation, line/scatter plot, statistics CSV export with overwrite confirmation |
| Curve Fitter | Ready | Unit-tested; Not reference-verified; live GUI NOT TESTED | CSV open and column selection, polynomial/linear least squares solved through matlab_linalg QR (mldivide), coefficient table, R²/RMSE/residuals, data+fit and residuals figures, curve_fit_* workspace variables, coefficients+fitted CSV export with overwrite confirmation |
| Simulation and Control | Planned | — | Disabled in launcher |
| Signal Analyzer | Planned | — | Disabled in launcher |
| Image Processing | Planned | — | Disabled in launcher |
| Machine Learning | Planned | — | Disabled in launcher |
| Robotics Math | Planned | — | Disabled in launcher |
| Embedded and Sensor Data | Planned | — | Disabled in launcher |

Desktop live GUI verification: NOT TESTED (no screenshot artifact is in the repository; app behaviour is covered by backend/integration tests only).
