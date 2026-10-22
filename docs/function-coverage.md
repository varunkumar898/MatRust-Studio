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
| `rand(m,n)` | missing | — | — | — | — | implement on session-held deterministic PRNG |
| `randn(m,n)` | missing | — | — | — | — | Box–Muller over same PRNG |
| `randi(max,m,n)` | missing | — | — | — | — | range rules incl. `[lo hi]` overload |
| `randperm(n)` | missing | — | — | — | — | Fisher–Yates over PRNG |
| `linspace(a,b,n)` | missing | — | — | — | — | implement in linalg arrays module |
| `logspace(a,b,n)` | missing | — | — | — | — | implement in linalg arrays module |
| `a:step:b` (colon operator) | missing | — | — | — | colon token reserved, unimplemented | parser range + interp + row slices |
| `magic(n)` | missing | — | — | — | — | standard Siamese/de la Loubère algorithm |
| `diag(v)` | missing | — | — | — | — | build + extract + k-th diagonal |
| `repmat(A,m,n)` | missing | — | — | — | — | tiling with growth guard |
| `meshgrid(x,y)` | missing | — | — | — | — | 2-D x/y + z overload |
| `ndgrid(x,y)` | missing | — | — | — | — | shares meshgrid grid logic |
| `cat(dim,A,B)` | missing | — | — | — | — | n-array concat along dim (needs N-D later) |
| `horzcat`, `vertcat` | missing | — | — | — | — | row/column cell concat |
| `blkdiag(A,B)` | missing | — | — | — | — | block diagonal assembly |

## 2. Array information and manipulation (24)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `size(A)` | partial | runtime::builtins | — | compatibility:size_zeros_ones_eye; matrices:scalars | only 1/2-arg forms; no `[r,c]=size(A)` multi-output; dims>2 unsupported | multi-out assignment when language supports it |
| `length(A)` | implemented | runtime::builtins | — | compatibility:length_and_numel | longest dim of 2-D | — |
| `numel(A)` | implemented | runtime::builtins | — | compatibility:length_and_numel | — | — |
| `ndims(A)` | missing | — | — | — | — | with 2-D model returns 2 or fewer |
| `isempty(A)` | missing | — | — | — | — | trivial once empty literal `[ ]` parseable |
| `reshape(A,m,n)` | missing | — | — | — | — | column-major reorder check |
| `squeeze(A)` | missing | — | — | — | — | with N-D landing |
| `permute(A,order)` | missing | — | — | — | — | 2-D permutations first |
| `transpose(A)` / `A'` | partial | linalg::transpose | — | matrices:matrix_multiply_is_not_elementwise | real values only; `'` not conjugate (no complex) | conj when complex lands |
| `flip`, `fliplr`, `flipud` | missing | — | — | — | — | trivial kernels + tests |
| `rot90(A)` | missing | — | — | — | — | k-step rotation |
| `circshift(A,k)` | missing | — | — | — | — | per-dim shift vector |
| `sort(A)` | missing | — | — | — | — | column sort first, dim argument, NaN-last rules |
| `sortrows(A,col)` | missing | — | — | — | — | stable sort by col(s) |
| `unique(A)` | missing | — | — | — | — | sorted unique; multi-output later |
| `find(cond)` | missing | — | — | — | — | 1/2-output linear+subscripts; G=matrix output |
| `any(A)`, `all(A)` | missing | — | — | — | — | reduce-cols semantics |
| `ismember(a,B)` | missing | — | — | — | — | sorted-set + binary search |
| `setdiff`, `union`, `intersect` | missing | — | — | — | — | sorted-set ops |
| `cumsum`, `cumprod` | missing | — | — | — | — | explain overflow |
| `diff(A)` | missing | — | — | — | — | n-times difference |
| `histcounts(x,edges)` | missing | — | — | — | — | shared binning with histogram |
| `end` (keyword) | missing | — | — | — | — | index context in interp |
| `triu`, `tril` | missing | — | — | — | — | strict/tighten with k argument |

