# Milestone A2 Learning Note: Numerical Linear Algebra Algorithms

**Milestone:** A2  
**Author:** RustLab Systems Architecture Team  
**Objective:** Provide a deep, accessible conceptual walkthrough of the numerical linear algebra algorithms implemented in RustLab.

---

## 1. Why Orthogonal Similarity Transforms Preserve Eigenvalues

When analyzing a square matrix $A$, finding eigenvalues directly from the characteristic polynomial $\det(A - \lambda I) = 0$ is numerically catastrophic for $N > 4$ (Abel-Ruffini theorem and extreme root sensitivity, e.g. Wilkinson's polynomial). Numerical algorithms instead apply successive similarity transformations:
$$A_{k+1} = P_k A_k P_k^{-1}$$

If $P_k$ is an **orthogonal** matrix ($P_k^T P_k = I \implies P_k^{-1} = P_k^T$), two critical mathematical properties hold:
1. **Eigenvalue preservation:** If $A v = \lambda v$, then:
   $$(P A P^T)(P v) = P A (P^T P) v = P (A v) = P (\lambda v) = \lambda (P v)$$
   The transformed matrix $P A P^T$ has identical eigenvalues $\lambda$, and transformed eigenvectors $P v$.
2. **Numerical stability:** Orthogonal transformations preserve 2-norms ($\|P x\|_2 = \|x\|_2$) and condition numbers ($\kappa_2(P A P^T) = \kappa_2(A)$). Roundoff errors cannot be exponentially magnified.

---

## 2. Householder Tridiagonalization (`tred2`)

For a symmetric matrix $A = A^T$, working directly on a dense $N \times N$ matrix during iterative eigenvalue search requires $O(N^3)$ operations per iteration. By first reducing $A$ to a **symmetric tridiagonal matrix** $T$, subsequent iterations take only $O(N)$ operations each!

```
Dense Symmetric A                       Tridiagonal T
┌                   ┐               ┌                   ┐
│  *   *   *   *    │               │  d1  e1   0   0   │
│  *   *   *   *    │  ──────────>  │  e1  d2  e2   0   │
│  *   *   *   *    │  Householder  │   0  e2  d3  e3   │
│  *   *   *   *    │  Similarity   │   0   0  e3  d4   │
└                   ┘               └                   ┘
```

### The Householder Reflection
A Householder matrix reflects vectors across a hyperplane orthogonal to a unit vector $u$:
$$P = I - 2 u u^T$$
Given a column vector $x = [x_1, x_2, \dots, x_k]^T$, choosing $u = \frac{x \pm \|x\| e_1}{\|x \pm \|x\| e_1\|}$ reflects $x$ onto the first coordinate axis:
$$P x = [\mp \|x\|, 0, 0, \dots, 0]^T$$

In `tred2`, for each column $i = N-1, N-2, \dots, 2$:
1. A Householder reflection $P_i$ is constructed to zero out all elements in row $i$ to the left of the subdiagonal.
2. The symmetric similarity transformation $P_i A P_i$ applies $P_i$ from both sides, zeroing column $i$ below the subdiagonal while preserving symmetry.
3. Total cost: $\frac{4}{3} N^3$ floating-point operations.

---

## 3. The Implicit QL Algorithm with Wilkinson Shifts (`tql2`)

Once $A$ is reduced to tridiagonal form with diagonal $d$ and subdiagonal $e$, we compute eigenvalues iteratively.

### The Basic QL Step
If $T = Q L$ where $Q$ is orthogonal and $L$ is lower triangular, the reverse product:
$$T_{new} = L Q = Q^T T Q$$
is also tridiagonal and orthogonally similar to $T$. With each step, the off-diagonal elements decay toward zero.

### Accelerated Convergence via the Wilkinson Shift
Unshifted QL converges linearly. To achieve **cubic convergence** (tripling the number of correct digits each iteration), we subtract a shift $\mu$ near an eigenvalue:
$$T - \mu I = Q L, \quad T_{new} = L Q + \mu I$$
The **Wilkinson shift** chooses $\mu$ as the eigenvalue of the trailing $2 \times 2$ submatrix:
$$\begin{pmatrix} d_{k} & e_{k} \\ e_{k} & d_{k+1} \end{pmatrix}$$
closest to $d_k$.

### Implicit QL (Francis Trick)
Forming $T - \mu I$ explicitly can cause catastrophic cancellation when $\mu \approx d_k$. The **implicit QL algorithm** avoids explicit subtraction entirely:
- It computes only the first Givens rotation $G_1$ corresponding to the first column of $T - \mu I$.
- By the **Implicit Q Theorem**, applying a sequence of Givens rotations to "chase the bulge" created by $G_1$ back into tridiagonal form implicitly accomplishes the exact same shifted step without numerical cancellation!

---

## 4. Matrix Exponential via Padé Approximation with Scaling & Squaring

The matrix exponential is defined by the infinite Taylor series:
$$e^A = \sum_{k=0}^\infty \frac{A^k}{k!} = I + A + \frac{A^2}{2} + \dots$$
Evaluating this series directly for large $\|A\|$ suffers from catastrophic cancellation (e.g. $e^{-100}$).

### Higham's Scaling and Squaring Algorithm
1. **Fundamental identity:** $e^A = (e^{A / 2^s})^{2^s}$.
2. **Scaling:** Divide $A$ by $2^s$ until $\|\tilde{A}\|_\infty = \|A / 2^s\|_\infty \le 0.5$.
3. **Diagonal Padé Approximant:** Near zero, a rational function $R_{p, p}(X) = [D_p(X)]^{-1} N_p(X)$ converges much faster than a polynomial of the same degree. For order $p = 6$:
   $$N_6(X) = \sum_{k=0}^6 c_k X^k, \quad D_6(X) = \sum_{k=0}^6 (-1)^k c_k X^k$$
   where $c_k = \frac{(12-k)! 6!}{12! k! (6-k)!}$.
4. **Solving linear system:** Compute $Y = D_6(\tilde{A})^{-1} N_6(\tilde{A})$ via Gaussian elimination with partial pivoting (`mldivide`).
5. **Squaring:** Repeatedly square $Y \leftarrow Y^2$ a total of $s$ times to recover $e^A$.

---

## 5. SVD as the "Swiss Army Knife" of Linear Algebra

The Singular Value Decomposition factors any real $M \times N$ matrix into:
$$A = U S V^T$$
where $U \in \mathbb{R}^{M \times M}$ and $V \in \mathbb{R}^{N \times N}$ are orthogonal, and $S$ contains non-negative singular values $\sigma_1 \ge \sigma_2 \ge \dots \ge 0$ on its diagonal.

From the SVD:
1. **Pseudoinverse (`pinv`):** $A^+ = V S^+ U^T$, inverting non-zero singular values.
2. **Rank (`rank`):** Count of singular values above threshold $\epsilon \cdot \max(M, N) \cdot \sigma_1$.
3. **Null Space (`null`):** The columns of $V$ corresponding to singular values $\le \text{tol}$ form an orthonormal basis for $\ker(A)$.
4. **Range (`orth`):** The columns of $U$ corresponding to singular values $> \text{tol}$ form an orthonormal basis for $\operatorname{im}(A)$.
5. **2-Norm Condition Number (`cond`):** $\kappa_2(A) = \frac{\sigma_{\max}}{\sigma_{\min}}$.
