# RustLab Technical Audit: Core Gaps, Capability Analysis, and Roadmap

**Date:** October 2026  
**Auditor:** Senior Systems Engineer & RustLab Core Architect  
**Scope:** Evaluation of existing crates (`rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-stdlib`, `rustlab-matio`, `rustlab-cli`) against full MATLAB compatibility and the 8 Target Capability Areas.

---

## 1. Executive Summary & Verification Baseline

Before evaluating missing features, the current workspace state was verified against strict quality bars:

| Check | Command | Result | Details |
| :--- | :--- | :--- | :--- |
| **Unit & Integration Tests** | `cargo test --all` | **PASSED** | 59/59 tests passing across all 6 crates (0 failures, 0 ignored). |
| **Clippy Linting** | `cargo clippy --all-targets -- -D warnings` | **PASSED** | Clean (0 errors, 0 warnings). |
| **Code Formatting** | `cargo fmt --all -- --check` | **PASSED** | 100% compliant with Rust formatting guidelines. |
| **Benchmark Validation** | `cargo run --release --bin rustlab -- bench` | **PASSED** | 11.82 GFLOPS GEMM, 0.487 ms 100x100 LU solve, 936 µs 4096-pt FFT. |

While the initial seven milestones established a solid column-major numerical foundation, significant language, type system, and numerical gaps must be resolved to transform RustLab into a full-scale MATLAB platform.

---

## 2. Comprehensive Gap Analysis

### 2.1 Linear Algebra Completeness
| Function | Current State | Required MATLAB Behavior | Target Engine / Approach |
| :--- | :--- | :--- | :--- |
| `eig` | **Missing** | Computes eigenvalues and right eigenvectors: `d = eig(A)` or `[V, D] = eig(A)`. Handles symmetric/Hermitian and general non-symmetric matrices with complex conjugate pairs. | Symmetric: Tridiagonalization via Householder + implicit QL/QR algorithm. General: Hessenberg reduction + Francis double-shift QR algorithm. |
| `svd` | **Missing** | Singular Value Decomposition: $A = U \Sigma V^T$. Returns singular values `s = svd(A)` or unitary factors `[U, S, V] = svd(A)`. Supports economy size `svd(A, 'econ')`. | Golub-Kahan bidiagonalization followed by implicit QR with Wilkinson shifts, or LAPACK/faer SVD kernel. |
| `pinv` | **Missing** | Moore-Penrose pseudoinverse: $A^+ = V \Sigma^+ U^T$, setting singular values below tolerance $r \cdot \sigma_1 \cdot \varepsilon$ to zero. | Implemented cleanly on top of `svd`. |
| `expm` | **Missing** | Matrix exponential $e^A$ for square matrices. Essential for LTI simulation ($\dot{x} = Ax \implies x(t) = e^{At} x_0$). | Scaling and squaring algorithm with [6/6] or [13/13] Padé rational approximants. |
| `null` | **Missing** | Orthonormal basis for null space: $A \cdot Z = 0$. | Computed from columns of $V$ in SVD corresponding to singular values $\le \text{tol}$. |
| `orth` | **Missing** | Orthonormal basis for range (column space) of $A$. | Computed from columns of $U$ in SVD corresponding to singular values $> \text{tol}$. |
| `rank` | **Missing** | Matrix numerical rank: count of singular values greater than $\max(m, n) \cdot \sigma_1 \cdot \varepsilon$. | Implemented on top of `svd`. |
| `cond` | **Missing** | 2-norm condition number: $\kappa_2(A) = \sigma_{\max} / \sigma_{\min}$. | Computed from ratio of extreme singular values in `svd`. |
| `kron` | **Missing** | Kronecker tensor product $K = A \otimes B$, where each block is $a_{ij} B$. | Efficient stride-mapped block assignment into shape $[m_A \cdot m_B, n_A \cdot n_B]$. |
| `qr` with pivoting | **Partial** | Current `qr` only implements unpivoted Gram-Schmidt. Missing column-pivoted QR: $A P = Q R$ where $P$ is a permutation matrix. | Householder reflections with greedy column norm pivoting. |