## 3. Elementary math (18)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `abs`, `sign` | partial | runtime::builtins (`abs`) | — | compatibility:scalar_math_functions | `abs` done; `sign` missing | add `sign` to same map |
| `sqrt`, `nthroot(x,n)` | partial | runtime::builtins (`sqrt`) | — | compatibility:scalar_math_functions | `sqrt` real-only (no complex); `nthroot` missing | complex roots after complex type |
| `exp`, `log`, `log2`, `log10` | partial | runtime::builtins (`exp`,`log`) | — | compatibility:scalar_math_functions | `log2`,`log10` missing | same unary-map path |
| `power(a,b)` / `.^` | partial | linalg::elementwise_pow | — | matrices:elementwise_power_and_division | f64 powf semantics; no integer-pow rules | integer semantics with integer types |
| `sin`, `cos`, `tan` | missing | — | — | — | — | unary map |
| `sind`, `cosd`, `tand` | missing | — | — | — | — | degree wrappers |
| `asin`, `acos`, `atan`, `atan2(y,x)` | missing | — | — | — | — | domain rules; atan2 quadrant tests |
| `sinh`, `cosh`, `tanh` | missing | — | — | — | — | unary map |
| `deg2rad`, `rad2deg` | missing | — | — | — | — | trivial |
| `hypot(a,b)` | missing | — | — | — | — | IEEE hypot (no overflow) |
| `floor`, `ceil`, `round`, `fix` | partial | runtime::builtins | — | compatibility:scalar_math_functions | floor/ceil/round done (round = half-away-from-zero); `fix` missing | add `fix` |
| `mod(a,b)`, `rem(a,b)` | missing | — | — | — | — | sign rules incl. negative operands |
| `gcd`, `lcm` | missing | — | — | — | — | Euclid; scalar first |
| `factorial(n)`, `nchoosek(n,k)` | missing | — | — | — | — | overflow/rounding documented |
| `primes(n)`, `isprime(n)`, `factor(n)` | missing | — | — | — | — | sieve |
| `real`, `imag`, `conj`, `angle` | missing | — | — | — | needs complex type | after complex type |
| `bitand`, `bitor`, `bitxor`, `bitshift` | missing | — | — | — | — | i64/u64 lanes |
| `dec2bin`, `bin2dec`, `dec2hex`, `hex2dec` | missing | — | — | — | — | text round-trips |

## 4. Statistics (11)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `sum`, `prod` | partial | linalg::reduce_cols (`sum`) | — | compatibility:column_reductions_match_matlab_defaults | `sum` only; `prod` missing | add prod reduction |
| `mean`, `median`, `mode` | missing | — | — | — | — | mean then median/mode (mode multi-modal note) |
| `std`, `var` | missing | — | — | — | — | Welford; note normalization choice |
| `min`, `max` | partial | linalg::reduce_cols | — | compatibility:column_reductions_match_matlab_defaults | values only; no index output; 2-arg elementwise max/min missing | add index + elementwise overloads |
| `range`, `iqr` | missing | — | — | — | — | range = max-min; iqr needs quantiles |
| `prctile`, `quantile` | missing | — | — | — | — | linear interpolation method |
| `corrcoef`, `corr`, `cov` | missing | — | — | — | — | cov matrix then correlations |
| `movmean`, `movmedian`, `movsum` | missing | — | — | — | — | window handling (edges) |
| `normalize` | missing | — | — | — | — | methods: data/z/scale/range |
| `rescale` | missing | — | — | — | — | trivial |
| `cummax`, `cummin` | missing | — | — | — | — | running extrema |

## 5. Linear algebra (14)

| Name | Status | Implementation | Dependencies | Tests | Limitations | Next action |
|---|---|---|---|---|---|---|
| `A*B`, `A.*B` | implemented | linalg::matmul / elementwise_mul | — | matrices:matrix_multiply_* | scalar broadcast like MATLAB | — |
| `A\b` | missing | — | — | — | no `\` token; `/` uses Gauss-Jordan inverse (documented divergence) | LU solve is the required first solver |
| `inv(A)`, `pinv(A)` | partial | linalg::matrix_inv (internal only) | — | matrices:matrix_power_and_inverse_division | inverse exists for `/`; no `inv` builtin; `pinv` missing | expose `inv`; SVD-based `pinv` |
| `det(A)` | missing | — | — | — | — | LU with partial pivoting |
| `rank(A)` | missing | — | — | — | — | via RREF threshold |
| `trace(A)` | missing | — | — | — | — | trivial |
| `norm(A)` | missing | — | — | — | — | p-norms (1,2,inf,fro) |
| `eig(A)` | missing | — | — | — | real 2x2 possible by hand; general needs complex QZ/eig | real Schur or LAPACK-style dependency decision |
| `svd(A)` | missing | — | — | — | — | Golub–Kahan one-sided, small matrices |
| `lu`, `qr`, `chol` | missing | — | — | — | — | LU first (also powers `A\b`) |
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
