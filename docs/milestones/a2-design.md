# Milestone A2 Design Note: Linear Algebra Completion

**Milestone:** A2  
**Author:** RustLab Systems Architecture Team  
**Objective:** Complete linear algebra capabilities for MATLAB parity: native symmetric eigenvalue decomposition, general eigenvalue decomposition and SVD via `faer`, pseudoinverse (`pinv`), matrix exponential (`expm`), rank, null space (`null`), range (`orth`), Kronecker product (`kron`), condition number (`cond`), pivoted QR, and general-length FFT (`fft`, `ifft`, `fft2`, `ifft2`) via `rustfft`.

---

## 1. Third-Party Dependency Licenses & Clean-Room Verification

Under our clean-room guidelines, all third-party crates must have permissive, non-copyleft open-source licenses compatible with commercial and research use (MIT and/or Apache-2.0):

| Crate | Version | License | Primary Purpose in RustLab |
|---|---|---|---|
| `faer` | 0.24.4 | MIT OR Apache-2.0 | General (non-symmetric) EVD, SVD, thin SVD |
| `rustfft` | 6.4.1 | MIT OR Apache-2.0 | High-performance arbitrary-length 1D and 2D FFT/IFFT |
| `num-complex` | 0.4.6 | MIT OR Apache-2.0 | Complex number representations (`Complex64`) |

- **Clean-room rule compliance:** No decompiled MATLAB code or GPL code (e.g. from Octave or LAPACK Fortran source) is copied or transcribed into this codebase. Mathematical algorithms are implemented independently from published textbooks (Golub & Van Loan *Matrix Computations*, Higham *Functions of Matrices*, Trefethen & Bau *Numerical Linear Algebra*).
- **Feature gating:** `faer` is encapsulated behind the optional Cargo feature `faer` (enabled in default features). Pure-native numerical routines (such as Householder tridiagonalization + implicit QL for symmetric `eig`, and Padé approximation for `expm`) function natively without external matrix engines.

---

## 2. Architectural Components

```
                          ┌───────────────────────────┐
                          │    rustlab-stdlib::linalg │
                          └─────────────┬─────────────┘
                                        │
           ┌────────────────────────────┼───────────────────────────┐
           │                            │                           │
           ▼                            ▼                           ▼
┌───────────────────────┐    ┌────────────────────┐    ┌─────────────────────────┐
│     eig_sym.rs        │    │      svd.rs        │    │        expm.rs          │
│ Householder Tridiag   │    │ SVD Decomposition  │    │ Padé Approximant (p=6)  │
│ + Implicit QL (TQL2)  │    │  (faer feature)    │    │ Scaling & Squaring      │
│ Ascending order       │    │                    │    │                         │
│ Sign normalization    │    │                    │    │                         │
└───────────────────────┘    └──────────┬─────────┘    └─────────────────────────┘
                                        │
                         ┌──────────────┴──────────────┐
                         ▼                             ▼
              ┌─────────────────────┐       ┌────────────────────┐
              │ pinv / rank / cond  │       │    null / orth     │
              │ Moore-Penrose       │       │ Orthonormal Bases  │
              └─────────────────────┘       └────────────────────┘
```

### 2.1 Symmetric vs General Eigenvalue Decomposition (`eig`)
- **Symmetric Matrices ($A = A^T$):**
  - Uses native RustLab implementation:
    1. **Householder Tridiagonalization (`tred2`):** Transforms symmetric $A$ into tridiagonal form $T$ using $N-2$ orthogonal Householder reflections $P_k = I - 2 v_k v_k^T$. Accumulates $Q = P_1 P_2 \dots P_{N-2}$.
    2. **Implicit QL Algorithm with Wilkinson Shift (`tql2`):** Iteratively diagonalizes $T$ using implicit Givens rotations with Wilkinson shifts for cubic convergence. Accumulates transformations into $V = Q \cdot Q_{QL}$.
    3. **Eigenvalue Ordering:** Sorted in ascending order $\lambda_1 \le \lambda_2 \le \dots \le \lambda_N$ matching MATLAB conventions for symmetric matrices.
    4. **Sign Normalization:** For each eigenvector column $V_{:, k}$, the element with the maximum absolute value is forced to have a positive sign:
       $$\text{if } V_{i^*, k} < 0 \text{ where } i^* = \operatorname{argmax}_i |V_{i, k}| \implies V_{:, k} \leftarrow -V_{:, k}$$
- **General Matrices ($A \neq A^T$):**
  - Handled via `faer`'s general Schur/EVD solver (`m.eigen()`), producing complex eigenvalues and eigenvectors when appropriate.

