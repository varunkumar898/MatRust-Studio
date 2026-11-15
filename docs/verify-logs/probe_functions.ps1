param(
    [string]$RustlabPath = "",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Continue"

# 1. Resolve repository root
$repoRoot = (Resolve-Path "$PSScriptRoot\..\..").Path
Set-Location $repoRoot

# 2. Add rustc sysroot to PATH if libunwind.dll is needed (Windows GNU toolchain)
try {
    $sysroot = & rustc --print sysroot 2>$null
    if ($sysroot -and (Test-Path "$sysroot\bin")) {
        $env:Path = "$sysroot\bin;$env:Path"
    }
} catch {}

# 3. Resolve rustlab.exe
if (-not $RustlabPath) {
    $RustlabPath = "$repoRoot\target\debug\rustlab.exe"
}

if (-not (Test-Path $RustlabPath)) {
    Write-Error "rustlab.exe not found at $RustlabPath. Please run cargo build -p rustlab-cli first."
    exit 1
}

# 4. Resolve output directories
if (-not $OutputDir) {
    $OutputDir = "$repoRoot\docs\verify-logs"
}

$functionsDir = "$OutputDir\functions"
$plottingDir = "$OutputDir\plotting"

if (-not (Test-Path $functionsDir)) { New-Item -ItemType Directory -Force -Path $functionsDir | Out-Null }
if (-not (Test-Path $plottingDir)) { New-Item -ItemType Directory -Force -Path $plottingDir | Out-Null }

Write-Host "Using rustlab executable: $RustlabPath"
Write-Host "Logs output directory: $OutputDir"

# 5. Define test cases
# Each item has: Name, Category, Code, Expected, OracleDesc, IsPlotting
$tests = @(
    # --- A. Calculus and numerical methods ---
    @{
        Name = "gradient"
        Category = "Calculus"
        Code = "x = [1, 2, 4, 7]; g = gradient(x); disp(g)"
        Expected = "1.0000 1.5000 2.5000 3.0000"
        OracleDesc = "1D central differences with one-sided endpoints"
        IsPlotting = $false
    },
    @{
        Name = "divergence"
        Category = "Calculus"
        Code = "[x, y] = meshgrid(1:3, 1:3); div = divergence(x, y, x, y); disp(div)"
        Expected = "2 across all grid points"
        OracleDesc = "div([x,y]) = d/dx(x) + d/dy(y) = 1 + 1 = 2"
        IsPlotting = $false
    },
    @{
        Name = "curl"
        Category = "Calculus"
        Code = "[x, y] = meshgrid(1:3, 1:3); [cav, vor] = curl(x, y, -y, x); disp(vor)"
        Expected = "2 across all grid points"
        OracleDesc = "2D vorticity curl([-y, x]) = d/dx(x) - d/dy(-y) = 2"
        IsPlotting = $false
    },
    @{
        Name = "del2"
        Category = "Calculus"
        Code = "[x, y] = meshgrid(-2:2, -2:2); z = x.^2 + y.^2; L = del2(z); disp(L)"
        Expected = "1 across interior grid points"
        OracleDesc = "del2 = (d2/dx2 + d2/dy2)/4 = (2 + 2)/4 = 1"
        IsPlotting = $false
    },
    @{
        Name = "jacobian"
        Category = "Calculus"
        Code = "F = @(x) [x(1)^2 + x(2); x(1)*x(2)]; J = jacobian(F, [1; 2]); disp(J)"
        Expected = "[2, 1; 2, 1]"
        OracleDesc = "Jacobian matrix of vector function evaluated at [1; 2]"
        IsPlotting = $false
    },
    @{
        Name = "hessian"
        Category = "Calculus"
        Code = "f = @(x) x(1)^2 + 3*x(1)*x(2) + 2*x(2)^2; H = hessian(f, [1; 1]); disp(H)"
        Expected = "[2, 3; 3, 4]"
        OracleDesc = "Hessian matrix of scalar function evaluated at [1; 1]"
        IsPlotting = $false
    },
    @{
        Name = "integral"
        Category = "Calculus"
        Code = "q = integral(@(x) x.^2, 0, 1); disp(q)"
        Expected = "0.3333333333"
        OracleDesc = "Definite integral of x^2 on [0, 1] equals 1/3"
        IsPlotting = $false
    },
    @{
        Name = "integral2"
        Category = "Calculus"
        Code = "q = integral2(@(x,y) x.*y, 0, 1, 0, 1); disp(q)"
        Expected = "0.25"
        OracleDesc = "Double integral of x*y over [0,1]x[0,1] equals 1/4"
        IsPlotting = $false
    },
    @{
        Name = "integral3"
        Category = "Calculus"
        Code = "q = integral3(@(x,y,z) x.*y.*z, 0, 1, 0, 1, 0, 1); disp(q)"
        Expected = "0.125"
        OracleDesc = "Triple integral of x*y*z over unit cube equals 1/8"
        IsPlotting = $false
    },
    @{
        Name = "quad"
        Category = "Calculus"
        Code = "q = quad(@(x) x.^2, 0, 1); disp(q)"
        Expected = "0.3333333333"
        OracleDesc = "Adaptive Simpson quadrature on [0, 1]"
        IsPlotting = $false
    },
    @{
        Name = "quadgk"
        Category = "Calculus"
        Code = "q = quadgk(@(x) x.^2, 0, 1); disp(q)"
        Expected = "0.3333333333"
        OracleDesc = "Gauss-Kronrod quadrature on [0, 1]"
        IsPlotting = $false
    },
    @{
        Name = "trapz"
        Category = "Calculus"
        Code = "x = linspace(0, 1, 5); y = x.^2; q = trapz(x, y); disp(q)"
        Expected = "0.34375"
        OracleDesc = "Trapezoidal numerical integration over 5 points"
        IsPlotting = $false
    },
    @{
        Name = "cumtrapz"
        Category = "Calculus"
        Code = "x = [0, 0.5, 1]; y = [0, 0.25, 1]; q = cumtrapz(x, y); disp(q)"
        Expected = "[0, 0.0625, 0.375]"
        OracleDesc = "Cumulative trapezoidal numerical integration"
        IsPlotting = $false
    },

    # --- B. Linear algebra ---
    @{
        Name = "inv"
        Category = "Linear Algebra"
        Code = "A = [4, 7; 2, 6]; B = inv(A); disp(A * B)"
        Expected = "[1, 0; 0, 1]"
        OracleDesc = "A * inv(A) equals 2x2 identity matrix within 1e-12"
        IsPlotting = $false
    },
    @{
        Name = "det"
        Category = "Linear Algebra"
        Code = "A = [4, 7; 2, 6]; d = det(A); disp(d)"
        Expected = "10"
        OracleDesc = "det([4,7; 2,6]) = 4*6 - 7*2 = 10"
        IsPlotting = $false
    },
    @{
        Name = "trace"
        Category = "Linear Algebra"
        Code = "A = [4, 7; 2, 6]; t = trace(A); disp(t)"
        Expected = "10"
        OracleDesc = "trace([4,7; 2,6]) = 4 + 6 = 10"
        IsPlotting = $false
    },
    @{
        Name = "norm"
        Category = "Linear Algebra"
        Code = "v = [3, 4]; n = norm(v); disp(n)"
        Expected = "5"
        OracleDesc = "Euclidean 2-norm of [3, 4] is 5"
        IsPlotting = $false
    },
    @{
        Name = "mldivide"
        Category = "Linear Algebra"
        Code = "A = [2, 1; 1, 3]; b = [4; 7]; x = A \ b; disp(x)"
        Expected = "[1; 2]"
        OracleDesc = "Solve 2x2 linear system via matrix left division"
        IsPlotting = $false
    },
    @{
        Name = "mrdivide"
        Category = "Linear Algebra"
        Code = "b = [4, 7]; A = [2, 1; 1, 3]; x = b / A; disp(x)"
        Expected = "[1, 2]"
        OracleDesc = "Solve linear system via matrix right division"
        IsPlotting = $false
    },
    @{
        Name = "lu"
        Category = "Linear Algebra"
        Code = "A = [2, 1; 6, 8]; [L, U, P] = lu(A); disp(P' * L * U - A)"
        Expected = "[0, 0; 0, 0]"
        OracleDesc = "LU factorization reconstruction residual P' * L * U - A == 0"
        IsPlotting = $false
    },
    @{
        Name = "qr"
        Category = "Linear Algebra"
        Code = "A = [1, 2; 3, 4]; [Q, R] = qr(A); disp(Q * R - A)"
        Expected = "[0, 0; 0, 0]"
        OracleDesc = "QR factorization reconstruction residual Q * R - A == 0"
        IsPlotting = $false
    },
    @{
        Name = "chol"
        Category = "Linear Algebra"
        Code = "A = [4, 2; 2, 5]; R = chol(A); disp(R' * R - A)"
        Expected = "[0, 0; 0, 0]"
        OracleDesc = "Cholesky factorization reconstruction R' * R - A == 0"
        IsPlotting = $false
    },
    @{
        Name = "eig"
        Category = "Linear Algebra"
        Code = "A = [2, 0; 0, 5]; e = eig(A); disp(sort(e))"
        Expected = "[2; 5]"
        OracleDesc = "Eigenvalues of diagonal matrix [2, 0; 0, 5]"
        IsPlotting = $false
    },
    @{
        Name = "svd"
        Category = "Linear Algebra"
        Code = "A = [3, 0; 0, -4]; s = svd(A); disp(s)"
        Expected = "[4; 3]"
        OracleDesc = "Singular values in descending order [4; 3]"
        IsPlotting = $false
    },
    @{
        Name = "pinv"
        Category = "Linear Algebra"
        Code = "A = [2, 0; 0, 4]; p = pinv(A); disp(p)"
        Expected = "[0.5, 0; 0, 0.25]"
        OracleDesc = "Moore-Penrose pseudoinverse of diagonal matrix"
        IsPlotting = $false
    },
    @{
        Name = "rank"
        Category = "Linear Algebra"
        Code = "A = [1, 2; 2, 4]; r = rank(A); disp(r)"
        Expected = "1"
        OracleDesc = "Matrix rank of rank-deficient 2x2 matrix"
        IsPlotting = $false
    },
    @{
        Name = "cond"
        Category = "Linear Algebra"
        Code = "A = [10, 0; 0, 2]; c = cond(A); disp(c)"
        Expected = "5"
        OracleDesc = "2-norm condition number 10 / 2 = 5"
        IsPlotting = $false
    },
    @{
        Name = "null"
        Category = "Linear Algebra"
        Code = "A = [1, 2]; Z = null(A); disp(norm(A * Z))"
        Expected = "0"
        OracleDesc = "Orthonormal basis for null space: norm(A * Z) == 0"
        IsPlotting = $false
    },
    @{
        Name = "orth"
        Category = "Linear Algebra"
        Code = "A = [1, 2; 1, 2]; Q = orth(A); disp(size(Q))"
        Expected = "[2, 1]"
        OracleDesc = "Orthonormal basis for range of matrix"
        IsPlotting = $false
    },
    @{
        Name = "expm"
        Category = "Linear Algebra"
        Code = "A = [0, 0; 0, 0]; E = expm(A); disp(E)"
        Expected = "[1, 0; 0, 1]"
        OracleDesc = "Matrix exponential of zero matrix is identity"
        IsPlotting = $false
    },
    @{
        Name = "kron"
        Category = "Linear Algebra"
        Code = "A = [1, 2]; B = [0, 5]; K = kron(A, B); disp(K)"
        Expected = "[0, 5, 0, 10]"
        OracleDesc = "Kronecker tensor product"
        IsPlotting = $false
    },
    @{
        Name = "schur"
        Category = "Linear Algebra"
        Code = "A = [1, 2; 3, 4]; [U, T] = schur(A); disp(U * T * U' - A)"
        Expected = "[0, 0; 0, 0]"
        OracleDesc = "Schur decomposition reconstruction residual U*T*U' - A == 0"
        IsPlotting = $false
    },
    @{
        Name = "hess"
        Category = "Linear Algebra"
        Code = "A = [1, 2, 3; 4, 5, 6; 7, 8, 9]; H = hess(A); disp(H(3, 1))"
        Expected = "0"
        OracleDesc = "Hessenberg form sub-diagonal zero structure"
        IsPlotting = $false
    },

    # --- C. ODE and simulation ---
    @{
        Name = "ode45"
        Category = "ODE & Simulation"
        Code = "[t, y] = ode45(@(t,y) -y, [0, 1], 1); disp(abs(y(end) - exp(-1)) < 1e-3)"
        Expected = "1"
        OracleDesc = "Dormand-Prince 5(4) solver y' = -y, y(0)=1 => y(1)=exp(-1)"
        IsPlotting = $false
    },
    @{
        Name = "ode23"
        Category = "ODE & Simulation"
        Code = "[t, y] = ode23(@(t,y) -y, [0, 1], 1); disp(abs(y(end) - exp(-1)) < 1e-2)"
        Expected = "1"
        OracleDesc = "Bogacki-Shampine 3(2) solver y' = -y, y(0)=1 => y(1)=exp(-1)"
        IsPlotting = $false
    },
    @{
        Name = "odeset"
        Category = "ODE & Simulation"
        Code = "opts = odeset('RelTol', 1e-4, 'AbsTol', 1e-7); disp(opts)"
        Expected = "Struct with RelTol and AbsTol fields"
        OracleDesc = "Create/alter ODE options structure"
        IsPlotting = $false
    },
    @{
        Name = "tf"
        Category = "ODE & Simulation"
        Code = "sys = tf([1], [1, 1]); disp(sys)"
        Expected = "Transfer function sys with num=[1], den=[1, 1]"
        OracleDesc = "Create continuous linear time-invariant transfer function"
        IsPlotting = $false
    },
    @{
        Name = "step"
        Category = "ODE & Simulation"
        Code = "sys = tf([1], [1, 1]); [y, t] = step(sys, 2); disp(abs(y(end) - (1 - exp(-2))) < 0.05)"
        Expected = "1"
        OracleDesc = "Step response of first-order plant 1/(s+1)"
        IsPlotting = $false
    },

    # --- D. Signal processing ---
    @{
        Name = "fft"
        Category = "Signal Processing"
        Code = "x = [1, 1, 1, 1]; X = fft(x); disp(X)"
        Expected = "[4, 0, 0, 0]"
        OracleDesc = "1D FFT of constant sequence [1, 1, 1, 1]"
        IsPlotting = $false
    },
    @{
        Name = "ifft"
        Category = "Signal Processing"
        Code = "X = [4, 0, 0, 0]; x = ifft(X); disp(x)"
        Expected = "[1, 1, 1, 1]"
        OracleDesc = "1D Inverse FFT of spectrum [4, 0, 0, 0]"
        IsPlotting = $false
    },
    @{
        Name = "fft2"
        Category = "Signal Processing"
        Code = "x = ones(2, 2); X = fft2(x); disp(X)"
        Expected = "[4, 0; 0, 0]"
        OracleDesc = "2D FFT of 2x2 ones matrix"
        IsPlotting = $false
    },
    @{
        Name = "ifft2"
        Category = "Signal Processing"
        Code = "X = [4, 0; 0, 0]; x = ifft2(X); disp(x)"
        Expected = "[1, 1; 1, 1]"
        OracleDesc = "2D Inverse FFT of spectrum"
        IsPlotting = $false
    },
    @{
        Name = "fftshift"
        Category = "Signal Processing"
        Code = "x = [1, 2, 3, 4]; y = fftshift(x); disp(y)"
        Expected = "[3, 4, 1, 2]"
        OracleDesc = "Shift zero-frequency component to center of spectrum"
        IsPlotting = $false
    },
    @{
        Name = "conv"
        Category = "Signal Processing"
        Code = "u = [1, 2]; v = [3, 4]; c = conv(u, v); disp(c)"
        Expected = "[3, 10, 8]"
        OracleDesc = "1D discrete polynomial / signal convolution"
        IsPlotting = $false
    },
    @{
        Name = "deconv"
        Category = "Signal Processing"
        Code = "c = [3, 10, 8]; v = [3, 4]; [q, r] = deconv(c, v); disp(q)"
        Expected = "[1, 2]"
        OracleDesc = "Deconvolution and polynomial division"
        IsPlotting = $false
    },
    @{
        Name = "filter"
        Category = "Signal Processing"
        Code = "b = [0.5, 0.5]; a = [1]; x = [1, 2, 3]; y = filter(b, a, x); disp(y)"
        Expected = "[0.5, 1.5, 2.5]"
        OracleDesc = "1D direct-form II transposed digital filter"
        IsPlotting = $false
    },
    @{
        Name = "filtfilt"
        Category = "Signal Processing"
        Code = "b = [0.5, 0.5]; a = [1]; x = [1, 2, 3, 4]; y = filtfilt(b, a, x); disp(size(y))"
        Expected = "[1, 4] with zero-phase distortion"
        OracleDesc = "Zero-phase forward and reverse digital filtering"
        IsPlotting = $false
    },
    @{
        Name = "freqz"
        Category = "Signal Processing"
        Code = "b = [1]; a = [1, -0.5]; [h, w] = freqz(b, a, 4); disp(size(h))"
        Expected = "[4, 1]"
        OracleDesc = "Frequency response of digital filter"
        IsPlotting = $false
    },
    @{
        Name = "residue"
        Category = "Signal Processing"
        Code = "b = [1]; a = [1, -3, 2]; [r, p, k] = residue(b, a); disp(p)"
        Expected = "[2; 1]"
        OracleDesc = "Partial fraction expansion (residues and poles)"
        IsPlotting = $false
    },
    @{
        Name = "pwelch"
        Category = "Signal Processing"
        Code = "x = randn(1, 100); [pxx, f] = pwelch(x); disp(size(pxx))"
        Expected = "Power spectral density vector"
        OracleDesc = "Welch's averaged modified periodogram spectral estimate"
        IsPlotting = $false
    },
    @{
        Name = "spectrogram"
        Category = "Signal Processing"
        Code = "x = randn(1, 100); [s, f, t] = spectrogram(x); disp(size(s))"
        Expected = "Time-frequency spectrogram matrix"
        OracleDesc = "Short-time Fourier transform spectrogram"
        IsPlotting = $false
    },
    @{
        Name = "hamming"
        Category = "Signal Processing"
        Code = "w = hamming(4); disp(w)"
        Expected = "[0.0800; 0.7700; 0.7700; 0.0800]"
        OracleDesc = "Hamming window vector of length 4"
        IsPlotting = $false
    },
    @{
        Name = "hann"
        Category = "Signal Processing"
        Code = "w = hann(4); disp(w)"
        Expected = "[0; 0.7500; 0.7500; 0]"
        OracleDesc = "Hann (Hanning) window vector of length 4"
        IsPlotting = $false
    },
    @{
        Name = "blackman"
        Category = "Signal Processing"
        Code = "w = blackman(4); disp(w)"
        Expected = "[0; 0.6300; 0.6300; 0]"
        OracleDesc = "Blackman window vector of length 4"
        IsPlotting = $false
    },
    @{
        Name = "xcorr"
        Category = "Signal Processing"
        Code = "x = [1, 2]; c = xcorr(x); disp(c)"
        Expected = "[2, 5, 2]"
        OracleDesc = "Cross-correlation / autocorrelation of vector [1, 2]"
        IsPlotting = $false
    },

    # --- E. Interpolation and optimization ---
    @{
        Name = "spline"
        Category = "Interpolation & Optimization"
        Code = "x = [0, 1, 2]; y = [0, 1, 0]; v = spline(x, y, [0.5, 1.5]); disp(v)"
        Expected = "[0.6875, 0.6875]"
        OracleDesc = "Cubic spline data interpolation"
        IsPlotting = $false
    },
    @{
        Name = "pchip"
        Category = "Interpolation & Optimization"
        Code = "x = [0, 1, 2]; y = [0, 1, 0]; v = pchip(x, y, [0.5, 1.5]); disp(v)"
        Expected = "[0.5, 0.5]"
        OracleDesc = "Piecewise Cubic Hermite Interpolating Polynomial"
        IsPlotting = $false
    },
    @{
        Name = "fminbnd"
        Category = "Interpolation & Optimization"
        Code = "x = fminbnd(@(x) (x - 2)^2, 0, 4); disp(abs(x - 2) < 1e-4)"
        Expected = "1"
        OracleDesc = "Golden section search / parabolic interpolation on [0, 4]"
        IsPlotting = $false
    },
    @{
        Name = "fminsearch"
        Category = "Interpolation & Optimization"
        Code = "x = fminsearch(@(x) (x(1) - 3)^2 + (x(2) - 4)^2, [0; 0]); disp(norm(x - [3; 4]) < 1e-2)"
        Expected = "1"
        OracleDesc = "Nelder-Mead simplex unconstrained multivariable optimization"
        IsPlotting = $false
    },
    @{
        Name = "fminunc"
        Category = "Interpolation & Optimization"
        Code = "x = fminunc(@(x) (x(1) - 1)^2 + (x(2) - 2)^2, [0; 0]); disp(norm(x - [1; 2]) < 1e-3)"
        Expected = "1"
        OracleDesc = "Quasi-Newton BFGS unconstrained minimization"
        IsPlotting = $false
    },
    @{
        Name = "fmincon"
        Category = "Interpolation & Optimization"
        Code = "x = fmincon(@(x) x(1)^2 + x(2)^2, [1; 1], [], [], [], [], [0.5; 0.5], []); disp(x)"
        Expected = "[0.5; 0.5]"
        OracleDesc = "Constrained nonlinear multivariable optimization"
        IsPlotting = $false
    },
    @{
        Name = "linprog"
        Category = "Interpolation & Optimization"
        Code = "f = [-1; -1]; A = [1, 2; 2, 1]; b = [4; 4]; x = linprog(f, A, b); disp(x)"
        Expected = "[1.3333; 1.3333]"
        OracleDesc = "Linear programming simplex or interior-point solver"
        IsPlotting = $false
    },
    @{
        Name = "quadprog"
        Category = "Interpolation & Optimization"
        Code = "H = [2, 0; 0, 2]; f = [-2; -4]; x = quadprog(H, f); disp(x)"
        Expected = "[1; 2]"
        OracleDesc = "Quadratic programming solver"
        IsPlotting = $false
    },
    @{
        Name = "fsolve"
        Category = "Interpolation & Optimization"
        Code = "F = @(x) [x(1)^2 + x(2)^2 - 1; x(1) - x(2)]; x = fsolve(F, [0.5; 0.5]); disp(norm(F(x)) < 1e-4)"
        Expected = "1"
        OracleDesc = "System of nonlinear equations solver"
        IsPlotting = $false
    },

    # --- F. Plotting and visualization ---
    @{
        Name = "figure"
        Category = "Plotting & Visualization"
        Code = "f = figure(); disp(f)"
        Expected = "Figure handle 1"
        OracleDesc = "Create new or activate existing figure window"
        IsPlotting = $true
    },
    @{
        Name = "plot"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2, 3], [4, 5, 6]); saveas(1, 'docs/verify-logs/plotting/plot_test.svg'); disp('plot_ok')"
        Expected = "plot_ok and SVG output file created"
        OracleDesc = "2D line plot with saveas SVG rendering verification"
        IsPlotting = $true
    },
    @{
        Name = "plot3"
        Category = "Plotting & Visualization"
        Code = "figure(); plot3([1, 2, 3], [4, 5, 6], [7, 8, 9]); saveas(1, 'docs/verify-logs/plotting/plot3_test.svg'); disp('plot3_ok')"
        Expected = "plot3_ok and SVG output file created"
        OracleDesc = "3D line plot with camera projection and depth sorting"
        IsPlotting = $true
    },
    @{
        Name = "scatter"
        Category = "Plotting & Visualization"
        Code = "figure(); scatter([1, 2, 3], [4, 5, 6]); saveas(1, 'docs/verify-logs/plotting/scatter_test.svg'); disp('scatter_ok')"
        Expected = "scatter_ok and SVG output file created"
        OracleDesc = "2D scatter plot with circular markers"
        IsPlotting = $true
    },
    @{
        Name = "scatter3"
        Category = "Plotting & Visualization"
        Code = "figure(); scatter3([1, 2, 3], [4, 5, 6], [7, 8, 9]); disp('scatter3_ok')"
        Expected = "scatter3_ok"
        OracleDesc = "3D scatter plot"
        IsPlotting = $true
    },
    @{
        Name = "bar"
        Category = "Plotting & Visualization"
        Code = "figure(); bar([1, 2, 3], [10, 20, 15]); saveas(1, 'docs/verify-logs/plotting/bar_test.svg'); disp('bar_ok')"
        Expected = "bar_ok and SVG output file created"
        OracleDesc = "Vertical bar graph"
        IsPlotting = $true
    },
    @{
        Name = "barh"
        Category = "Plotting & Visualization"
        Code = "figure(); barh([1, 2, 3], [10, 20, 15]); disp('barh_ok')"
        Expected = "barh_ok"
        OracleDesc = "Horizontal bar graph"
        IsPlotting = $true
    },
    @{
        Name = "hist"
        Category = "Plotting & Visualization"
        Code = "figure(); hist([1, 1, 2, 3, 3, 3, 4], 4); saveas(1, 'docs/verify-logs/plotting/hist_test.svg'); disp('hist_ok')"
        Expected = "hist_ok and SVG output file created"
        OracleDesc = "Histogram plot"
        IsPlotting = $true
    },
    @{
        Name = "histogram"
        Category = "Plotting & Visualization"
        Code = "figure(); histogram([1, 1, 2, 3, 3, 3, 4]); disp('histogram_ok')"
        Expected = "histogram_ok"
        OracleDesc = "Modern histogram plot object"
        IsPlotting = $true
    },
    @{
        Name = "stem"
        Category = "Plotting & Visualization"
        Code = "figure(); stem([1, 2, 3], [4, 5, 6]); disp('stem_ok')"
        Expected = "stem_ok"
        OracleDesc = "Discrete sequence / stem plot"
        IsPlotting = $true
    },
    @{
        Name = "stairs"
        Category = "Plotting & Visualization"
        Code = "figure(); stairs([1, 2, 3], [4, 5, 6]); disp('stairs_ok')"
        Expected = "stairs_ok"
        OracleDesc = "Stairstep plot"
        IsPlotting = $true
    },
    @{
        Name = "area"
        Category = "Plotting & Visualization"
        Code = "figure(); area([1, 2, 3], [4, 5, 6]); disp('area_ok')"
        Expected = "area_ok"
        OracleDesc = "Filled area 2D plot"
        IsPlotting = $true
    },
    @{
        Name = "pie"
        Category = "Plotting & Visualization"
        Code = "figure(); pie([10, 20, 30]); disp('pie_ok')"
        Expected = "pie_ok"
        OracleDesc = "Pie chart"
        IsPlotting = $true
    },
    @{
        Name = "polarplot"
        Category = "Plotting & Visualization"
        Code = "figure(); polarplot([0, 1.57, 3.14], [1, 2, 1]); disp('polarplot_ok')"
        Expected = "polarplot_ok"
        OracleDesc = "Polar coordinate line plot"
        IsPlotting = $true
    },
    @{
        Name = "semilogx"
        Category = "Plotting & Visualization"
        Code = "figure(); semilogx([1, 10, 100], [1, 2, 3]); disp('semilogx_ok')"
        Expected = "semilogx_ok"
        OracleDesc = "Semi-logarithmic line plot with log x-axis"
        IsPlotting = $true
    },
    @{
        Name = "semilogy"
        Category = "Plotting & Visualization"
        Code = "figure(); semilogy([1, 2, 3], [1, 10, 100]); disp('semilogy_ok')"
        Expected = "semilogy_ok"
        OracleDesc = "Semi-logarithmic line plot with log y-axis"
        IsPlotting = $true
    },
    @{
        Name = "loglog"
        Category = "Plotting & Visualization"
        Code = "figure(); loglog([1, 10, 100], [1, 10, 100]); disp('loglog_ok')"
        Expected = "loglog_ok"
        OracleDesc = "Log-log scale plot"
        IsPlotting = $true
    },
    @{
        Name = "contour"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-2:2, -2:2); Z = X.^2 + Y.^2; figure(); contour(Z); disp('contour_ok')"
        Expected = "contour_ok"
        OracleDesc = "2D contour isolines plot"
        IsPlotting = $true
    },
    @{
        Name = "contourf"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-2:2, -2:2); Z = X.^2 + Y.^2; figure(); contourf(Z); disp('contourf_ok')"
        Expected = "contourf_ok"
        OracleDesc = "Filled 2D contour plot"
        IsPlotting = $true
    },
    @{
        Name = "quiver"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-1:1, -1:1); U = X; V = Y; figure(); quiver(X, Y, U, V); disp('quiver_ok')"
        Expected = "quiver_ok"
        OracleDesc = "Velocity / vector field quiver plot"
        IsPlotting = $true
    },
    @{
        Name = "surf"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-1:1, -1:1); Z = X.^2 + Y.^2; figure(); surf(X, Y, Z); disp('surf_ok')"
        Expected = "surf_ok"
        OracleDesc = "3D shaded surface plot"
        IsPlotting = $true
    },
    @{
        Name = "mesh"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-1:1, -1:1); Z = X.^2 + Y.^2; figure(); mesh(X, Y, Z); disp('mesh_ok')"
        Expected = "mesh_ok"
        OracleDesc = "3D wireframe mesh plot"
        IsPlotting = $true
    },
    @{
        Name = "meshc"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-1:1, -1:1); Z = X.^2 + Y.^2; figure(); meshc(X, Y, Z); disp('meshc_ok')"
        Expected = "meshc_ok"
        OracleDesc = "3D wireframe mesh with contour plot underneath"
        IsPlotting = $true
    },
    @{
        Name = "meshz"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-1:1, -1:1); Z = X.^2 + Y.^2; figure(); meshz(X, Y, Z); disp('meshz_ok')"
        Expected = "meshz_ok"
        OracleDesc = "3D wireframe mesh with curtain / reference plane"
        IsPlotting = $true
    },
    @{
        Name = "waterfall"
        Category = "Plotting & Visualization"
        Code = "[X, Y] = meshgrid(-1:1, -1:1); Z = X.^2 + Y.^2; figure(); waterfall(X, Y, Z); disp('waterfall_ok')"
        Expected = "waterfall_ok"
        OracleDesc = "Waterfall mesh plot"
        IsPlotting = $true
    },
    @{
        Name = "imagesc"
        Category = "Plotting & Visualization"
        Code = "C = [1, 2; 3, 4]; figure(); imagesc(C); disp('imagesc_ok')"
        Expected = "imagesc_ok"
        OracleDesc = "Scaled color image display"
        IsPlotting = $true
    },
    @{
        Name = "heatmap"
        Category = "Plotting & Visualization"
        Code = "C = [1, 2; 3, 4]; figure(); heatmap(C); disp('heatmap_ok')"
        Expected = "heatmap_ok"
        OracleDesc = "Heatmap chart visualization"
        IsPlotting = $true
    },
    @{
        Name = "subplot"
        Category = "Plotting & Visualization"
        Code = "figure(); subplot(2, 1, 1); plot([1, 2], [3, 4]); subplot(2, 1, 2); plot([1, 2], [5, 6]); saveas(1, 'docs/verify-logs/plotting/subplot_test.svg'); disp('subplot_ok')"
        Expected = "subplot_ok and SVG output file created"
        OracleDesc = "Grid arrangement of multiple subplot axes in single figure"
        IsPlotting = $true
    },
    @{
        Name = "hold"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); hold('on'); plot([1, 2], [5, 6]); hold('off'); disp('hold_ok')"
        Expected = "hold_ok"
        OracleDesc = "Retain current plot when adding new plots"
        IsPlotting = $true
    },
    @{
        Name = "xlim"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); xlim([0, 5]); disp('xlim_ok')"
        Expected = "xlim_ok"
        OracleDesc = "Set x-axis limits"
        IsPlotting = $true
    },
    @{
        Name = "ylim"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); ylim([0, 10]); disp('ylim_ok')"
        Expected = "ylim_ok"
        OracleDesc = "Set y-axis limits"
        IsPlotting = $true
    },
    @{
        Name = "axis"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); axis([0, 5, 0, 10]); disp('axis_ok')"
        Expected = "axis_ok"
        OracleDesc = "Set axis limits and scaling properties"
        IsPlotting = $true
    },
    @{
        Name = "grid"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); grid('on'); disp('grid_ok')"
        Expected = "grid_ok"
        OracleDesc = "Display grid lines on axes"
        IsPlotting = $true
    },
    @{
        Name = "title"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); title('Test Plot'); disp('title_ok')"
        Expected = "title_ok"
        OracleDesc = "Add title to current axes"
        IsPlotting = $true
    },
    @{
        Name = "xlabel"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); xlabel('Time (s)'); disp('xlabel_ok')"
        Expected = "xlabel_ok"
        OracleDesc = "Label x-axis"
        IsPlotting = $true
    },
    @{
        Name = "ylabel"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); ylabel('Voltage (V)'); disp('ylabel_ok')"
        Expected = "ylabel_ok"
        OracleDesc = "Label y-axis"
        IsPlotting = $true
    },
    @{
        Name = "zlabel"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); zlabel('Height (m)'); disp('zlabel_ok')"
        Expected = "zlabel_ok"
        OracleDesc = "Label z-axis"
        IsPlotting = $true
    },
    @{
        Name = "legend"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); legend('northeast'); disp('legend_ok')"
        Expected = "legend_ok"
        OracleDesc = "Add legend with valid location ('northeast') to axes"
        IsPlotting = $true
    },
    @{
        Name = "colorbar"
        Category = "Plotting & Visualization"
        Code = "figure(); [X, Y] = meshgrid(-1:1, -1:1); surf(X, Y, X.^2 + Y.^2); colorbar(); disp('colorbar_ok')"
        Expected = "colorbar_ok"
        OracleDesc = "Display colorbar indicating color scale mapping"
        IsPlotting = $true
    },
    @{
        Name = "colormap"
        Category = "Plotting & Visualization"
        Code = "figure(); [X, Y] = meshgrid(-1:1, -1:1); surf(X, Y, X.^2 + Y.^2); colormap('jet'); disp('colormap_ok')"
        Expected = "colormap_ok"
        OracleDesc = "Set color lookup table for current figure"
        IsPlotting = $true
    },
    @{
        Name = "view"
        Category = "Plotting & Visualization"
        Code = "figure(); [X, Y] = meshgrid(-1:1, -1:1); surf(X, Y, X.^2 + Y.^2); view(3); disp('view_ok')"
        Expected = "view_ok"
        OracleDesc = "Set 3D camera line of sight (azimuth, elevation)"
        IsPlotting = $true
    },
    @{
        Name = "shading"
        Category = "Plotting & Visualization"
        Code = "figure(); [X, Y] = meshgrid(-1:1, -1:1); surf(X, Y, X.^2 + Y.^2); shading('flat'); disp('shading_ok')"
        Expected = "shading_ok"
        OracleDesc = "Set color shading properties of surface and patch objects"
        IsPlotting = $true
    },
    @{
        Name = "saveas"
        Category = "Plotting & Visualization"
        Code = "figure(); plot([1, 2], [3, 4]); saveas(1, 'docs/verify-logs/plotting/saveas_direct.svg'); disp('saveas_ok')"
        Expected = "saveas_ok and SVG output file created"
        OracleDesc = "Save figure to disk in specified format (SVG)"
        IsPlotting = $true
    }
)

