# Math Gallery Runtime Compatibility Audit (Task MG-0)

## 1. Audit Scope & Executive Summary

This audit establishes the concrete runtime compatibility baseline for the MatRust Studio numerical computing engine across all mathematical, simulation, and visualization functions required by the **Math Gallery**.

Every classification in this document is backed by an independent, reproducible CLI invocation executed through the native `rustlab` CLI executable (`rustlab 0.1.0`), built from repository commit `8e567c5` (PR #56 baseline).

- **Repository Revision Tested:** `8e567c5a449ca919e4f92e91f6622a9d0e119ae3`
- **Branch:** `docs/plan-math-gallery`
- **CLI Version:** `rustlab 0.1.0`
- **Rust Toolchain:** `rustc 1.99.1 (58cf8f9c3 2026-10-13)` (`aarch64-pc-windows-gnullvm`)
- **Cargo Version:** `cargo 1.99.1 (5f94df478 2026-08-27)`
- **Audit Date:** 2026-11-15T16:02:08+05:30
- **Total Probed Functions:** 107
- **EXISTS-WORKS:** 52 (48.6%)
- **MISSING:** 55 (51.4%)
- **EXISTS-WRONG:** 0 (0.0%)
- **CRASH:** 0 (0.0%)
- **UNRESOLVED-PROBE:** 0 (0.0%)

---

## 2. Category Summary Table

| Category | Total Inventory | EXISTS-WORKS | MISSING | CRASH / FAIL | Coverage (%) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Linear Algebra** | 20 | 18 | 2 | 0 | 90.0% |
| **Signal Processing** | 17 | 12 | 5 | 0 | 70.6% |
| **Plotting & Visualization** | 43 | 17 | 26 | 0 | 39.5% |
| **Calculus & Numerical Methods** | 13 | 3 | 10 | 0 | 23.1% |
| **ODE & Simulation** | 5 | 2 | 3 | 0 | 40.0% |
| **Interpolation & Optimization** | 9 | 0 | 9 | 0 | 0.0% |
| **Total** | **107** | **52** | **55** | **0** | **48.6%** |

---

## 3. Key Findings & Architectural Gaps

### 3.1 Implemented But Unregistered Functions
The following functions possess working source implementations in `crates/rustlab-stdlib/src/plot_builtins.rs`, but are currently omitted from `register_all` in `crates/rustlab-stdlib/src/lib.rs`, rendering them unavailable to scripts and the CLI:
- `axis`: Defined in `plot_builtins.rs:434`, but omitted from interpreter registration.
- `colorbar`: Defined in `plot_builtins.rs:414`, but omitted from interpreter registration.
- `colormap`: Defined in `plot_builtins.rs:419`, but omitted from interpreter registration.
- `shading`: Defined in `plot_builtins.rs:424`, but omitted from interpreter registration.
- `view`: Defined in `plot_builtins.rs:429`, but omitted from interpreter registration.

### 3.2 High-Priority Missing Engine Capabilities
1. **Ordinary Differential Equations (`ode45`, `ode23`, `odeset`)**: Fully missing from the runtime engine. Wave 2 control/simulations rely on Simulink block diagrams (`mlapp-sim`), but script-level ODE solvers do not exist.
2. **Numerical Quadrature (`integral`, `integral2`, `integral3`, `quad`, `quadgk`)**: `trapz` and `cumtrapz` work for discrete arrays, but adaptive continuous quadrature over function handles `@(x) ...` is unavailable.
3. **Optimization & Nonlinear Root Solving (`fzero`, `fminsearch`, `fminbnd`, `fmincon`, `fsolve`)**: Unconstrained Nelder-Mead simplex and 1D root bracketing are missing from stdlib.
4. **Vector Calculus (`divergence`, `curl`, `del2`, `jacobian`, `hessian`)**: Only 1D/2D `gradient` exists; multi-dimensional vector operators are absent.
5. **3D and Advanced Plotting (`plot3`, `mesh`, `contour`, `quiver`)**: While 2D line, scatter, bar, histogram, and 2D surface (`surf`) exist with deterministic SVG rendering, 3D line plots (`plot3`), vector arrows (`quiver`), and contour isolines (`contour`/`contourf`) are not registered.

---

## 4. Complete Function Verification Table

| Category | Function | Runtime Status | Source Impl | Registered | Test Oracle / Specification | Raw Evidence Log |
| :--- | :--- | :--- | :---: | :---: | :--- | :--- |
| Calculus | `cumtrapz` | `EXISTS-WORKS` | Yes | Yes | Cumulative trapezoidal numerical integration | [`docs/verify-logs/functions/cumtrapz.log`](docs/verify-logs/functions/cumtrapz.log) |
| Calculus | `curl` | `MISSING` | No | No | 2D vorticity curl([-y, x]) = d/dx(x) - d/dy(-y) = 2 | [`docs/verify-logs/functions/curl.log`](docs/verify-logs/functions/curl.log) |
| Calculus | `del2` | `MISSING` | No | No | del2 = (d2/dx2 + d2/dy2)/4 = (2 + 2)/4 = 1 | [`docs/verify-logs/functions/del2.log`](docs/verify-logs/functions/del2.log) |
| Calculus | `divergence` | `MISSING` | No | No | div([x,y]) = d/dx(x) + d/dy(y) = 1 + 1 = 2 | [`docs/verify-logs/functions/divergence.log`](docs/verify-logs/functions/divergence.log) |
| Calculus | `gradient` | `EXISTS-WORKS` | Yes | Yes | 1D central differences with one-sided endpoints | [`docs/verify-logs/functions/gradient.log`](docs/verify-logs/functions/gradient.log) |
| Calculus | `hessian` | `MISSING` | No | No | Hessian matrix of scalar function evaluated at [1; 1] | [`docs/verify-logs/functions/hessian.log`](docs/verify-logs/functions/hessian.log) |
| Calculus | `integral` | `MISSING` | No | No | Definite integral of x^2 on [0, 1] equals 1/3 | [`docs/verify-logs/functions/integral.log`](docs/verify-logs/functions/integral.log) |
| Calculus | `integral2` | `MISSING` | No | No | Double integral of x*y over [0,1]x[0,1] equals 1/4 | [`docs/verify-logs/functions/integral2.log`](docs/verify-logs/functions/integral2.log) |
| Calculus | `integral3` | `MISSING` | No | No | Triple integral of x*y*z over unit cube equals 1/8 | [`docs/verify-logs/functions/integral3.log`](docs/verify-logs/functions/integral3.log) |
| Calculus | `jacobian` | `MISSING` | No | No | Jacobian matrix of vector function evaluated at [1; 2] | [`docs/verify-logs/functions/jacobian.log`](docs/verify-logs/functions/jacobian.log) |
| Calculus | `quad` | `MISSING` | No | No | Adaptive Simpson quadrature on [0, 1] | [`docs/verify-logs/functions/quad.log`](docs/verify-logs/functions/quad.log) |
| Calculus | `quadgk` | `MISSING` | No | No | Gauss-Kronrod quadrature on [0, 1] | [`docs/verify-logs/functions/quadgk.log`](docs/verify-logs/functions/quadgk.log) |
| Calculus | `trapz` | `EXISTS-WORKS` | Yes | Yes | Trapezoidal numerical integration over 5 points | [`docs/verify-logs/functions/trapz.log`](docs/verify-logs/functions/trapz.log) |
| Linear Algebra | `chol` | `EXISTS-WORKS` | Yes | Yes | Cholesky factorization reconstruction R' * R - A == 0 | [`docs/verify-logs/functions/chol.log`](docs/verify-logs/functions/chol.log) |
| Linear Algebra | `cond` | `EXISTS-WORKS` | Yes | Yes | 2-norm condition number 10 / 2 = 5 | [`docs/verify-logs/functions/cond.log`](docs/verify-logs/functions/cond.log) |
| Linear Algebra | `det` | `EXISTS-WORKS` | Yes | Yes | det([4,7; 2,6]) = 4*6 - 7*2 = 10 | [`docs/verify-logs/functions/det.log`](docs/verify-logs/functions/det.log) |
| Linear Algebra | `eig` | `EXISTS-WORKS` | Yes | Yes | Eigenvalues of diagonal matrix [2, 0; 0, 5] | [`docs/verify-logs/functions/eig.log`](docs/verify-logs/functions/eig.log) |
| Linear Algebra | `expm` | `EXISTS-WORKS` | Yes | Yes | Matrix exponential of zero matrix is identity | [`docs/verify-logs/functions/expm.log`](docs/verify-logs/functions/expm.log) |
| Linear Algebra | `hess` | `MISSING` | No | No | Hessenberg form sub-diagonal zero structure | [`docs/verify-logs/functions/hess.log`](docs/verify-logs/functions/hess.log) |
| Linear Algebra | `inv` | `EXISTS-WORKS` | Yes | Yes | A * inv(A) equals 2x2 identity matrix within 1e-12 | [`docs/verify-logs/functions/inv.log`](docs/verify-logs/functions/inv.log) |
| Linear Algebra | `kron` | `EXISTS-WORKS` | Yes | Yes | Kronecker tensor product | [`docs/verify-logs/functions/kron.log`](docs/verify-logs/functions/kron.log) |
| Linear Algebra | `lu` | `EXISTS-WORKS` | Yes | Yes | LU factorization reconstruction residual P' * L * U - A == 0 | [`docs/verify-logs/functions/lu.log`](docs/verify-logs/functions/lu.log) |
| Linear Algebra | `mldivide` | `EXISTS-WORKS` | Yes | Yes | Solve 2x2 linear system via matrix left division | [`docs/verify-logs/functions/mldivide.log`](docs/verify-logs/functions/mldivide.log) |
| Linear Algebra | `mrdivide` | `EXISTS-WORKS` | Yes | Yes | Solve linear system via matrix right division | [`docs/verify-logs/functions/mrdivide.log`](docs/verify-logs/functions/mrdivide.log) |
| Linear Algebra | `norm` | `EXISTS-WORKS` | Yes | Yes | Euclidean 2-norm of [3, 4] is 5 | [`docs/verify-logs/functions/norm.log`](docs/verify-logs/functions/norm.log) |
| Linear Algebra | `null` | `EXISTS-WORKS` | Yes | Yes | Orthonormal basis for null space: norm(A * Z) == 0 | [`docs/verify-logs/functions/null.log`](docs/verify-logs/functions/null.log) |
| Linear Algebra | `orth` | `EXISTS-WORKS` | Yes | Yes | Orthonormal basis for range of matrix | [`docs/verify-logs/functions/orth.log`](docs/verify-logs/functions/orth.log) |
| Linear Algebra | `pinv` | `EXISTS-WORKS` | Yes | Yes | Moore-Penrose pseudoinverse of diagonal matrix | [`docs/verify-logs/functions/pinv.log`](docs/verify-logs/functions/pinv.log) |
| Linear Algebra | `qr` | `EXISTS-WORKS` | No | Yes | QR factorization reconstruction residual Q * R - A == 0 | [`docs/verify-logs/functions/qr.log`](docs/verify-logs/functions/qr.log) |
| Linear Algebra | `rank` | `EXISTS-WORKS` | Yes | Yes | Matrix rank of rank-deficient 2x2 matrix | [`docs/verify-logs/functions/rank.log`](docs/verify-logs/functions/rank.log) |
| Linear Algebra | `schur` | `MISSING` | No | No | Schur decomposition reconstruction residual U*T*U' - A == 0 | [`docs/verify-logs/functions/schur.log`](docs/verify-logs/functions/schur.log) |
| Linear Algebra | `svd` | `EXISTS-WORKS` | Yes | Yes | Singular values in descending order [4; 3] | [`docs/verify-logs/functions/svd.log`](docs/verify-logs/functions/svd.log) |
| Linear Algebra | `trace` | `EXISTS-WORKS` | Yes | Yes | trace([4,7; 2,6]) = 4 + 6 = 10 | [`docs/verify-logs/functions/trace.log`](docs/verify-logs/functions/trace.log) |
| ODE & Simulation | `ode23` | `MISSING` | No | No | Bogacki-Shampine 3(2) solver y' = -y, y(0)=1 => y(1)=exp(-1) | [`docs/verify-logs/functions/ode23.log`](docs/verify-logs/functions/ode23.log) |
| ODE & Simulation | `ode45` | `MISSING` | No | No | Dormand-Prince 5(4) solver y' = -y, y(0)=1 => y(1)=exp(-1) | [`docs/verify-logs/functions/ode45.log`](docs/verify-logs/functions/ode45.log) |
| ODE & Simulation | `odeset` | `MISSING` | No | No | Create/alter ODE options structure | [`docs/verify-logs/functions/odeset.log`](docs/verify-logs/functions/odeset.log) |
| ODE & Simulation | `step` | `EXISTS-WORKS` | Yes | Yes | Step response of first-order plant 1/(s+1) | [`docs/verify-logs/functions/step.log`](docs/verify-logs/functions/step.log) |
| ODE & Simulation | `tf` | `EXISTS-WORKS` | Yes | Yes | Create continuous linear time-invariant transfer function | [`docs/verify-logs/functions/tf.log`](docs/verify-logs/functions/tf.log) |
| Signal Processing | `blackman` | `EXISTS-WORKS` | Yes | Yes | Blackman window vector of length 4 | [`docs/verify-logs/functions/blackman.log`](docs/verify-logs/functions/blackman.log) |
| Signal Processing | `conv` | `EXISTS-WORKS` | Yes | Yes | 1D discrete polynomial / signal convolution | [`docs/verify-logs/functions/conv.log`](docs/verify-logs/functions/conv.log) |
| Signal Processing | `deconv` | `MISSING` | No | No | Deconvolution and polynomial division | [`docs/verify-logs/functions/deconv.log`](docs/verify-logs/functions/deconv.log) |
| Signal Processing | `fft` | `EXISTS-WORKS` | Yes | Yes | 1D FFT of constant sequence [1, 1, 1, 1] | [`docs/verify-logs/functions/fft.log`](docs/verify-logs/functions/fft.log) |
| Signal Processing | `fft2` | `EXISTS-WORKS` | Yes | Yes | 2D FFT of 2x2 ones matrix | [`docs/verify-logs/functions/fft2.log`](docs/verify-logs/functions/fft2.log) |
| Signal Processing | `fftshift` | `EXISTS-WORKS` | Yes | Yes | Shift zero-frequency component to center of spectrum | [`docs/verify-logs/functions/fftshift.log`](docs/verify-logs/functions/fftshift.log) |
| Signal Processing | `filter` | `EXISTS-WORKS` | Yes | Yes | 1D direct-form II transposed digital filter | [`docs/verify-logs/functions/filter.log`](docs/verify-logs/functions/filter.log) |
| Signal Processing | `filtfilt` | `EXISTS-WORKS` | Yes | Yes | Zero-phase forward and reverse digital filtering | [`docs/verify-logs/functions/filtfilt.log`](docs/verify-logs/functions/filtfilt.log) |
| Signal Processing | `freqz` | `MISSING` | No | No | Frequency response of digital filter | [`docs/verify-logs/functions/freqz.log`](docs/verify-logs/functions/freqz.log) |
| Signal Processing | `hamming` | `EXISTS-WORKS` | Yes | Yes | Hamming window vector of length 4 | [`docs/verify-logs/functions/hamming.log`](docs/verify-logs/functions/hamming.log) |
| Signal Processing | `hann` | `EXISTS-WORKS` | Yes | Yes | Hann (Hanning) window vector of length 4 | [`docs/verify-logs/functions/hann.log`](docs/verify-logs/functions/hann.log) |
| Signal Processing | `ifft` | `EXISTS-WORKS` | Yes | Yes | 1D Inverse FFT of spectrum [4, 0, 0, 0] | [`docs/verify-logs/functions/ifft.log`](docs/verify-logs/functions/ifft.log) |
| Signal Processing | `ifft2` | `EXISTS-WORKS` | Yes | Yes | 2D Inverse FFT of spectrum | [`docs/verify-logs/functions/ifft2.log`](docs/verify-logs/functions/ifft2.log) |
| Signal Processing | `pwelch` | `MISSING` | No | No | Welch's averaged modified periodogram spectral estimate | [`docs/verify-logs/functions/pwelch.log`](docs/verify-logs/functions/pwelch.log) |
| Signal Processing | `residue` | `MISSING` | No | No | Partial fraction expansion (residues and poles) | [`docs/verify-logs/functions/residue.log`](docs/verify-logs/functions/residue.log) |
| Signal Processing | `spectrogram` | `MISSING` | No | No | Short-time Fourier transform spectrogram | [`docs/verify-logs/functions/spectrogram.log`](docs/verify-logs/functions/spectrogram.log) |
| Signal Processing | `xcorr` | `EXISTS-WORKS` | Yes | Yes | Cross-correlation / autocorrelation of vector [1, 2] | [`docs/verify-logs/functions/xcorr.log`](docs/verify-logs/functions/xcorr.log) |
| Interpolation & Optimization | `fminbnd` | `MISSING` | No | No | Golden section search / parabolic interpolation on [0, 4] | [`docs/verify-logs/functions/fminbnd.log`](docs/verify-logs/functions/fminbnd.log) |
| Interpolation & Optimization | `fmincon` | `MISSING` | No | No | Constrained nonlinear multivariable optimization | [`docs/verify-logs/functions/fmincon.log`](docs/verify-logs/functions/fmincon.log) |
| Interpolation & Optimization | `fminsearch` | `MISSING` | No | No | Nelder-Mead simplex unconstrained multivariable optimization | [`docs/verify-logs/functions/fminsearch.log`](docs/verify-logs/functions/fminsearch.log) |
| Interpolation & Optimization | `fminunc` | `MISSING` | No | No | Quasi-Newton BFGS unconstrained minimization | [`docs/verify-logs/functions/fminunc.log`](docs/verify-logs/functions/fminunc.log) |
| Interpolation & Optimization | `fsolve` | `MISSING` | No | No | System of nonlinear equations solver | [`docs/verify-logs/functions/fsolve.log`](docs/verify-logs/functions/fsolve.log) |
| Interpolation & Optimization | `linprog` | `MISSING` | No | No | Linear programming simplex or interior-point solver | [`docs/verify-logs/functions/linprog.log`](docs/verify-logs/functions/linprog.log) |
| Interpolation & Optimization | `pchip` | `MISSING` | No | No | Piecewise Cubic Hermite Interpolating Polynomial | [`docs/verify-logs/functions/pchip.log`](docs/verify-logs/functions/pchip.log) |
| Interpolation & Optimization | `quadprog` | `MISSING` | No | No | Quadratic programming solver | [`docs/verify-logs/functions/quadprog.log`](docs/verify-logs/functions/quadprog.log) |
| Interpolation & Optimization | `spline` | `MISSING` | No | No | Cubic spline data interpolation | [`docs/verify-logs/functions/spline.log`](docs/verify-logs/functions/spline.log) |
| Plotting & Visualization | `area` | `MISSING` | No | No | Filled area 2D plot | [`docs/verify-logs/plotting/area.log`](docs/verify-logs/plotting/area.log) |
| Plotting & Visualization | `axis` | `MISSING` | Yes | No | Set axis limits and scaling properties | [`docs/verify-logs/plotting/axis.log`](docs/verify-logs/plotting/axis.log) |
| Plotting & Visualization | `bar` | `EXISTS-WORKS` | Yes | Yes | Vertical bar graph | [`docs/verify-logs/plotting/bar.log`](docs/verify-logs/plotting/bar.log) |
| Plotting & Visualization | `barh` | `MISSING` | No | No | Horizontal bar graph | [`docs/verify-logs/plotting/barh.log`](docs/verify-logs/plotting/barh.log) |
| Plotting & Visualization | `colorbar` | `MISSING` | Yes | No | Display colorbar indicating color scale mapping | [`docs/verify-logs/plotting/colorbar.log`](docs/verify-logs/plotting/colorbar.log) |
| Plotting & Visualization | `colormap` | `MISSING` | Yes | No | Set color lookup table for current figure | [`docs/verify-logs/plotting/colormap.log`](docs/verify-logs/plotting/colormap.log) |
| Plotting & Visualization | `contour` | `MISSING` | No | No | 2D contour isolines plot | [`docs/verify-logs/plotting/contour.log`](docs/verify-logs/plotting/contour.log) |
| Plotting & Visualization | `contourf` | `MISSING` | No | No | Filled 2D contour plot | [`docs/verify-logs/plotting/contourf.log`](docs/verify-logs/plotting/contourf.log) |
| Plotting & Visualization | `figure` | `EXISTS-WORKS` | Yes | Yes | Create new or activate existing figure window | [`docs/verify-logs/plotting/figure.log`](docs/verify-logs/plotting/figure.log) |
| Plotting & Visualization | `grid` | `EXISTS-WORKS` | Yes | Yes | Display grid lines on axes | [`docs/verify-logs/plotting/grid.log`](docs/verify-logs/plotting/grid.log) |
| Plotting & Visualization | `heatmap` | `MISSING` | No | No | Heatmap chart visualization | [`docs/verify-logs/plotting/heatmap.log`](docs/verify-logs/plotting/heatmap.log) |
| Plotting & Visualization | `hist` | `EXISTS-WORKS` | Yes | Yes | Histogram plot | [`docs/verify-logs/plotting/hist.log`](docs/verify-logs/plotting/hist.log) |
| Plotting & Visualization | `histogram` | `MISSING` | No | No | Modern histogram plot object | [`docs/verify-logs/plotting/histogram.log`](docs/verify-logs/plotting/histogram.log) |
| Plotting & Visualization | `hold` | `EXISTS-WORKS` | Yes | Yes | Retain current plot when adding new plots | [`docs/verify-logs/plotting/hold.log`](docs/verify-logs/plotting/hold.log) |
| Plotting & Visualization | `imagesc` | `MISSING` | No | No | Scaled color image display | [`docs/verify-logs/plotting/imagesc.log`](docs/verify-logs/plotting/imagesc.log) |
| Plotting & Visualization | `legend` | `EXISTS-WORKS` | Yes | Yes | Add legend with valid location ('northeast') to axes | [`docs/verify-logs/plotting/legend.log`](docs/verify-logs/plotting/legend.log) |
| Plotting & Visualization | `loglog` | `MISSING` | No | No | Log-log scale plot | [`docs/verify-logs/plotting/loglog.log`](docs/verify-logs/plotting/loglog.log) |
| Plotting & Visualization | `mesh` | `MISSING` | No | No | 3D wireframe mesh plot | [`docs/verify-logs/plotting/mesh.log`](docs/verify-logs/plotting/mesh.log) |
| Plotting & Visualization | `meshc` | `MISSING` | No | No | 3D wireframe mesh with contour plot underneath | [`docs/verify-logs/plotting/meshc.log`](docs/verify-logs/plotting/meshc.log) |
| Plotting & Visualization | `meshz` | `MISSING` | No | No | 3D wireframe mesh with curtain / reference plane | [`docs/verify-logs/plotting/meshz.log`](docs/verify-logs/plotting/meshz.log) |
| Plotting & Visualization | `pie` | `MISSING` | No | No | Pie chart | [`docs/verify-logs/plotting/pie.log`](docs/verify-logs/plotting/pie.log) |
| Plotting & Visualization | `plot` | `EXISTS-WORKS` | Yes | Yes | 2D line plot with saveas SVG rendering verification | [`docs/verify-logs/plotting/plot.log`](docs/verify-logs/plotting/plot.log) |
| Plotting & Visualization | `plot3` | `MISSING` | No | No | 3D line plot with camera projection and depth sorting | [`docs/verify-logs/plotting/plot3.log`](docs/verify-logs/plotting/plot3.log) |
| Plotting & Visualization | `polarplot` | `MISSING` | No | No | Polar coordinate line plot | [`docs/verify-logs/plotting/polarplot.log`](docs/verify-logs/plotting/polarplot.log) |
| Plotting & Visualization | `quiver` | `MISSING` | No | No | Velocity / vector field quiver plot | [`docs/verify-logs/plotting/quiver.log`](docs/verify-logs/plotting/quiver.log) |
| Plotting & Visualization | `saveas` | `EXISTS-WORKS` | Yes | Yes | Save figure to disk in specified format (SVG) | [`docs/verify-logs/plotting/saveas.log`](docs/verify-logs/plotting/saveas.log) |
| Plotting & Visualization | `scatter` | `EXISTS-WORKS` | Yes | Yes | 2D scatter plot with circular markers | [`docs/verify-logs/plotting/scatter.log`](docs/verify-logs/plotting/scatter.log) |
| Plotting & Visualization | `scatter3` | `MISSING` | No | No | 3D scatter plot | [`docs/verify-logs/plotting/scatter3.log`](docs/verify-logs/plotting/scatter3.log) |
| Plotting & Visualization | `semilogx` | `MISSING` | No | No | Semi-logarithmic line plot with log x-axis | [`docs/verify-logs/plotting/semilogx.log`](docs/verify-logs/plotting/semilogx.log) |
| Plotting & Visualization | `semilogy` | `MISSING` | No | No | Semi-logarithmic line plot with log y-axis | [`docs/verify-logs/plotting/semilogy.log`](docs/verify-logs/plotting/semilogy.log) |
| Plotting & Visualization | `shading` | `MISSING` | Yes | No | Set color shading properties of surface and patch objects | [`docs/verify-logs/plotting/shading.log`](docs/verify-logs/plotting/shading.log) |
| Plotting & Visualization | `stairs` | `MISSING` | No | No | Stairstep plot | [`docs/verify-logs/plotting/stairs.log`](docs/verify-logs/plotting/stairs.log) |
| Plotting & Visualization | `stem` | `MISSING` | No | No | Discrete sequence / stem plot | [`docs/verify-logs/plotting/stem.log`](docs/verify-logs/plotting/stem.log) |
| Plotting & Visualization | `subplot` | `EXISTS-WORKS` | Yes | Yes | Grid arrangement of multiple subplot axes in single figure | [`docs/verify-logs/plotting/subplot.log`](docs/verify-logs/plotting/subplot.log) |
| Plotting & Visualization | `surf` | `EXISTS-WORKS` | Yes | Yes | 3D shaded surface plot | [`docs/verify-logs/plotting/surf.log`](docs/verify-logs/plotting/surf.log) |
| Plotting & Visualization | `title` | `EXISTS-WORKS` | Yes | Yes | Add title to current axes | [`docs/verify-logs/plotting/title.log`](docs/verify-logs/plotting/title.log) |
| Plotting & Visualization | `view` | `MISSING` | Yes | No | Set 3D camera line of sight (azimuth, elevation) | [`docs/verify-logs/plotting/view.log`](docs/verify-logs/plotting/view.log) |
| Plotting & Visualization | `waterfall` | `MISSING` | No | No | Waterfall mesh plot | [`docs/verify-logs/plotting/waterfall.log`](docs/verify-logs/plotting/waterfall.log) |
| Plotting & Visualization | `xlabel` | `EXISTS-WORKS` | Yes | Yes | Label x-axis | [`docs/verify-logs/plotting/xlabel.log`](docs/verify-logs/plotting/xlabel.log) |
| Plotting & Visualization | `xlim` | `EXISTS-WORKS` | Yes | Yes | Set x-axis limits | [`docs/verify-logs/plotting/xlim.log`](docs/verify-logs/plotting/xlim.log) |
| Plotting & Visualization | `ylabel` | `EXISTS-WORKS` | Yes | Yes | Label y-axis | [`docs/verify-logs/plotting/ylabel.log`](docs/verify-logs/plotting/ylabel.log) |
| Plotting & Visualization | `ylim` | `EXISTS-WORKS` | Yes | Yes | Set y-axis limits | [`docs/verify-logs/plotting/ylim.log`](docs/verify-logs/plotting/ylim.log) |
| Plotting & Visualization | `zlabel` | `EXISTS-WORKS` | Yes | Yes | Label z-axis | [`docs/verify-logs/plotting/zlabel.log`](docs/verify-logs/plotting/zlabel.log) |

---

## 5. Verification Provenance

All test logs are signed by cryptographic hash in [`docs/verify-logs/LOGS_SHA256.md`](../verify-logs/LOGS_SHA256.md).
The environment parameters and execution timestamps are recorded in [`docs/verify-logs/metadata.md`](../verify-logs/metadata.md).
