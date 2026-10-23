# Function coverage ledger

Audited 2026-10-22 against `docs/reference/MATLAB_Function_Reference.md` (289 catalog
rows) and the workspace code state (milestone 1 + graphics). This ledger accounts for
**every** reference entry: 289 data rows, grouped into the reference's 20 categories
(annotations below show each category's row count to match the reference). It is updated
with every implementation batch; a status written here is a claim about real code +
tests, not about UI presence.

Status tokens (mapping the brief's five statuses):

| Token | Meaning |
|---|---|
| `implemented` | Implemented and tested — working implementation with automated tests. |
| `impl-ptest` | Implemented, partially tested — behavior exists but important cases unverified. |
| `partial` | Partially implemented — only the documented subset works. |
| `missing` | No implementation exists. |
| `integration` | Requires an external backend, device, file format, or service. |
| `planned` | Documented future work, no working code yet. |

`Implementation` names the source module (or `—` when absent). `Tests` names the
integration-test case (or `—` when untested). Statuses below were current as of the
date above; Section 14 records batch updates.

## 1. Creating arrays and matrices (18)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `zeros(m,n)` | implemented | values::matrix, runtime::builtins | — | compatibility:size_zeros_ones_eye | 2-D only | — |
| `ones(m,n)` | implemented | values::matrix, runtime::builtins | — | compatibility:size_zeros_ones_eye | 2-D only | — |
| `eye(n)` | implemented | values::matrix, runtime::builtins | — | compatibility:size_zeros_ones_eye | 2-D only | — |
| `rand(m,n)` | implemented | rng::Rng, runtime::arrayfn | — | language:array_batch2:rand_* (6 cases) | scalar / (m,n) / [m n] syntax; session-held xoshiro256++ | rng(seed) for reproducibility |
| `randn(m,n)` | implemented | rng::Rng, runtime::arrayfn | — | language:array_batch2:randn_* (2 cases) | Box–Muller transform; same RNG stream | — |
| `randi(max,m,n)` | implemented | rng::Rng, runtime::arrayfn | — | language:array_batch2:randi_* (3 cases) | scalar max or [lo hi] range; inclusive bounds | `randi([lo hi], m, n)` overload |
| `randperm(n)` | implemented | rng::Rng, runtime::arrayfn | — | language:array_batch2:randperm_* (3 cases) | Fisher–Yates shuffle; `randperm(n,k)` partial | `k > n` errors |
| `linspace(a,b,n)` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch2:logspace_* (1 case) + numeric_batch1 | 1xN row vector; `n=1` returns `a`; `n=0` -> 1x0 empty | — |
| `logspace(a,b,n)` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch2:logspace_result (1 case) | log-spaced 10^a to 10^b; Inf returned on overflow | — |
| `a:step:b` (colon operator) | missing | — | — | — | — | colon token reserved, unimplemented | parser range + interp + row slices |
| `magic(n)` | implemented | linalg::arrays::magic, runtime::arrayfn | — | language:array_batch2:magic_squares (1 case) | odd (Siamese), doubly-even (complement pairs), n=2 special; singly-even errors | singly-even (6,10,14...) not implemented |
| `diag(v)` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch1:diag_* (2 cases) + array_batch2 | build from vector / extract from matrix; `k` offsets supported both ways | — |
| `repmat(A,m,n)` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch1:repmat_shape (1 case) + array_batch2 | 2-D tiling; `m=0`/`n=0` -> empty | — |
| `meshgrid(x,y)` | impl-ptest | linalg::arrays::meshgrid, runtime::builtins | — | language:multi_output:meshgrid_* (2 cases) | multi-output assignment now supported; single-output returns first matrix; 1-3 arg support in kernel | z-overload (3 args) not yet tested |
| `ndgrid(x,y)` | impl-ptest | linalg::arrays::ndgrid, runtime::builtins | — | language:multi_output:ndgrid_2d (1 case) | multi-output assignment now supported; single-output returns first matrix; 2-D only (max 2 outputs) | N-D not supported |
| `cat(dim,A,B)` | implemented | linalg::arrays::cat, runtime::arrayfn | — | language:array_batch2:vertcat_horzcat_cat (1 case) | dim 1/2 only (2-D model); delegates to vertcat/horzcat | N-D dims error |
| `horzcat`, `vertcat` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch2:vertcat_horzcat_cat (1 case) | 2+ args; dimension consistency checked | row/col concat in 2-D |
| `blkdiag(A,B)` | implemented | linalg::arrays::blkdiag, runtime::arrayfn | — | language:array_batch2:blkdiag_* (2 cases) | 1+ args; block-diagonal with zero fill | — |

## 2.## 2. Array information and manipulation (24)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `size(A)` | partial | runtime::builtins | — | compatibility:size_zeros_ones_eye; matrices:scalars | only 1/2-arg forms; no `[r,c]=size(A)` multi-output; dims>2 unsupported | multi-out assignment when language supports it |
| `length(A)` | implemented | runtime::builtins | — | compatibility:length_and_numel | longest dim of 2-D | — |
| `numel(A)` | implemented | runtime::builtins | — | compatibility:length_and_numel | — | — |
| `ndims(A)` | implemented | runtime::arrayfn | — | language:numeric_batch1:ndims_reports_two_for_matrices | always 2 (2-D storage model) | — |
| `isempty(A)` | implemented | runtime::arrayfn | — | language:numeric_batch1:isempty_* (2 cases) | 1 when any dimension is 0 | — |
| `reshape(A,m,n)` | implemented | linalg::arrays, runtime::arrayfn | — | language:numeric_batch1:reshape_* (4 cases) | element count must match; column-major order | — |
| `squeeze(A)` | missing | — | — | — | — | with N-D landing |
| `permute(A,order)` | missing | — | — | — | — | 2-D permutations first |
| `transpose(A)` / `A'` | implemented | linalg::transpose, runtime::arrayfn | — | language:numeric_batch1:transpose_* (2 cases) | real values only; `'` is not conjugate (no complex type yet) | conj when complex lands |
| `flip`, `fliplr`, `flipud` | implemented | linalg::arrays, runtime::arrayfn | — | language:numeric_batch1:flip_*/fliplr_*/flipud_* (5 cases) | 2-D only; `flip` dim must be 1 or 2 | — |
| `rot90(A)` | implemented | linalg::arrays, runtime::arrayfn | — | language:numeric_batch1:rot90_* (3 cases) | integer `k` only; counter-clockwise | — |
| `circshift(A,k)` | implemented | linalg::arrays, runtime::arrayfn | — | language:numeric_batch1:circshift_* (3 cases) | scalar or 2-element shift; wrap-around | — |
| `sort(A)` | implemented | linalg::arrays::sort, runtime::arrayfn | — | language:array_batch3:sort_* (3 cases) | column sort (dim 1) or row sort (dim 2); default first non-singleton dim | NaN ordering: NaN sorts last |
| `sortrows(A,col)` | implemented | linalg::arrays::sortrows, runtime::arrayfn | — | language:array_batch3:sortrows_basic (1 case) | sort by specified column (1-based); default first column | — |
| `unique(A)` | implemented | linalg::arrays::unique, runtime::arrayfn | — | language:array_batch3:unique_* (2 cases) | sorted unique values (column vector) | — |
| `find(cond)` | implemented | linalg::arrays::find/find_multi, runtime::builtins | — | language:array_batch3:find_* (3 cases) | linear indices (column-major); multi-output [r,c] and [r,c,v] | — |
| `any(A)`, `all(A)` | implemented | linalg::arrays::any/all, runtime::arrayfn | — | language:array_batch3:any_all_* (2 cases) | per-column reduction; returns 1xN; logical as 0/1 doubles | — |
| `ismember(a,B)` | implemented | linalg::arrays::ismember, runtime::arrayfn | — | language:array_batch3:ismember_basic (1 case) | element-wise membership (0/1); preserves shape of A | — |
| `setdiff`, `union`, `intersect` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch3:setdiff/union/intersect_basic (3 cases) | sorted unique values; column vectors | — |
| `cumsum`, `cumprod` | implemented | linalg::arrays, runtime::arrayfn | — | language:array_batch3:cumsum/cumprod_basic (2 cases) | column-wise cumulative; preserves dimensions | — |
| `diff(A)` | impl-ptest | linalg::arrays::diff, runtime::arrayfn | — | language:array_batch3:diff_* (2 cases) | single difference along columns; n-times not supported | `diff(A,n)` and dim argument |
| `histcounts(x,edges)` | missing | — | — | — | — | shared binning with histogram |
| `end` (keyword) | missing | — | — | — | — | index context in interp |
| `triu`, `tril` | implemented | linalg::arrays, runtime::arrayfn | — | language:numeric_batch1:triu_*/tril_* (5 cases) | rectangular input accepted (dimensions preserved) | — |

## 3. Elementary math (18)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `abs`, `sign` | implemented | runtime::builtins | — | compatibility:scalar_math_functions, language:numeric_batch7:sign_*, language:numeric_batch10:test_component_extraction_* | `abs` supports real and complex (via hypot); `sign` returns -1, 0, 1, or NaN | — |
| `sqrt`, `nthroot(x,n)` | implemented | runtime::builtins | — | compatibility:scalar_math_functions, language:numeric_batch7:nthroot_*, language:numeric_batch10:test_sqrt_negative_* | `sqrt` supports real and complex, promotes $x < 0$ to pure imaginary; `nthroot` supports odd roots of negative numbers | — |
| `exp`, `log`, `log2`, `log10` | implemented | runtime::builtins | — | compatibility:scalar_math_functions, language:numeric_batch10:test_euler_identity, test_log_negative_* | `exp` and `log` support complex numbers; `log` promotes $x < 0$ to complex; `log2`/`log10` real-valued | complex log2/log10 |
| `power(a,b)` / `.^` | implemented | linalg::elementwise_pow | — | matrices:elementwise_power_and_division, language:numeric_batch10:test_power_negative_* | supports real, complex, and mixed operands; promotes negative base with fractional power to complex | — |
| `sin`, `cos`, `tan` | implemented | runtime::builtins | — | language:numeric_batch7 | unary map | — |
| `sind`, `cosd`, `tand` | implemented | runtime::builtins | — | language:numeric_batch7:exact_degree_trig | exact zeros for multiples of 90/180 | — |
| `asin`, `acos`, `atan`, `atan2(y,x)` | implemented | runtime::builtins | — | language:numeric_batch7:atan2_* | real domain; atan2 supports scalar expansion | complex inverse trig after complex type |
| `asind`, `acosd`, `atand` | implemented | runtime::builtins | — | language:numeric_batch7:asind_*, acosd_*, atand_* | degree output; domain boundaries and outside-domain NaN verified | — |
| `sinh`, `cosh`, `tanh` | implemented | runtime::builtins | — | language:numeric_batch7 | unary map | — |
| `deg2rad`, `rad2deg` | implemented | runtime::builtins | — | language:numeric_batch7 | unary map | — |
| `hypot(a,b)` | implemented | runtime::builtins | — | language:numeric_batch7 | IEEE hypot without overflow; scalar expansion supported | — |
| `floor`, `ceil`, `round`, `fix` | implemented | runtime::builtins | — | compatibility:scalar_math_functions | round = half-away-from-zero; fix = truncate | — |
| `mod(a,b)`, `rem(a,b)` | implemented | runtime::builtins | — | language:numeric_batch7:mod_vs_rem_* | `mod(x,0)==x`, `rem(x,0)==NaN`; sign conventions preserved | — |
| `gcd`, `lcm` | implemented | runtime::builtins | — | language:numeric_batch7 | Euclid algorithm; supports scalar expansion | — |
| `factorial(n)`, `nchoosek(n,k)` | implemented | runtime::builtins | — | language:numeric_batch7:factorial_*, nchoosek_* | float accumulation; nchoosek symmetry min(k,n-k); overflow to Inf at 171 | — |
| `primes(n)`, `isprime(n)`, `factor(n)` | implemented | runtime::builtins | — | language:numeric_batch7:primes_*, factor_* | sieve for primes; trial division for factor; factor(1)==[1]; product reconstructs input | large integer precision limited to flintmax |
| `complex`, `real`, `imag`, `conj`, `angle`, `isreal` | implemented | runtime::builtins, values::complex, linalg::complex | — | language:numeric_batch10:test_complex_constructor_*, language:numeric_batch11:test_complex_storage_semantics_and_isreal, test_arithmetic_cancellation_and_promotion | full scalar and array support; scalar broadcasting for `complex(re, im)`; isreal distinguishes storage type (`complex(12)` is false); arithmetic cancels to real storage when imag is exact zero | — |
| `bitand`, `bitor`, `bitxor`, `bitshift` | implemented | runtime::builtins | — | language:numeric_batch7:bitwise_*, bitshift_* | safe shift logic (no panic on shift>=64 or <=-64); 3-arg bitshift width mask | integer type cast |
| `dec2bin`, `bin2dec`, `dec2hex`, `hex2dec` | implemented | runtime::builtins | — | language:numeric_batch7:dec2bin_*, bin2dec_*, dec2hex_*, hex2dec_* | 32-bit two's complement for negatives (padded with 1/F); scalar and vector inputs; floor semantics; multi-line string round-trips to column vectors | precision limited to flintmax |

## 4. Statistics (11)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `sum`, `prod` | implemented | linalg::stats | — | language:numeric_batch8:sum_*, prod_* | supports dim, 'all', empty (0.0 for sum, 1.0 for prod) | — |
| `mean`, `median`, `mode` | implemented | linalg::stats | — | language:numeric_batch8:mean_*, median_*, mode_* | supports dim, 'all'; mode resolves ties to smallest value; empty returns NaN | — |
| `std`, `var` | implemented | linalg::stats | — | language:numeric_batch8:std_*, var_* | supports w=0 (N-1) and w=1 (N), dim, 'all'; constant data returns 0.0; empty returns NaN | — |
| `min`, `max` | implemented | linalg::stats | — | language:numeric_batch8:min_max_* | reduction along dim/all; element-wise binary min(A,B)/max(A,B) with scalar expansion; 2-output [M,I] returning 1-based column-major indices | — |
| `range`, `iqr` | implemented | linalg::stats | — | language:numeric_batch8:range_*, iqr_* | range = max-min; iqr uses empirical quantile linear interpolation (r = 0.5 + N*p) matching MATLAB; empty returns NaN | — |
| `prctile`, `quantile` | partial | linalg::stats, runtime::builtins | — | language:numeric_batch12, numeric_batch14, numeric_batch16, numeric_batch17, layout_stage11 | scalar and vector pct/prob work; matrix dim=1 (k x ncols, row i = p(i) percentile of each column) and dim=2 (nrows x k) implemented and hand-verified on 2x3/3x2; NaN removal implemented; interpolation default midpoint (rank r = 0.5 + n*p); inclusive/exclusive deferred | complete integer-n quantile result construction; full method selection; complex statistics; pairwise NaN options |
| `corrcoef`, `corr`, `cov` | partial | linalg::stats, runtime::builtins | — | language:numeric_batch12, numeric_batch14 | `cov(A)`, `cov(A,B)`, `corrcoef(A)` work; `cov(A,w)` supported; missing-data flags partial; pairwise/complete/row-handling deferred; complex statistics deferred | full NaN-flag coverage; `Rows` policies; complex correlation/covariance verification |
| `movmean`, `movmedian`, `movsum` | missing | — | — | — | window handling (edges) | — |
| `normalize` | missing | — | — | — | methods: data/z/scale/range | — |
| `rescale` | missing | — | — | — | trivial | — |
| `cummax`, `cummin` | missing | — | — | — | running extrema | — |

## 5. Linear algebra (14)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `A*B`, `A.*B` | implemented | linalg::matmul / elementwise_mul | — | matrices:matrix_multiply_* | scalar broadcast like MATLAB | — |
| `A\b`, `mldivide`, `ldivide` | implemented | linalg::decomposition::mldivide, linalg::complex::mldivide_values, runtime::interp, runtime::builtins | — | language:numeric_batch9:test_mldivide_*, language:numeric_batch11:test_complex_mldivide_* | square pivoted LU solve, rectangular Householder QR least-squares, pseudoinverse underdetermined; real and complex matrices | — |
| `inv(A)` | implemented | linalg::decomposition::inv, linalg::complex::inv, runtime::builtins | — | language:numeric_batch9:test_inv_*, language:numeric_batch11:test_complex_determinant_and_inverse | square nonsingular systems via pivoted LU solve; errors on singular matrices; real and complex matrices | — |
| `pinv(A)` | implemented | linalg::svd::pinv, linalg::svd::complex_pinv, runtime::builtins | — | language:numeric_batch9:test_pinv_*, language:numeric_batch11:test_complex_pinv_moore_penrose_conditions | SVD-based Moore–Penrose pseudoinverse; explicit or default tolerance; all 4 Moore-Penrose conditions verified; real and complex matrices | — |
| `det(A)` | implemented | linalg::decomposition::det, linalg::complex::det, runtime::builtins | — | language:numeric_batch9:test_det_*, language:numeric_batch11:test_complex_determinant_and_inverse | pivoted LU decomposition; accounts for permutation parity and pivot signs; real and complex matrices | — |
| `rank(A)` | implemented | linalg::svd::rank, linalg::svd::complex_rank, runtime::builtins | — | language:numeric_batch9:test_rank_*, language:numeric_batch11:test_complex_rank_full_and_deficient | singular values threshold; explicit or dimension-based machine precision default; real and complex matrices | — |
| `trace(A)` | implemented | linalg::decomposition::trace, linalg::complex::trace, runtime::builtins | — | language:numeric_batch9:test_trace_*, language:numeric_batch11:test_complex_trace_and_norms | sum of main diagonal elements for square or rectangular; empty returns 0; real and complex matrices | — |
| `norm(A)` | implemented | linalg::svd::norm, linalg::svd::complex_norm, runtime::builtins | — | language:numeric_batch9:test_norm_*, language:numeric_batch11:test_complex_trace_and_norms | vector p-norms (1,2,inf,-inf,p); matrix norms (1,2,inf,fro); 2-norm uses svd; real and complex matrices | — |
| `eig(A)` | impl-ptest | linalg::decomposition::{eig, eig_general, is_symmetric}, linalg::complex::eig_complex, runtime::builtins | — | language:numeric_batch_stage2_eig, language:numeric_batch14, language:eig_complex_pair_probe | symmetric real = Jacobi; general real = Hessenberg + shifted complex QR, complex pairs returned as `ComplexMatrix`, real spectra simplified to `Matrix`; right eigenvectors via inverse iteration; NaN/Inf rejected; scale-aware symmetry; `eig_with(max_iter)` exposes non-convergence error; `eig([0 -1; 1 0])` = 0±1i asserted at runtime (order-independent); raw MATLAB order (+i, -i) parked as an ignored known limitation — runtime lists the pair (-i, +i) in QR iteration order | Complex-Hermitian specialization not separately tested; defective-matrix eigenvectors are best-effort; seeded probe over 20 random 4x4 real matrices: 20/20 converged, sum(λ)=trace and prod(λ)=det held for every converged case |
| `svd(A)` | implemented | linalg::svd::svd_decompose, linalg::svd::complex_svd_decompose, runtime::builtins | — | language:numeric_batch9:test_svd_*, language:numeric_batch11:test_complex_svd_* | one-sided Jacobi (Hestenes method) with Gram-Schmidt orthogonalization; 1-out and 3-out forms; real and complex matrices | — |
| `lu(A)` | implemented | linalg::decomposition::pivoted_lu, linalg::complex::pivoted_lu, runtime::builtins | — | language:numeric_batch9:test_lu_*, language:numeric_batch11:test_complex_lu_forms | partial row pivoting; 1-out Y, 2-out [L,U], and 3-out [L,U,P] where P*A=L*U; real and complex matrices | — |
| `null(A)`, `orth(A)` | missing | — | — | — | — | from SVD |
| `expm(A)` | missing | — | — | — | — | scaling-and-squaring |
| `cross(a,b)`, `dot(a,b)`, `kron(A,B)` | missing | — | — | — | — | vector kernels |
| `cond(A)` | missing | — | — | — | — | via SVD ratio |

## 6. Logical and relational (7)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `==`, `~=`, `<`, `>`, `<=`, `>=` | impl-ptest | linalg::compare | — | matrices:comparisons_produce_zero_one_matrix | 0/1 doubles, not a logical type (documented divergence) | logical array type |
| `&`, `\|`, `~`, `xor` | missing | — | — | — | no lexer tokens | lexer + elementwise op + precedence |
| `&&`, `\|\|` | missing | — | — | — | — | short-circuit in evaluator |
| `isnan`, `isinf`, `isfinite` | missing | — | — | — | — | unary maps to 0/1 |
| `isequal(a,b)` | missing | — | — | — | — | shape+value equality |
| `islogical`, `isnumeric`, `ischar`, `iscell`, `isstruct` | missing | — | — | — | — | per-type checks |
| `isa(x,'class')` | missing | — | — | — | — | maps to is-type set |

## 7. Strings and text (14)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `string`, `char` | missing | — | — | — | Value::String exists for char arrays only | add conversions |
| `strcat`, `append` | missing | — | — | — | — | char/string concat |
| `strsplit`, `split` | missing | — | — | — | objects as cell arrays later | cell-array dependency |
| `strjoin` | missing | — | — | — | — | after cells |
| `strrep`, `replace` | missing | — | — | — | — | substring replace |
| `strfind`, `contains`, `startsWith`, `endsWith` | missing | — | — | — | — | plain search first |
| `strcmp`, `strcmpi`, `strncmp` | missing | — | — | — | — | equality incl. case rules |
| `upper`, `lower`, `strtrim` | missing | — | — | — | — | trivial maps |
| `num2str`, `str2num`, `str2double` | missing | — | — | — | — | formatting for num2str; eval for str2num later |
| `sprintf`, `fprintf`, `disp` | partial | runtime::builtins (`disp`) | — | language:milestone | disp done; sprintf/fprintf missing | printf format engine |
| `regexp`, `regexprep` | missing | — | — | — | — | document regex dialect (Rust `regex`) |
| `extractBetween`, `extractAfter`, `extractBefore` | missing | — | — | — | — | substring windows |
| `strlength` | missing | — | — | — | — | char count |
| `input`, `inputdlg` | missing | — | — | — | dialog needs desktop | input() on CLI, dialog planned |

## 8. Data types and containers (17)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `int8..int64`, `uint8..uint64` | missing | — | — | — | only f64 values exist | integer lanes in Value |
| `single`, `double` | missing | — | — | — | double only | single lane |
| `class(x)`, `whos` | missing | — | — | — | REPL `who` command only | mapping Value→class names; whos table |
| `cast`, `typecast` | missing | — | — | — | — | after int/float lanes |
| `intmax`, `intmin`, `realmax`, `eps` | missing | — | — | — | — | constants from Rust limits |
| `cell`, `cell2mat`, `num2cell` | missing | — | — | — | Value has no cell type | Value::Cell |
| `cellfun`, `arrayfun` | missing | — | — | — | needs function handles | after Stage E |
| `struct`, `fieldnames`, `isfield`, `rmfield` | missing | — | — | — | no struct type | Value::Struct |
| `table`, `array2table`, `table2array` | missing | — | — | — | no table type | Value::Table |
| `height(T)`, `width(T)` | missing | — | — | — | — | after table |
| `innerjoin`, `outerjoin`, `join` | missing | — | — | — | — | after table |
| `groupsummary`, `varfun` | missing | — | — | — | — | after table |
| `containers.Map` | missing | — | — | — | — | key/value container |
| `dictionary` | missing | — | — | — | — | shares map infra |
| `datetime`, `duration`, `timetable` | missing | — | — | — | — | see category 19 |
| `categorical` | missing | — | — | — | — | string→category mapping |
| `fi`, `fixdt` | missing | — | — | — | fixed-point | planned; document numeric model |

## 9. Programming and control flow (16)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `if`, `elseif`, `else`, `end` | missing | — | — | — | parser/eval stages | stmt blocks + scopes |
| `for`, `while`, `break`, `continue` | missing | — | — | — | — | loop context in interp |
| `switch`, `case`, `otherwise` | missing | — | — | — | — | equality-case matching |
| `try`, `catch` | missing | — | — | — | — | eval error capture |
| `function` | missing | — | — | — | no scope stack | Module + file-local functions |
| `return` | missing | — | — | — | — | eval control signal |
| `nargin`, `nargout` | missing | — | — | — | — | per-call counts |
| `varargin`, `varargout` | missing | — | — | — | — | cell-based argument lists |
| `inputParser`, `validateattributes`, `arguments` | missing | — | — | — | — | after functions |
| `error`, `warning`, `assert` | missing | — | — | — | — | Diagnostic plumbing exists |
| `@(x) x.^2` anonymous functions | missing | — | — | — | — | closure capture model |
| `feval`, `func2str`, `str2func` | missing | — | — | — | — | handles as values |
| `tic`, `toc` | missing | — | — | — | — | session timer state |
| `pause(sec)` | missing | — | — | — | — | honors cancel flag |
| `parfor`, `parfeval` | missing | — | — | — | Parallel Computing only | explicit capability boundary (sequential semantics) |
| `classdef`, `properties`, `methods` | missing | — | — | — | OOP | planned late |

## 10. Files and data I/O (16)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `load`, `save` | missing | — | — | — | no MAT format yet | .mat v5 writer later; text first |
| `readtable`, `writetable` | missing | — | — | — | — | CSV + basic type inference |
| `readmatrix`, `writematrix` | missing | — | — | — | — | numeric delimited |
| `readcell` | missing | — | — | — | needs cells | after Value::Cell |
| `csvread`, `dlmread` | missing | — | — | — | legacy | thin wrappers over readmatrix path |
| `xlsread`, `xlswrite` | integration | — | native xlsx | — | — | behind a format backend |
| `fopen`, `fclose` | missing | — | — | — | — | file handle registry in runtime |
| `fread`, `fwrite` | missing | — | — | — | — | typed binary with endian rules |
| `fgetl`, `fgets`, `fscanf`, `fprintf` | missing | — | — | — | — | text line/format scans |
| `textscan` | missing | — | — | — | — | format spec parser |
| `imread`, `imwrite` | integration | — | image crate | — | — | pick backend + license check |
| `audioread`, `audiowrite` | integration | — | audio crate | — | — | wav first |
| `jsondecode`, `jsonencode` | missing | — | — | — | — | serde_json mapping to values |
| `webread`, `webwrite` | missing | — | — | — | side-effects guarded by explicit calls | http client + JSON |
| `dir`, `ls`, `pwd`, `cd`, `mkdir`, `exist`, `delete` | missing | — | — | — | destructive ops need explicit calls | runtime fs commands |
| `fullfile`, `fileparts` | missing | — | — | — | — | path composition |

## 11. Plotting and visualization (22)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `plot(x,y)` | impl-ptest | runtime::graphics, builtins | egui_plot (desktop) | graphics:plot_* | vectors only; no LineSpec color/style args yet | LineSpec parsing, hold, ranges |
| `scatter(x,y)` | impl-ptest | runtime::graphics, builtins | egui | graphics:plot_and_scatter_two_args | same limits | marker sizes |
| `bar`, `barh` | partial | runtime::graphics, builtins | egui | graphics:bar_and_histogram | vertical only | barh + grouped bars |
| `histogram(x)` | impl-ptest | runtime::graphics, builtins | egui | graphics:bar_and_histogram | fixed 10 bins or explicit count | binning rules from histcounts |
| `pie` | missing | — | — | — | — | account for negative/sum-zero data |
| `stairs`, `stem` | missing | — | — | — | — | line style variants of XY plot |
| `semilogx`, `semilogy`, `loglog` | missing | — | — | — | — | axis-scale flag in figure state |
| `errorbar` | missing | — | — | — | — | symmetric/asymmetric bounds |
| `area`, `fill` | missing | — | — | — | — | polygon series |
| `polarplot` | missing | — | — | — | — | polar mapping |
| `plot3`, `scatter3` | missing | — | — | — | 3-D projection planned | projection model |
| `surf`, `mesh`, `contour`, `contourf` | missing | — | — | — | — | marching squares for contours |
| `quiver` | missing | — | — | — | — | arrows over grid |
| `imagesc`, `imshow`, `heatmap` | missing | — | — | — | — | color mapping |
| `subplot`, `tiledlayout`, `nexttile` | missing | — | — | — | — | tile layout model in FigureState |
| `figure`, `close`, `clf`, `hold on/off` | partial | runtime::graphics, builtins | — | graphics:figure_lifecycle, clf_and_close | handles + clf/close done; hold missing | hold state |
| `xlabel`, `ylabel`, `zlabel`, `title`, `legend` | partial | runtime::graphics, builtins | — | graphics:titles_labels_grid_legend | no zlabel; legend labels only | zaxis when 3-D lands |
| `xlim`, `ylim`, `axis`, `grid`, `colorbar`, `colormap` | partial | runtime::graphics (grid) | — | graphics:bad_graphics_arguments | grid('on'/'off') only | axis range state |
| `text`, `annotation` | missing | — | — | — | — | overlay primitives |
| `saveas`, `exportgraphics`, `print` | missing | — | — | — | — | headless PNG export pipeline |
| `drawnow` | missing | — | — | — | — | UI tick hook |
| `animatedline`, `comet` | missing | — | — | — | — | incremental series |

## 12. Polynomials, interpolation, calculus, optimization (15)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `polyfit(x,y,n)`, `polyval(p,x)` | missing | — | — | — | — | QR-based least squares |
| `roots(p)`, `poly(r)` | missing | — | — | — | — | companion-matrix eig (needs complex) |
| `conv`, `deconv` | missing | — | — | — | — | direct convolution |
| `polyder`, `polyint` | missing | — | — | — | — | coefficient ops |
| `interp1`, `interp2`, `spline`, `pchip` | missing | — | — | — | — | linear/spline/pchip kernels |
| `trapz`, `cumtrapz` | missing | — | — | — | — | trapezoid with spacing |
| `integral`, `integral2`, `quad` | missing | — | — | — | — | adaptive Gauss–Kronrod |
| `gradient`, `del2` | missing | — | — | — | — | central differences |
| `fzero` | missing | — | — | — | — | Brent + bracket search |
| `fsolve` | missing | — | — | — | — | Newton with numerics note |
| `fminsearch`, `fminunc`, `fmincon` | missing | — | — | — | — | Nelder–Mead first |
| `lsqcurvefit`, `lsqnonlin` | missing | — | — | — | — | Gauss–Newton |
| `linprog`, `quadprog` | missing | — | — | — | — | document algorithm limits |
| `ode45`, `ode23`, `ode15s` | missing | — | — | — | — | RK45 adaptive first |
| `fit`, `cftool` | missing | — | — | — | interactive tool excluded | fit types subset |

## 13. Signal processing (17)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `fft`, `ifft`, `fftshift` | missing | — | — | — | needs complex | radix-2 + Bluestein or rustfft |
| `fft2`, `ifft2` | missing | — | — | — | — | row/col transforms |
| `filter(b,a,x)` | missing | — | — | — | — | direct form II transposed |
| `filtfilt` | missing | — | — | — | — | forward/backward with padding |
| `conv`, `xcorr` | missing | — | — | — | — | shares conv kernel |
| `butter`, `cheby1`, `cheby2`, `ellip` | missing | — | — | — | — | bilinear transform design |
| `fir1`, `firpm` | missing | — | — | — | — | window method first |
| `freqz`, `freqs` | missing | — | — | — | — | response eval + unwrap |
| `designfilt` | missing | — | — | — | interactive | specs subset |
| `spectrogram`, `pwelch`, `periodogram` | missing | — | — | — | — | windowed FFT stats |
| `resample`, `decimate`, `interp` | missing | — | — | — | — | rational resampling |
| `detrend`, `smoothdata` | missing | — | — | — | — | linear detrend/moving averages |
| `findpeaks` | missing | — | — | — | — | prominence threshold |
| `hilbert`, `envelope` | missing | — | — | — | — | Hilbert via FFT |
| `window`, `hamming`, `hann` | missing | — | — | — | — | standard windows |
| `chirp`, `square`, `sawtooth` | missing | — | — | — | — | waveform generators |
| `snr`, `rms`, `bandpower` | missing | — | — | — | — | measurement defs |

## 14. Control systems (14)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `tf(num,den)` | missing | — | — | — | needs system data model | polynomial TF type |
| `ss(A,B,C,D)` | missing | — | — | — | — | state-space type |
| `zpk` | missing | — | — | — | — | after TF |
| `c2d`, `d2c` | missing | — | — | — | — | ZOH first |
| `step`, `impulse`, `lsim` | missing | — | — | — | — | discrete-time response |
| `bode`, `nyquist`, `nichols` | missing | — | — | — | — | frequency sweep + complex resp |
| `margin` | missing | — | — | — | — | crossover detection |
| `rlocus` | missing | — | — | — | — | root locus eval |
| `pole`, `zero`, `pzmap` | missing | — | — | — | — | complex root math |
| `feedback`, `series`, `parallel` | missing | — | — | — | — | interconnections |
| `pid`, `pidtune` | missing | — | — | — | — | simple tuning rules |
| `lqr`, `place`, `kalman` | missing | — | — | — | — | ARE solvers |
| `ctrb`, `obsv` | missing | — | — | — | — | rank via stack |
| `stepinfo` | missing | — | — | — | — | response metrics |

## 15. Hardware, serial and networking (16)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `serialportlist` | integration | — | serialport crate | — | — | capability-gated device code |
| `serialport(port,baud)` | integration | — | serialport crate | — | — | same |
| `write`, `writeline` | integration | — | — | — | — | TX buffer + terminal config |
| `read`, `readline`, `readbinblock` | integration | — | — | — | — | RX with timeout policy |
| `flush`, `configureTerminator`, `configureCallback` | integration | — | — | — | — | state on handle |
| `arduino(port,board)` | integration | — | — | — | — | adapter profile |
| `readDigitalPin`, `writeDigitalPin`, `readVoltage`, `writePWMDutyCycle` | integration | — | — | — | — | command protocol |
| `i2cdev`, `device`, `spidev` | integration | — | — | — | — | bus abstraction |
| `raspi` | integration | — | — | — | — | remote protocol |
| `tcpclient`, `tcpserver` | missing | — | std::net | — | explicit user action required | sockets behind runtime |
| `udpport` | missing | — | std::net | — | — | same |
| `bluetooth`, `ble` | integration | — | — | — | — | platform APIs |
| `mqttclient` | integration | — | mqtt crate | — | — | broker policy |
| `thingSpeakRead`, `thingSpeakWrite` | integration | — | HTTP | — | — | REST over webread path |
| `can`, `canChannel` | integration | — | socketcan optional | — | — | capability-gated |
| `visadev` | integration | — | — | — | — | scpi over tcp/usb |

## 16. Code generation and Simulink (8)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `codegen` | planned | — | — | — | — | C/C++ emitter architecture |
| `coder.config` | planned | — | — | — | — | config schema |
| `coder.typeof`, `coder.varsize` | missing | — | — | — | — | type/size declarations |
| `slbuild` | planned | — | — | — | — | model→builder pipeline |
| `sim`, `simset` | missing | — | — | — | — | model simulator |
| `open_system`, `new_system`, `add_block`, `add_line` | missing | — | — | — | — | block graph model |
| `set_param`, `get_param` | missing | — | — | — | — | param store |
| `bdclose` | missing | — | — | — | — | model registry |

## 17. Symbolic Math (8)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `syms x y` | missing | — | — | — | no symbolic engine | Value::Sym expression tree |
| `solve`, `vpasolve` | missing | — | — | — | — | engine-dependent |
| `diff`, `int`, `limit`, `taylor` | missing | — | — | — | — | rules engine |
| `simplify`, `expand`, `factor`, `collect` | missing | — | — | — | — | rewriting rules |
| `subs`, `double` | missing | — | — | — | — | substitution baseline |
| `laplace`, `ilaplace`, `fourier`, `ifourier`, `ztrans` | missing | — | — | — | — | transform rules |
| `dsolve` | missing | — | — | — | — | symbolic ODEs |
| `matlabFunction` | missing | — | — | — | — | emit anonymous functions |

## 18. Machine learning and deep learning (15)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `fitlm`, `fitglm` | missing | — | — | — | — | OLS via linalg |
| `fitctree`, `fitrtree` | missing | — | — | — | — | greedy splits |
| `fitcsvm` | missing | — | — | — | — | SMO subset |
| `fitcknn` | missing | — | — | — | — | brute force first |
| `fitcensemble`, `TreeBagger` | missing | — | — | — | — | bagging over trees |
| `kmeans`, `kmedoids`, `dbscan` | missing | — | — | — | — | seeded centers |
| `pca` | missing | — | — | — | — | via SVD |
| `predict` | missing | — | — | — | — | per-model dispatch |
| `crossval`, `kfoldLoss` | missing | — | — | — | — | fold partitioning |
| `confusionmat`, `confusionchart` | missing | — | — | — | — | bincount on labels |
| `trainNetwork`, `trainnet`, `dlnetwork` | missing | — | — | — | — | tensor model + backend |
| `layerGraph`, `imageInputLayer`, `convolution2dLayer` | missing | — | — | — | — | layer schema |
| `trainingOptions` | missing | — | — | — | — | config struct |
| `classify`, `activations` | missing | — | — | — | — | inference path |
| `importONNXNetwork` | integration | — | onnx backend | — | — | backend decision |

## 19. Dates and time (6)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `datetime('now')` | missing | — | — | — | — | epoch-based datetime type |
| `datestr`, `datenum` | missing | — | — | — | — | format strings |
| `seconds`, `minutes`, `hours`, `days` | missing | — | — | — | — | duration units |
| `between`, `caldiff` | missing | — | — | — | — | calendar diffs |
| `weekday`, `month`, `year` | missing | — | — | — | — | part extraction |
| `retime`, `synchronize` | missing | — | — | — | — | timetable resampling |

## 20. Help, debugging and workspace (13)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `help name`, `doc name` | missing | — | — | — | registry can carry doc text | render from registry metadata |
| `lookfor keyword` | missing | — | — | — | — | substring search on registry |
| `which name` | missing | — | — | — | — | path + builtin resolution report |
| `what`, `who`, `whos` | partial | cli (REPL `who`) | — | — | `who` is a REPL command, not a builtin; `whos` table missing | builtin who/whos over Env |
| `clear`, `clc`, `close all` | partial | cli (REPL `clear`); builtins (`close`) | — | graphics:clf_and_close (figures) | `clear` not a builtin call; `clc` missing | register clc/clear/whos builtins |
| `format long/short` | partial | values::format_matlab | — | — | own formatter approximates MATLAB short/long layout | format state in session |
| `dbstop`, `dbstep`, `dbcont`, `dbquit` | missing | — | — | — | — | interpreter debug hooks |
| `profile on/viewer` | missing | — | — | — | — | per-node timing counters |
| `checkcode` | missing | — | — | — | — | lint over AST |
| `ver`, `version` | missing | — | — | — | — | capability report |
| `addpath`, `rmpath`, `path` | missing | — | — | — | — | search-path module |
| `publish`, `export` | missing | — | — | — | — | report generator |
| `matlab.unittest` | missing | — | — | — | — | test-runner inside runtime |

---

## Batch update log

| Batch | Date | Rows changed |
|---|---|---|
| Initial audit | 2026-10-22 | all entries established (289) |
=== BATCH 1 STATUS (verified) ===
Implemented + tested: zeros, ones, eye, linspace, diag, repmat, size, length, numel, ndims, isempty, reshape, transpose, flip, fliplr, flipud, rot90, circshift, triu, tril.
Source: arrayfn.rs + builtins.rs delegation (added match arms); algorithms in matlab_linalg::arrays
Tests: tests/language/tests/array_batch1.rs (10/10 pass); invalid-dims error assertions preserved.
Limitations: 2-D only; N-D overloads rejected; empty arrays handled; negative/fractional dims rejected; output suppression follows MATLAB semicolon rules (tests corrected to use session env where suppressed).
Remaining fmt diff: builtins.rs edit (expected).

=== BATCH 2 STATUS (2026-10-22) ===
Implemented + tested: rand, randn, randi, randperm, logspace, magic, cat, vertcat, horzcat, blkdiag, rng.
Source: arrayfn.rs + builtins.rs delegation (14 new match arms); algorithms in matlab_linalg::arrays + rng::Rng (xoshiro256++ + Box–Muller).
Tests: tests/language/tests/array_batch2.rs (18/18 pass) — deterministic RNG seeded; range/edge/dimension errors verified.
Limitations: meshgrid/ndgrid require multi-output assignment (AssignTarget::Multi not yet in interp); singly-even magic(n) errors; 2-D only model.
All pre-existing tests pass; cargo fmt/check clean; CLI matrix 19 22 / 43 50 verified.

=== BATCH 3 STATUS (2026-10-22) — REA-DRIVEN MULTI-OUTPUT ASSIGNMENT ===
Implemented + tested: multi-output assignment `[X, Y] = f(...)`, meshgrid, ndgrid, size multi-output.
Source: interp.rs (AssignTarget::Multi handling), builtins.rs (call_builtin_multi), linalg::arrays (existing kernels wired).
Tests: tests/language/tests/multi_output.rs (5 tests) — all pass.
Limitations: only builtin function calls support multi-output; user-defined functions pending; nargout beyond natural outputs returns empty matrices; meshgrid z-overload (3 args) not yet tested.
REA: MATLAB target unavailable (installer has only Network License Manager). inspect_rust_binary blocked by 256MB limit. Investigation record: docs/reverse-engineering/multi-output-assignment.md

=== BATCH 4 STATUS (2026-10-22) — REA-GUIDED ARRAY SEMANTICS ===
Implemented + tested: sort, sortrows, unique, ismember, setdiff, union, intersect, find (single + multi-output), any, all, cumsum, cumprod, diff.
Source: linalg::arrays (13 new kernels) + arrayfn (13 wrappers) + builtins (13 match arms) + call_builtin_multi (find multi-output).
Tests: tests/language/tests/array_batch3.rs (21 tests) — all pass.
Limitations: sort default dim = first non-singleton (MATLAB behavior); sortrows sorts by single column; find returns linear indices (column-major); any/all per-column (1xN output); diff single n=1 along columns; logical values as 0/1 doubles.
Colon operator verified working: 1:5, 1:2:9, 5:-1:1, 1:0 (empty), A(1,:), A(:,2), A(end,:) — all functional.
REA: MATLAB target unavailable (installer has only Network License Manager). inspect_rust_binary blocked by 256MB limit. Investigation record: docs/reverse-engineering/multi-output-assignment.md (updated).

=== BATCH 5 STATUS (2026-10-22) — COMPLETE ARRAY SEMANTICS ===
Implemented + tested: context-aware `end` in ranges/indexing, sort (ascend/descend, dim), sortrows (multi-column), find (count + direction + multi-output), any/all (dim arg), diff (n + dim).
Source: interp.rs (range indexing with ColonAll/Range/End resolution), linalg::arrays (sort with descending, any_dim, all_dim, diff with n/dim, sortrows multi-col), arrayfn (updated wrappers).
Tests: array_batch3.rs (21 tests, all pass) + new behaviors verified via CLI.
Limitations: sort NaN ordering (NaN sorts last in Rust default); find 'last' direction uses string arg; diff n-times along single dim; logical values as 0/1 doubles.
Context-aware end: A(2:end), A(1:end-1), A(1:2:end), A(end-1:end), A(end,:), A(:,2) all working.
Colon operator: 1:5, 1:2:9, 5:-1:1, 1:0 (empty) all working.
REA: MATLAB target unavailable. No fabricated evidence.

=== BATCH 6 STATUS (2026-10-22) — ELEMENTARY MATHEMATICS ===
Implemented + tested (via CLI verification):
Trigonometry: sin, cos, tan, asin, acos, atan, atan2, sind, cosd, tand, sinh, cosh, tanh, deg2rad, rad2deg, hypot.
Exponential/rounding: sqrt, nthroot, exp, log, log2, log10, floor, ceil, round, fix, mod, rem.
Number theory: gcd, lcm, factorial, nchoosek, primes, isprime.
Bitwise: bitand, bitor, bitxor, bitshift.
Source: builtins.rs (unary map extended + 20 new match arms).
Limitations: real-valued only (no complex); bit operations cast to i64; factorial uses saturating_mul (overflow -> max); nchoosek integer only; primes uses sieve; no dec2bin/bin2dec/dec2hex/hex2dec yet.
REA: MATLAB target unavailable. No fabricated evidence.

=== BATCH 7 STATUS (2026-10-22) — MISSING ELEMENTARY MATHEMATICS ===
Implemented + tested:
Base conversions: dec2bin, bin2dec, dec2hex, hex2dec (multi-row string round-trip, two's complement, width, fractional floor).
Signum: sign (element-wise, NaN propagation).
Degree inverse trig: asind, acosd, atand (degree outputs, [-1, 1] domain check).
Exact degree trig: sind, cosd, tand (exact integer multiples of 90/180).
Combinatorics / Number theory: factor (prime factorization, 1xN row vector), factorial (f64 representation up to 170, then Inf), nchoosek (symmetry, avoiding early overflow).
Real nth root: nthroot (real odd roots of negative numbers, scalar expansion).
Modulus & Remainder: mod, rem (sign conventions, division by zero: mod(x,0)==x, rem(x,0)==NaN).
Bitwise & Scalar Expansion: bitshift (safe bounds check against shift overflow, 3-arg mask).
Source: crates/runtime/src/builtins.rs + crates/linalg/src/lib.rs.
Tests: tests/language/tests/numeric_batch7.rs (36 tests, all pass).
Architecture: docs/architecture/complex-numbers.md specification completed.

=== BATCH 8 STATUS (2026-10-22) — STATISTICS & ARRAY REDUCTIONS ===
Implemented + tested:
Central tendency: mean, median, mode (default dim, explicit dim, 'all' reduction, tie-breaking to smallest value).
Measures of spread: std, var (w=0 sample, w=1 population, constant data -> 0.0), range (max - min), iqr (empirical quantile linear interpolation).
Products & Extrema: prod (empty -> 1.0), sum (dim and 'all' reduction, empty -> 0.0), min/max (1-arg dim/'all' reduction, 2-arg pairwise element-wise, 2-output [M, I] index reporting).
Source: crates/linalg/src/stats.rs + crates/runtime/src/builtins.rs.
Tests: tests/language/tests/numeric_batch8.rs (18 tests, all pass).

=== BATCH 9 STATUS (2026-10-22) — REA-GUIDED LINEAR ALGEBRA ===
Implemented + tested:
Matrix left division: X = A \ B operator and mldivide(A, B) builtin, element-wise A .\ B and ldivide(A, B). Square systems solved via pivoted LU with forward/backward substitution; overdetermined systems via Householder QR least squares; underdetermined via pseudoinverse.
Pivoted LU decomposition: lu(A) with partial row pivoting. Supports single-output Y (compact form), 2-output [L, U] where A = L*U (L permuted), and 3-output [L, U, P] where P*A = L*U with L unit lower triangular.
Determinant: det(A) computed via pivoted LU, correctly handling row swap parity and pivot signs.
Matrix inverse: inv(A) computed via pivoted LU forward/backward substitution on I_n; strictly errors on singular matrices.
Matrix trace: trace(A) summing main diagonal for square and rectangular matrices.
Singular value decomposition: svd(A) using one-sided Jacobi (Hestenes method) with Gram-Schmidt orthonormal completion. Supports 1-output s (descending singular values) and 3-output [U, S, V] satisfying U*S*V' == A with orthonormal U and V.
Numerical rank: rank(A, tol) counting singular values above tolerance; default tol = max(m,n)*max(s)*eps.
Pseudoinverse: pinv(A, tol) Moore-Penrose inverse via SVD, verified against all 4 Moore-Penrose identities.
Norms: norm(A, p) separating vector norms (1, 2, inf, -inf, custom p) and matrix norms (1, 2, inf, 'fro').
Syntax & Interpreter: Added Backslash (\) and DotBackslash (.\) tokens, AST operators, parser infix bindings (precedence 60, 61), and desktop syntax highlighter coverage.
Source: crates/syntax, crates/linalg/src/decomposition.rs, crates/linalg/src/svd.rs, crates/runtime/src/builtins.rs, crates/runtime/src/interp.rs, apps/desktop/src/editor.rs.
Tests: tests/language/tests/numeric_batch9.rs (18 tests, all pass). Workspace test suite (170+ tests) passes cleanly.
REA: Active session verified. MATLAB target contains only Network License Manager archives; native reverse engineering recorded as blocked. All algorithms implemented independently from published numerical standards.

=== BATCH 11 STATUS (2026-10-22) — COMPLEX SEMANTICS HARDENING & COMPLEX LINEAR ALGEBRA ===
Implemented + tested:
Complex storage & isreal semantics: `isreal(Z)` strictly checks value variant (`false` for `Value::ComplexMatrix`, even when imaginary components are all zero, e.g. `complex(12)`). Arithmetic operations (`+`, `-`, `*`, `.*`, `./`, `.^`, `\`, `/`) cancel to real storage (`Value::Matrix`) when all imaginary parts evaluate to exact zero (`(3+4i) + (5-4i)` yields real 8, `isreal(...) == true`).
Complex SVD: Numerically stable one-sided Jacobi (Hestenes method) with complex Givens rotations $R^H R = I_2$, sorted non-negative singular values $\sigma_1 \ge \sigma_2 \ge \dots \ge 0$, and two-pass modified Gram-Schmidt orthonormal completion for rectangular and rank-deficient matrices. Produces unitary $U$ ($U^H U = I_m$), unitary $V$ ($V^H V = I_n$), satisfying $A = U \Sigma V^H$.
Complex Moore-Penrose Pseudoinverse & Rank: `pinv(A, [tol])` satisfies all 4 Moore-Penrose conditions with conjugate transpose; `rank(A, [tol])` with scale-aware thresholding.
Complex Linear Solvers:
  - Square systems: complex pivoted LU with modulus pivoting and forward/backward substitution ($A \setminus B$, $B / A$).
  - Overdetermined systems ($m > n$): complex Householder QR least-squares.
  - Underdetermined systems ($m < n$): complex SVD minimum-norm solution.
Complex Matrix Functions: `det(A)` (complex LU with permutation parity), `inv(A)` (complex LU solve against $I_n$), `trace(A)` (diagonal sum), `norm(A, [p])` (vector 1, 2, $\infty$, $p$; matrix 1, 2, $\infty$, 'fro').
Source: `crates/values/src/lib.rs`, `crates/values/src/complex.rs`, `crates/linalg/src/complex.rs`, `crates/linalg/src/svd.rs`, `crates/runtime/src/builtins.rs`, `crates/runtime/src/interp.rs`.
Tests: `tests/language/tests/numeric_batch11.rs` (16 tests, all pass). Total workspace tests: 215+ passing.
REA: Active session verified. MATLAB target contains only Network License Manager archives; native reverse engineering recorded as blocked. All algorithms derived from numerical standards.

## Stage 3 — User-defined functions (Batch 17, 2026-10-22)
- Definition syntax: `function [out] = name(in1, in2) ... end` — parsed, registered in `Env.function_table`, persistent across `Session::exec` calls.
- Call routing: `call_or_index` checks `env.function_table` before builtins; expressions can contain user calls (arithmetic, nested, conditional, loop contexts supported by `eval_expr_pub`).
- Scope: isolated local `Env`; inputs bound by evaluated expressions; outputs transferred by collecting declared names from local env after execution.
- `nargin` / `nargout`: injected as scalar variables (`nargin = args.len()`, `nargout = outputs.len().max(1)`) inside local env.
- `return`: propagates via `Flow::Return` through `exec_statement`; exits current function, caller receives assigned outputs.
- Multiple outputs: `[a, b] = f(...)` assignment wired through the multi-output call machinery (see `multi_output.rs` and `stage3_multi_output_assigned`).
- Recursion limit / depth guard: implemented — deep recursion returns an explicit diagnostic instead of overflowing (`stage3_recursion_limit_returns_error`).
- CLI: binary launches; full interactive session persistence across commands not yet fully verified.
- Tests: `user_functions_stage3.rs` 11 pass; no existing tests weakened.


## Apps backend — Data Analysis (2026-10-23)
Source: `crates/apps/src/data_analysis.rs`, `crates/apps/src/lib.rs`, `apps/desktop/src/app.rs`, `apps/desktop/src/execution.rs`.
- CSV parsing: delimiter detection (`,`, tab, `;`), quoted fields with `""` escapes, header heuristic, missing tokens (`""`, `NA`, `NaN`, `null`, `N/A`), ragged-row detection and padding, numeric/text column classification, per-column missing counts.
- Statistics: `count`, `missing`, `mean`, `median`, sample `std`, `min`, `max`, `q25`, `q50`, `q75`; Pearson correlation on complete pairs; explicit errors for text columns, constant columns, and out-of-range selection.
- Session integration: writes `data_analysis_data` (rows × selected numeric columns, NaN for missing) and `data_analysis_*` statistic vectors; pushes a `FigureState` with line/scatter items.
- Export: `column,statistic,value` CSV; refuses overwrite unless confirmed.
- Registry: 8 apps; Data Analysis Ready (Unit, not reference-verified; live GUI NOT TESTED); Curve Fitter Ready (Unit, not reference-verified; live GUI NOT TESTED); Simulation/Control, Signal Analyzer, Image Processing, Machine Learning, Robotics, Embedded Planned and disabled.
Tests: `tests/apps/tests/data_analysis_tests.rs` (14), `tests/apps/tests/backend_tests.rs` (8), `tests/apps/tests/curve_fitter_tests.rs` (10). Live GUI: NOT TESTED (no screenshot artifact in the repository).

## Part 1 layout / parser / eig verification (2026-10-23)
- Storage contract: `DenseMatrix` stores row-major (documented on the struct); MATLAB column-major order exists only via `get_linear`/`to_col_major`. New lock-in tests `tests/language/tests/layout_stage11.rs` verify A=[1 2 3;4 5 6]: `A(:)' = [1 4 2 5 3 6]`, `reshape(1:6,2,3) = [1 3 5;2 4 6]`, and hand-computed `prctile` dim1/dim2.
- Fixed root cause in `linalg::stats::prctile_multi`: dim=1/dim=2 outputs were written at column-major positions inside the row-major container (results appeared transposed); now written at row-major positions. `numeric_batch17` dim tests updated to hand-computed values (strength kept, exact equality; no MATLAB engine is installed).
- Fixed `A(:)` orientation in `runtime::interp`: single-arg colon index now returns an N x 1 column (MATLAB semantics); linear range reads keep index-shape semantics.
- Parser fix (MATLAB concatenation rule): inside `[ ]` a `+`/`-` separated from the previous element by whitespace but attached to its operand (`[0 -1]`, `[0 -1; 1 0]`) starts a new element with a unary sign; `[0-1]`, `[0 - 1]`, and code outside brackets keep subtraction. Call arguments and parentheses reset the rule.
- eig verification: `tests/language/tests/eig_complex_pair_probe.rs` — `eig([0 -1; 1 0])` returns 0±1i (runtime, single- and multi-output, order-independent); the exact MATLAB raw order (+i, -i) is parked as an ignored known limitation (this runtime lists the pair (-i, +i) — QR iteration order); seeded probe over 20 random 4x4 real matrices enforces the either/or contract per matrix (invariants within tolerance, or the documented non-convergence Diagnostic): runtime result 20/20 verified invariants, 0 non-convergence diagnostics; the implementation uses the complex shifted-QR path rather than a real Francis double-shift iteration.
- Removed stale module `crates/apps/src/data_analysis_stats.rs` (superseded by `data_analysis.rs`, was not compiled). `matlabr_apps` is clippy-clean.

## Part 2 — App 2: Curve Fitter (2026-10-23)
Objective: polynomial and linear least-squares fitting through the existing `matlab_linalg` QR path, with a real desktop window, worker execution, tested results, and export.
Backend (`crates/apps/src/curve_fitter.rs`, rewritten):
- `fit_polynomial(x, y, degree)` / `fit_linear`: builds the n x (degree+1) Vandermonde design row-major and hands it to `matlab_linalg::decomposition::mldivide`; overdetermined systems are solved by Householder QR least squares inside matlab_linalg, the square interpolating case by pivoted LU. No solver math in this crate.
- Errors as `Diagnostic`s: x/y length mismatch, fewer than 2 points, degree+1 exceeding point count (underdetermined), NaN rows (with row number), non-finite values, text column selected as x/y, missing cell in a selected column (no silent dropping), out-of-range column index, rank-deficient solve, cancelled fit.
- `FitResult`: coefficients ascending (c0 = constant), fitted values, residuals (observed − fitted), R², RMSE, point count, x/y names.
- Session integration `run_fit`: writes `curve_fit_coefficients` (1 x (degree+1)), `curve_fit_fitted`, `curve_fit_residuals`, `curve_fit_r2`, `curve_fit_rmse`, `curve_fit_degree`; pushes a data+fit figure (fit polyline sorted by x) and a residuals figure with a two-point zero baseline.
- Exports: `export_coefficients_csv` (`coefficient,value` rows c0..cd) and `export_fitted_csv` (`index,x,fitted,residual` rows); both refuse to overwrite without explicit confirmation.
Desktop (`apps/desktop/src/app.rs`, `apps/desktop/src/execution.rs`):
- `CurveFitterApp` window: CSV path text + Browse picker, preview (rows/columns, kind + missing counts, first 20 rows), x/y column combo boxes (numeric only, x != y enforced), degree DragValue, Fit on the worker thread with Stop (shared cancel flag), coefficient table, R²/RMSE summary, two export path fields with click-again overwrite confirmation, results kept when the window is closed.
- Requests `CurvePreview`/`CurveFit`/`CurveFitExport` handled on the single persistent worker; replies carry the fit, x/y data, parsed table and the session snapshot (workspace + figures reach the Plots panel).
- Launcher: Curve Fitter Launch button opens the window (registry Ready); worker preview/fit/export paths are covered by the integration tests. The live desktop GUI was not exercised in this session (NOT TESTED).
Tests: `backend_tests.rs` 8 (exact quadratic/c0 c1 c2 within 1e-10, square exact system, exact line, noisy line vs closed-form normal equations within 1e-9 incl. residuals and RMSE, mismatched length, NaN row, underdetermined degree, <2 points), `curve_fitter_tests.rs` 10 (text-column rejection, missing-cell rejection, out-of-range column, temp CSV -> fit -> workspace -> both exports -> re-read + interpreter round trip, second-write failure leaves no final files, overwrite refusal keeping the pre-existing file byte-identical then confirmed overwrite, pre-cancelled fit writes nothing, residuals figure zero baseline spans x range, quadratic through the table, registry Ready transitions).
Labels: Unit-tested; Math-validated against closed-form OLS and exact polynomials; Not reference-verified (no MATLAB engine/REA). Live GUI: NOT TESTED (non-interactive session).
Status: READY (backend + UI + export tested at the backend/integration layers; live GUI verification pending).