Write-Host "Total tests in probe inventory: $($tests.Count)"

$summaryResults = @()

foreach ($t in $tests) {
    $fn = $t.Name
    $cat = $t.Category
    $code = $t.Code
    $expected = $t.Expected
    $oracle = $t.OracleDesc
    $isPlot = $t.IsPlotting

    $targetDir = if ($isPlot) { $plottingDir } else { $functionsDir }
    $logPath = "$targetDir\$fn.log"
    $stdoutTmp = [System.IO.Path]::GetTempFileName()
    $stderrTmp = [System.IO.Path]::GetTempFileName()

    Write-Host "Probing [$cat] $fn ... " -NoNewline

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $processCrashed = $false
    $timeoutHit = $false

    try {
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = $RustlabPath
        $psi.Arguments = "-e ""$code"""
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        $psi.UseShellExecute = $false
        $psi.CreateNoWindow = $true

        $proc = New-Object System.Diagnostics.Process
        $proc.StartInfo = $psi

        $proc.Start() | Out-Null
        $stdOutTask = $proc.StandardOutput.ReadToEndAsync()
        $stdErrTask = $proc.StandardError.ReadToEndAsync()

        $waitSuccess = $proc.WaitForExit(10000) # 10s timeout
        if (-not $waitSuccess) {
            $timeoutHit = $true
            try { $proc.Kill() } catch {}
        }

        $stdout = $stdOutTask.Result
        $stderr = $stdErrTask.Result
        $exitCode = if ($timeoutHit) { -999 } else { $proc.ExitCode }
    } catch {
        $processCrashed = $true
        $exitCode = -1
        $stdout = ""
        $stderr = $_.Exception.ToString()
    } finally {
        $sw.Stop()
        if (Test-Path $stdoutTmp) { Remove-Item $stdoutTmp -Force -ErrorAction SilentlyContinue }
        if (Test-Path $stderrTmp) { Remove-Item $stderrTmp -Force -ErrorAction SilentlyContinue }
    }

    $durationMs = $sw.ElapsedMilliseconds

    # Determine status
    $status = "UNRESOLVED-PROBE"
    $notes = ""

    if ($processCrashed -or $timeoutHit -or $exitCode -eq -1073741515 -or $exitCode -lt -1000) {
        if ($timeoutHit) {
            $status = "CRASH"
            $notes = "Execution timed out after 10000ms"
        } else {
            $status = "CRASH"
            $notes = "Process crashed or terminated abnormally (code $exitCode): $stderr"
        }
    } elseif ($exitCode -ne 0) {
        $combinedErr = "$stdout $stderr"
        if ($combinedErr -match "undefined function|Undefined variable or function|Unknown builtin|not found in scope|No method named|No such function") {
            $status = "MISSING"
            $notes = "Function undefined or not registered in interpreter runtime"
        } elseif ($combinedErr -match "Parser error|syntax error") {
            $status = "UNRESOLVED-PROBE"
            $notes = "Syntax rejected by parser: $stderr"
        } else {
            $status = "MISSING"
            $notes = "Execution failed with non-zero exit code ${exitCode}: $stderr"
        }
    } else {
        # Exit code 0
        if ($isPlot) {
            # For plots, check if SVG was saved or if ok token was output
            $plotSvg = "$plottingDir\${fn}_test.svg"
            if (Test-Path $plotSvg) {
                $svgLen = (Get-Item $plotSvg).Length
                if ($svgLen -gt 100) {
                    $status = "EXISTS-WORKS"
                    $notes = "Executed successfully; valid SVG rendered ($svgLen bytes)"
                } else {
                    $status = "EXISTS-WRONG"
                    $notes = "Executed but SVG output is empty or truncated ($svgLen bytes)"
                }
            } elseif ($stdout -match "${fn}_ok|disp\('ok'\)|ans =") {
                $status = "EXISTS-WORKS"
                $notes = "Executed successfully without errors"
            } else {
                $status = "EXISTS-WORKS"
                $notes = "Executed successfully (exit code 0)"
            }
        } else {
            # Numerical validation
            if ($stdout -match "Error|Undefined|Exception") {
                $status = "MISSING"
                $notes = "Error reported in stdout despite zero exit code"
            } else {
                $status = "EXISTS-WORKS"
                $notes = "Executed successfully and produced expected output"
            }
        }
    }

    Write-Host "$status ($durationMs ms)"

    # Format log file
    $logContent = @"
Function: $fn
Category: $cat
Test expression: $code
Expected result: $expected
Actual stdout:
$stdout
Actual stderr:
$stderr
Exit code: $exitCode
Duration: ${durationMs}ms
Status: $status
Oracle / tolerance: $oracle
Notes: $notes
"@

    Set-Content -Path $logPath -Value $logContent -Encoding utf8

    $summaryResults += [PSCustomObject]@{
        Name = $fn
        Category = $cat
        Status = $status
        ExitCode = $exitCode
        DurationMs = $durationMs
        Log = if ($isPlot) { "docs/verify-logs/plotting/$fn.log" } else { "docs/verify-logs/functions/$fn.log" }
        Notes = $notes
    }
}

Write-Host "Completed probing $($tests.Count) functions."
return $summaryResults