### 2.2 Singular Value Decomposition (`svd`)
- Given $A \in \mathbb{R}^{M \times N}$, computes:
  $$A = U S V^T$$
  - `s = svd(A)`: Returns $\min(M, N) \times 1$ column vector of singular values sorted in descending order $\sigma_1 \ge \sigma_2 \ge \dots \ge \sigma_r \ge 0$.
  - `[U, S, V] = svd(A)`: Full SVD ($U$ is $M \times M$, $S$ is $M \times N$, $V$ is $N \times N$).
  - `[U, S, V] = svd(A, 'econ')` or `svd(A, 0)`: Economy-size SVD ($U$ is $M \times K$, $S$ is $K \times K$, $V$ is $N \times K$ where $K = \min(M, N)$).

### 2.3 Moore-Penrose Pseudoinverse (`pinv`)
- Computed from SVD $A = U S V^T$:
  $$A^+ = V S^+ U^T$$
  where tolerance $\text{tol} = \max(M, N) \cdot \sigma_1 \cdot \epsilon_{\text{f64}}$, and:
  $$S^+_{i, i} = \begin{cases} \frac{1}{\sigma_i} & \text{if } \sigma_i > \text{tol} \\ 0 & \text{otherwise} \end{cases}$$
- Properties verified:
  1. $A A^+ A = A$
  2. $A^+ A A^+ = A^+$
  3. $(A A^+)^T = A A^+$
  4. $(A^+ A)^T = A^+ A$

### 2.4 Matrix Exponential (`expm`)
- Computed via Padé approximation with scaling and squaring (Higham 2005 / Golub & Van Loan):
  1. Calculate matrix infinity-norm $\|A\|_\infty$.
  2. Scale $A$: find integer $s \ge 0$ such that $\|\tilde{A}\|_\infty = \|A / 2^s\|_\infty \le 0.5$.
  3. Compute degree-6 diagonal Padé approximant $R_{6, 6}(\tilde{A}) = (D_6(\tilde{A}))^{-1} N_6(\tilde{A})$:
     $$N_6(X) = \sum_{k=0}^6 c_k X^k, \quad D_6(X) = \sum_{k=0}^6 (-1)^k c_k X^k$$
     with coefficients $c_k = \frac{(12 - k)! 6!}{12! k! (6 - k)!}$.
  4. Solve $D_6 Y = N_6$ using Gaussian elimination with partial pivoting (`mldivide`).
  5. Square $s$ times: $e^A = Y^{2^s}$.

### 2.5 Null Space (`null`), Range (`orth`), Rank (`rank`), Condition (`cond`)
- **`rank(A)`:** Number of singular values exceeding $\text{tol} = \max(M, N) \cdot \sigma_1 \cdot \epsilon$.
- **`orth(A)`:** First $r = \operatorname{rank}(A)$ columns of $U$.
- **`null(A)`:** Last $N - r$ columns of $V$ corresponding to singular values $\le \text{tol}$.
- **`cond(A)`:** Ratio $\sigma_{\max} / \sigma_{\min}$. Returns $\infty$ if $\sigma_{\min} = 0$.

### 2.6 Pivoted QR (`[Q, R, P] = qr(A)`)
- Columns of $A$ are permuted so that diagonal entries $|R_{k, k}|$ are non-increasing:
  $$A P = Q R$$
  where $P$ is an $N \times N$ permutation matrix, or returned as a permutation vector when called with `qr(A, 'vector')`.

### 2.7 Kronecker Product (`kron(A, B)`)
- For $A \in \mathbb{R}^{m_A \times n_A}$ and $B \in \mathbb{R}^{m_B \times n_B}$, computes block matrix $C \in \mathbb{R}^{(m_A m_B) \times (n_A n_B)}$ where:
  $$C_{(i-1)m_B + k, (j-1)n_B + l} = A_{i, j} \cdot B_{k, l}$$

### 2.8 Arbitrary-Length FFT Library (`rustfft`)
- `fft(x)` and `ifft(x)`: Replaced naive $O(N^2)$ fallback with `rustfft`'s mixed-radix Bluestein/Cooley-Tukey engine running in $O(N \log N)$ for arbitrary lengths (including prime lengths).
- `fft2(X)` and `ifft2(X)`: 2D FFT computed along columns then rows.
- **Reference Preservation:** The original recursive radix-2 Cooley-Tukey implementation is retained in `rustlab-stdlib/src/signal/radix2.rs` as an educational reference module.