### 2.2 Transforms & Spectral Analysis
| Capability | Current State | Gap & Requirement |
| :--- | :--- | :--- |
| **Non-power-of-2 FFT** | Naive $O(N^2)$ fallback | Currently falls back to direct $O(N^2)$ sum for non-power-of-2 lengths. For $N = 1000$ or arbitrary primes, this causes orders-of-magnitude slowdowns. Must implement Bluestein's Chirp-Z transform ($O(N \log N)$ convolution) or leverage `rustfft`. |
| **2D FFT (`fft2`, `ifft2`)** | **Missing** | 2D Fourier transform across rows then columns. Essential for 2D image processing (B4). |

### 2.3 Function Handles, Closures, and Functional Programming
| Capability | Current State | Gap & Requirement |
| :--- | :--- | :--- |
| **Function Handles (`@func`)** | **Missing** | Named function handles (e.g., `h = @sin`) do not exist. Must be represented as a first-class `Value::FunctionHandle`. |
| **Anonymous Functions & Closures** | Pseudo-implemented via static `"@anonymous"` | Currently stores a single static function definition under `"@anonymous"`, with zero closure capture. Overwrites prior definitions. Must capture lexical scope snapshot and return unique instance ID. |
| **`cellfun`, `arrayfun`, `structfun`** | **Missing** | Higher-order functional mapping over heterogeneous cells, matrices, and struct fields with `'UniformOutput'` handling. |

### 2.4 Text Processing, String Library, and Formatted I/O
| Capability | Current State | Gap & Requirement |
| :--- | :--- | :--- |
| **`sprintf`, `fprintf`, `disp`** | **Missing** | C-style format strings (`%d`, `%f`, `%10.4e`, `%s`), vector argument expansion, printing to standard output/file descriptors. |
| **`num2str`, `mat2str`** | **Missing** | Converts numeric matrices and arrays to human-readable strings or evaluatable MATLAB string expressions. |
| **String Functions** | **Missing** | `strsplit`, `strrep`, `regexp`, `regexprep`, `upper`, `lower`, `strcmp`, `strcmpi`, `strfind`, `contains`, `startsWith`, `endsWith`. |

### 2.5 Indexing, Slicing, and Tensor Dimensionality
| Capability | Current State | Gap & Requirement |
| :--- | :--- | :--- |
| **Logical Indexing** | **Missing** | `A(A > 2)` or `A(mask)` is unsupported. In MATLAB, logical indexing compresses matching elements into a column vector, and allows masked assignment `A(mask) = val`. |
| **Nested `end` Contexts** | Flat context argument | Evaluating `A(1:end-B(end))` or nested indices fails to track the correct dimensional bound for inner vs outer indexing. Requires a thread/evaluator-local stack of `IndexingContext`s. |
| **N-D Array Slicing & Operations** | Hardcoded to 2D | Multi-dimensional slicing `A(1:2, 1:3, 1:4)` and reductions along dimension $k > 2$ (`sum(A, 3)`) are hardcoded to 2D matrices. |

### 2.6 Numeric Types and Arithmetic Rules
| Capability | Current State | Gap & Requirement |
| :--- | :--- | :--- |
| **Integer Types & Saturation** | Only `Int32`, `Int64` without saturation | MATLAB provides `int8`, `uint8`, `int16`, `uint16`, `int32`, `uint32`, `int64`, `uint64`. In MATLAB, integer arithmetic **saturates** rather than overflowing or wrapping: e.g., `uint8(250) + uint8(10) == 255`. |
| **Single Precision** | `Single` variant exists, incomplete ops | `single` promotion and operations must match IEEE-754 single-precision rules. |
| **Sparse Matrices (CSC)** | **Missing** | Compressed Sparse Column (`sparse`, `full`, `nnz`, `spzeros`, `speye`) format required for large-scale engineering systems. |
| **NaN/Inf Semantics** | Constants defined, builtins missing | `isnan`, `isinf`, `isfinite` builtins missing. Comparisons with `NaN` must evaluate to `false` for relational operators except `~=`. |

