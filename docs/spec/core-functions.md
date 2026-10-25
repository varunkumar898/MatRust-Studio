# RustLab Specification: Prioritized Top 100 Core Functions

This document specifies the prioritized top 100 standard library functions for `rustlab`, organized by domain, including their signatures, input/output behaviors, and mathematical contracts.

---

## Tier 1: Array Creation & Basic Generators (1–12)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 1 | `zeros` | `zeros(m, n, ...)` | Allocates an $M \times N \times \dots$ array filled with `0.0`. Supports type specification (e.g. `zeros(2, 3, 'uint8')`). |
| 2 | `ones` | `ones(m, n, ...)` | Allocates an array filled with `1.0`. |
| 3 | `eye` | `eye(n)` / `eye(m, n)` | Generates an identity matrix where diagonal elements are $1.0$ and off-diagonals are $0.0$. |
| 4 | `rand` | `rand(m, n, ...)` | Uniformly distributed pseudorandom numbers on interval $(0, 1)$. |
| 5 | `randn` | `randn(m, n, ...)` | Standard normally distributed pseudorandom numbers ($\mu = 0, \sigma = 1$). |
| 6 | `linspace` | `linspace(x1, x2, n)` | Linearly spaced vector of $N$ points (default 100) between $x_1$ and $x_2$, inclusive. |
| 7 | `logspace` | `logspace(a, b, n)` | Logarithmically spaced vector of $N$ points between $10^a$ and $10^b$. |
| 8 | `colon` (`:`) | `colon(a, b)` / `colon(a, d, b)` | Generates regular spaced vector `[a, a+d, a+2d, ...]`. |
| 9 | `true` | `true(m, n)` | Generates a logical array of `true` values. |
| 10 | `false` | `false(m, n)` | Generates a logical array of `false` values. |
| 11 | `meshgrid` | `[X, Y] = meshgrid(x, y)` | 2D and 3D coordinate grids from vectors. |
| 12 | `magic` | `magic(n)` | Generates an $N \times N$ magic square whose row, column, and diagonal sums are identical. |

---

## Tier 2: Array Inspection & Shape Manipulation (13–30)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 13 | `size` | `s = size(A)` / `[m, n] = size(A)` / `d = size(A, dim)` | Returns dimensions of array as a $1 \times K$ vector or multiple return values. |
| 14 | `length` | `l = length(A)` | Returns length of the largest array dimension: $\max(\text{size}(A))$. |
| 15 | `numel` | `n = numel(A)` | Returns total number of elements: $\prod_i \text{size}(A, i)$. |
| 16 | `ndims` | `k = ndims(A)` | Number of array dimensions (minimum 2). |
| 17 | `isempty` | `tf = isempty(A)` | Returns `true` if any dimension size is 0 (`numel(A) == 0`). |
| 18 | `isscalar` | `tf = isscalar(A)` | Returns `true` if $\text{size}(A) == [1, 1]$. |
| 19 | `isvector` | `tf = isvector(A)` | Returns `true` if array is $1 \times N$ or $N \times 1$ ($N \ge 0$). |
| 20 | `ismatrix` | `tf = ismatrix(A)` | Returns `true` if array has exactly 2 dimensions. |
| 21 | `reshape` | `B = reshape(A, m, n, ...)` | Reshapes array elements in column-major order; total element count must be invariant. |
| 22 | `repmat` | `B = repmat(A, m, n)` | Replicates and tiles an array $M \times N$ times. |
| 23 | `squeeze` | `B = squeeze(A)` | Removes singleton dimensions where size is 1 (for dimensions $> 2$). |
| 24 | `transpose` (`.'`) | `B = transpose(A)` | Swaps rows and columns without taking complex conjugate. |
| 25 | `ctranspose` (`'`) | `B = ctranspose(A)` | Hermitian / conjugate transpose. |
| 26 | `fliplr` | `B = fliplr(A)` | Flips matrix horizontally (left to right along column dimension). |
| 27 | `flipud` | `B = flipud(A)` | Flips matrix vertically (up to down along row dimension). |
| 28 | `rot90` | `B = rot90(A, k)` | Rotates matrix by $k \times 90^\circ$ counterclockwise. |
| 29 | `circshift` | `B = circshift(A, k)` | Circularly shifts array elements along specified dimensions. |
| 30 | `diag` | `d = diag(A)` / `A = diag(v)` | Extracts diagonal of matrix or constructs diagonal matrix from vector. |

