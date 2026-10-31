# MatRust Studio Numerical Capabilities

## 1. Overview

MatRust Studio includes an extensive clean-room mathematics engine supporting numerical linear algebra, elementary functions, calculus, and approximation theory.

## 2. Capabilities Table

| Capability | Builtin Commands | Rust Backend Module | Implementation Method |
|---|---|---|---|
| Matrix Arithmetic | `+`, `-`, `*`, `.*`, `/`, `./`, `^`, `.^` | `rustlab-math::ops` | Column-major dense and sparse arithmetic with broadcasting |
| Linear System Solve | `mldivide` (`\`), `mrdivide` (`/`) | `rustlab-stdlib::linalg` | Gaussian elimination with partial pivoting / LU / QR |
| Matrix Decompositions | `lu`, `qr`, `chol`, `svd`, `eig` | `rustlab-stdlib::linalg` | Clean-room Householder, QL with Wilkinson shifts, Cholesky, and pivoted QR |
| Matrix Functions | `inv`, `det`, `rank`, `cond`, `trace`, `norm`, `expm` | `rustlab-stdlib::linalg` | Scaling and squaring with Padé approximants (`expm`), singular values (`cond`, `norm`) |
| Polynomial Roots | `roots` | `rustlab-stdlib::numerical` | Companion matrix eigenvalue calculation via `eig` |
| Polynomial Evaluation | `polyval` | `rustlab-stdlib::numerical` | Horner's method |
| Polynomial Curve Fit | `polyfit` | `rustlab-stdlib::numerical` | Vandermonde system with least-squares normal equations |
| Numerical Integration | `trapz`, `cumtrapz` | `rustlab-stdlib::numerical` | Discrete trapezoidal integration with arbitrary or unit spacing |
| Numerical Differentiation | `diff`, `gradient` | `rustlab-stdlib::numerical` | Forward, backward, and second-order central difference gradients |
| 1D Interpolation | `interp1` | `rustlab-stdlib::numerical` | Binary-search interval localization with linear or nearest interpolation |
| Elementary Mathematics | `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`, `log`, `log10`, `log2`, `sqrt`, `round`, `floor`, `ceil`, `fix`, `abs`, `sign`, `mod`, `rem` | `rustlab-stdlib::math_elem` | IEEE-754 elementwise mathematical functions with full matrix support |
| Complex Numbers | `real`, `imag`, `angle`, `conj`, complex arithmetic | `rustlab-stdlib::signal`, `rustlab-math` | Double precision `Complex64` arrays |