### 2.7 Calling Conventions, Error Control, and Path Resolution
| Capability | Current State | Gap & Requirement |
| :--- | :--- | :--- |
| **`nargin`, `nargout`** | **Missing** | Inside user functions, `nargin` and `nargout` query the exact count of supplied inputs and requested outputs. |
| **`varargin`, `varargout`** | **Missing** | Variable argument capture and multiple output packaging into cell arrays. |
| **Diagnostics: `error`, `warning`, `assert`** | **Missing** | User-invoked exception raising (`error`), non-halting diagnostic emission (`warning`), and boolean sanity assertion (`assert`). |
| **Script vs Function Files** | Flat string evaluation | Script files must execute in the caller's workspace; function files must define isolated functions. Path search order (`which`, `exist`, `addpath`, `rmpath`, `path`) must locate `.m` files on disk dynamically. |

---

## 3. Architecture Proposal & Evaluation: Linear Algebra Core (A2)

For completing the advanced linear algebra routines (`eig`, `svd`, `pinv`, `expm`, `qr` with pivoting), two main architecture choices exist:

1. **Pure Clean-Room Native Algorithms in `rustlab-math`:**
   - *Pros:* Complete control, pedagogical transparency, zero heavy external C/Fortran bindings, clean-room verified.
   - *Cons:* Writing industrial-grade general complex eigenvalue decomposition ($QZ$ algorithm) and Jacobi/Divide-and-Conquer SVD from scratch requires extensive numerical stabilization.
2. **Integration of `faer` (or `nalgebra`):**
   - `faer`: Pure Rust, MIT-licensed, high-performance linear algebra library targeting native CPU architecture with zero C/Fortran dependencies. Provides state-of-the-art SVD, Eigenvalue, Schur, and pivoted QR decompositions.
   - `nalgebra`: Apache-2.0, widely used, but primarily optimized for fixed-size 2D/3D robotics math rather than large dynamic matrix decompositions.
   - *Recommendation:* Implement clean native algorithms for standard structural transforms (`pinv`, `expm`, `null`, `orth`, `rank`, `cond`, `kron`), and evaluate leveraging pure-Rust `faer` (under an optional feature flag) for production-grade general non-symmetric $O(N^3)$ eigenvalue and SVD kernels, while maintaining a pure clean-room fallback.

---

## 4. Proposed Execution Ordering

To ensure stability and adhere to the rule of stopping for user approval after each milestone, the following phased sequence is proposed:

```
[Phase A: Language & Core Gaps]
  ├── A1: Function Handles, Closures, varargin/varargout, nargin/nargout, String Library, printf
  ├── A2: Linear Algebra Completion (eig, svd, pinv, expm, null, orth, rank, cond, kron)
  └── A3: Numeric Types (single, int8..uint64 with saturation), Logical Indexing, N-D Parity, NaN/Inf

[Phase B: Capability Area Crates]
  ├── B1: rustlab-math-tools (Root finding, calculus, ODEs: ode45/ode23, optimization: fminsearch/linprog)
  ├── B2: rustlab-data       (Tables, CSV/Excel/JSON I/O, advanced stats, plotters graphics engine)
  ├── B3: rustlab-sim        (LTI systems, state-space, transfer functions, step/bode, simulation engine)
  ├── B4: rustlab-signal     (rustfft/Bluestein, filter design, audio I/O via hound, image ops via image)
  ├── B5: rustlab-ml         (Preprocessing, regressions, clustering, SVM, decision trees, MLP neural net)
  ├── B6: rustlab-robotics   (SO(3)/SE(3) transforms, forward/inverse kinematics, trajectory, PID/LQR, A*/RRT)
  ├── B7: rustlab-lang       (M-file path resolution, tic/toc profiler, test framework, parfor via rayon)
  └── B8: rustlab-hw         (Mockable serial port, TCP/UDP sockets, MISRA-compliant C99 code generator)
```

### Detailed Justification for Ordering:
1. **Phase A1 must come first**: Every advanced toolbox (ODE solvers like `ode45(@f, tspan, y0)`, optimization like `fminsearch(@cost, x0)`, and functional iteration `cellfun`) requires first-class function handles and anonymous functions with closure capture.
2. **Phase A2 follows immediately**: ODE solvers and state-space simulations (`rustlab-sim`) require matrix exponentials (`expm`), eigenvalues (`eig`), and singular values (`svd`).
3. **Phase A3 completes data foundations**: Image processing (`rustlab-signal`) requires `uint8` with saturation; data science (`rustlab-data`) requires logical masking and missing-value (`NaN`) semantics.
4. **Phase B proceeds crate-by-crate**: Each capability area builds upon a verified, complete core engine.