---

## Tier 3: Elementwise Elementary Math (31–48)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 31 | `abs` | `Y = abs(X)` | Absolute value (modulus $|z| = \sqrt{a^2 + b^2}$ for complex numbers). |
| 32 | `sign` | `Y = sign(X)` | Signum function (returns $1, 0, -1$, or $z/|z|$ for complex). |
| 33 | `sqrt` | `Y = sqrt(X)` | Square root; returns complex result if input has negative reals. |
| 34 | `exp` | `Y = exp(X)` | Natural exponential $e^X$. |
| 35 | `log` | `Y = log(X)` | Natural logarithm $\ln(X)$; returns complex branch for negative inputs. |
| 36 | `log10` | `Y = log10(X)` | Base-10 logarithm. |
| 37 | `log2` | `Y = log2(X)` | Base-2 logarithm. |
| 38 | `sin` | `Y = sin(X)` | Sine in radians. |
| 39 | `cos` | `Y = cos(X)` | Cosine in radians. |
| 40 | `tan` | `Y = tan(X)` | Tangent in radians. |
| 41 | `asin` | `Y = asin(X)` | Inverse sine (arcsine). |
| 42 | `acos` | `Y = acos(X)` | Inverse cosine (arccosine). |
| 43 | `atan` | `Y = atan(X)` | Inverse tangent (arctangent). |
| 44 | `atan2` | `Y = atan2(Y, X)` | Four-quadrant inverse tangent. |
| 45 | `round` | `Y = round(X)` | Rounds to nearest decimal/integer; ties round away from zero. |
| 46 | `floor` | `Y = floor(X)` | Rounds toward $-\infty$. |
| 47 | `ceil` | `Y = ceil(X)` | Rounds toward $+\infty$. |
| 48 | `fix` | `Y = fix(X)` | Rounds toward zero (truncation). |

---

## Tier 4: Reductions & Summary Statistics (49–62)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 49 | `sum` | `S = sum(A, dim)` | Sum of array elements along specified dimension (defaults to first non-singleton). |
| 50 | `prod` | `P = prod(A, dim)` | Product of array elements along specified dimension. |
| 51 | `cumsum` | `B = cumsum(A, dim)` | Cumulative sum along dimension. |
| 52 | `cumprod` | `B = cumprod(A, dim)` | Cumulative product along dimension. |
| 53 | `diff` | `Y = diff(X)` | Difference between adjacent elements: $Y(i) = X(i+1) - X(i)$. |
| 54 | `mean` | `M = mean(A, dim)` | Arithmetic mean of elements along dimension. |
| 55 | `median` | `M = median(A, dim)` | Median value along dimension. |
| 56 | `std` | `S = std(A, flag, dim)` | Standard deviation (normalized by $N-1$ by default, or $N$ when flag=1). |
| 57 | `var` | `V = var(A, flag, dim)` | Variance ($\text{std}^2$). |
| 58 | `min` | `[M, I] = min(A)` | Minimum value and corresponding linear/subscript indices. |
| 59 | `max` | `[M, I] = max(A)` | Maximum value and corresponding indices. |
| 60 | `any` | `tf = any(A, dim)` | True if any element along dimension is non-zero. |
| 61 | `all` | `tf = all(A, dim)` | True if all elements along dimension are non-zero. |
| 62 | `cov` | `C = cov(X, Y)` | Covariance matrix between vectors or columns of matrix. |

---

## Tier 5: Sorting, Searching & Set Operations (63–70)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 63 | `sort` | `[B, I] = sort(A, dim, direction)` | Sorts elements in ascending/descending order and returns permutation indices. |
| 64 | `sortrows` | `[B, I] = sortrows(A, col)` | Sorts matrix rows based on specified columns. |
| 65 | `find` | `indices = find(X)` / `[r, c, v] = find(X)` | Locates non-zero elements; returns linear indices or row/column coordinates. |
| 66 | `unique` | `[C, ia, ic] = unique(A)` | Returns sorted unique values and index mappings. |
| 67 | `ismember` | `[lia, locb] = ismember(A, B)` | Array of logical flags indicating membership of elements in set $B$. |
| 68 | `intersect` | `C = intersect(A, B)` | Set intersection. |
| 69 | `union` | `C = union(A, B)` | Set union. |
| 70 | `setdiff` | `C = setdiff(A, B)` | Set difference. |

---

## Tier 6: Linear Algebra & Matrix Decompositions (71–84)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 71 | `mldivide` (`\`) | `x = A \ b` | Matrix left divide: solves linear system $Ax = b$ via LU / QR / Cholesky / least squares. |
| 72 | `mrdivide` (`/`) | `x = b / A` | Matrix right divide: solves $xA = b$ (equivalent to `(A' \ b')'`). |
| 73 | `inv` | `Y = inv(X)` | Matrix inverse for square matrix. |
| 74 | `det` | `d = det(X)` | Matrix determinant. |
| 75 | `trace` | `t = trace(X)` | Sum of diagonal elements $\sum X_{ii}$. |
| 76 | `rank` | `r = rank(A, tol)` | Matrix rank (number of singular values $> \text{tol}$). |
| 77 | `norm` | `n = norm(X, p)` | Vector or matrix $p$-norm ($p = 1, 2, \infty, \text{'fro'}$). |
| 78 | `cond` | `c = cond(X)` | Condition number for matrix inversion: $\sigma_{\max} / \sigma_{\min}$. |
| 79 | `pinv` | `Y = pinv(X)` | Moore-Penrose pseudoinverse via SVD. |
| 80 | `lu` | `[L, U, P] = lu(A)` | LU factorization with partial pivoting: $PA = LU$. |
| 81 | `qr` | `[Q, R] = qr(A)` | QR decomposition: $A = QR$ ($Q$ orthogonal, $R$ upper triangular). |
| 82 | `chol` | `R = chol(A)` | Cholesky factorization of symmetric positive-definite matrix: $A = R'R$. |
| 83 | `eig` | `[V, D] = eig(A)` | Eigenvalues ($D$) and eigenvectors ($V$): $AV = VD$. |
| 84 | `svd` | `[U, S, V] = svd(A)` | Singular Value Decomposition: $A = U S V'$. |

---

## Tier 7: Signal Processing, Transforms & Complex Functions (85–92)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 85 | `fft` | `Y = fft(X, n, dim)` | 1D Discrete Fourier Transform computed via Fast Fourier Transform. |
| 86 | `ifft` | `Y = ifft(X, n, dim)` | 1D Inverse Discrete Fourier Transform. |
| 87 | `fft2` | `Y = fft2(X)` | 2D Fast Fourier Transform. |
| 88 | `fftshift` | `Y = fftshift(X)` | Shifts zero-frequency component to center of spectrum. |
| 89 | `conv` | `C = conv(u, v)` | Convolution of two 1D vectors / polynomial multiplication. |
| 90 | `real` | `Y = real(Z)` | Real component of complex array. |
| 91 | `imag` | `Y = imag(Z)` | Imaginary component of complex array. |
| 92 | `angle` | `P = angle(Z)` | Phase angle (argument in radians) of complex array. |

---

## Tier 8: Containers, Introspection & Headless Visualization (93–100)

| # | Function | Signature | Description & Behavior |
| :--- | :--- | :--- | :--- |
| 93 | `cell` | `C = cell(m, n)` | Creates empty cell array of specified dimensions. |
| 94 | `struct` | `s = struct('f1', v1, ...)` | Creates structure array with given fields and values. |
| 95 | `fieldnames` | `fn = fieldnames(s)` | Returns field names of struct as a cell array of character vectors. |
| 96 | `isfield` | `tf = isfield(s, 'name')` | Checks if struct has specified field. |
| 97 | `isa` | `tf = isa(obj, 'classname')` | Checks whether object belongs to designated class. |
| 98 | `class` | `name = class(obj)` | Returns string name of object's type (e.g. `'double'`, `'cell'`). |
| 99 | `plot` | `h = plot(x, y, ...)` | Headless plot engine: computes coordinates, line specs, and returns clean renderable figure model. |
| 100 | `scatter` | `h = scatter(x, y, sz, c)` | Headless scatter plot generating serialized point cloud data. |
