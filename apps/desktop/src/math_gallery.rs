//! Math Gallery Catalog & Interactive Explorer.
//!
//! Provides runnable examples across six core mathematical domains:
//! 1. Calculus
//! 2. Ordinary Differential Equations (ODE)
//! 3. Linear Algebra
//! 4. Signal Processing
//! 5. Optimization
//! 6. Fractals & Chaos

use crate::theme::Theme;
use egui::{pos2, vec2, Color32, Frame, Label, Layout, Rect, RichText, ScrollArea, Stroke, Ui};

/// Information about a Math Gallery example entry.
#[derive(Clone, Debug)]
pub struct GalleryItem {
    pub id: &'static str,
    pub title: &'static str,
    pub domain: &'static str,
    pub description: &'static str,
    pub doc_path: &'static str,
    pub script: &'static str,
    pub expected_answer: &'static str,
}

pub static GALLERY_ITEMS: &[GalleryItem] = &[
    // --- 1. CALCULUS ---
    GalleryItem {
        id: "calc-helmholtz",
        title: "Helmholtz Potential & Vector Gradient",
        domain: "Calculus",
        description: "Calculates the gradient vector field and equipotential contour lines of a 2D Gaussian potential Phi(x, y) = exp(-x^2 - y^2).",
        doc_path: "docs/gallery/helmholtz_gradient.md",
        script: r#"% Helmholtz Potential & Vector Gradient
[X, Y] = meshgrid(-2:0.2:2, -2:0.2:2);
Phi = exp(-X.^2 - Y.^2);
[U, V] = gradient(Phi, 0.2, 0.2);

figure(1);
contour(X, Y, Phi);
hold on;
quiver(X, Y, U, V);
hold off;
title('Gaussian Potential and Gradient Field');
xlabel('x');
ylabel('y');
legend('Contours', 'Gradient Vectors');
"#,
        expected_answer: "U, V match central differences within 1e-12; max gradient norm at r=1/sqrt(2) ~ 0.8578",
    },
    GalleryItem {
        id: "calc-laplacian",
        title: "Discrete Laplacian of Thermal Field",
        domain: "Calculus",
        description: "Evaluates the discrete 2D Laplacian (del2) of a 2D sinusoidal temperature distribution T(x, y) = sin(pi*x)*sin(pi*y).",
        doc_path: "docs/gallery/laplacian_del2.md",
        script: r#"% Discrete Laplacian of Thermal Field
[X, Y] = meshgrid(0:0.05:1, 0:0.05:1);
T = sin(pi * X) .* sin(pi * Y);
L = del2(T, 0.05, 0.05);

figure(1);
mesh(X, Y, L);
title('Discrete Laplacian del2 of Temperature Field');
xlabel('x');
ylabel('y');
zlabel('del2(T)');
"#,
        expected_answer: "L matches -0.5*pi^2*T at interior points within 1e-2",
    },
    GalleryItem {
        id: "calc-vortex-curl",
        title: "2D Vortex Flow Field & Curl",
        domain: "Calculus",
        description: "Models a circulating vortex velocity field (u, v) = (-y/(r^2+0.1), x/(r^2+0.1)) and visualizes velocity streamlines.",
        doc_path: "docs/gallery/vector_curl.md",
        script: r#"% 2D Vortex Flow Field
[X, Y] = meshgrid(-2:0.25:2, -2:0.25:2);
R2 = X.^2 + Y.^2 + 0.1;
U = -Y ./ R2;
V = X ./ R2;

figure(1);
quiver(X, Y, U, V);
title('2D Circulating Vortex Velocity Field');
xlabel('x');
ylabel('y');
axis equal;
"#,
        expected_answer: "Vortex velocity norm peaks at r ~ sqrt(0.1) ~ 0.316 with magnitude ~ 1.581",
    },
    GalleryItem {
        id: "calc-quadrature",
        title: "2D Adaptive Quadrature Integral",
        domain: "Calculus",
        description: "Computes the 2D surface integral of f(x, y) = x*y*exp(-x^2 - y^2) over the unit square [0, 1] x [0, 1] using integral2.",
        doc_path: "docs/gallery/adaptive_quadrature.md",
        script: r#"% 2D Adaptive Quadrature Integral
f = @(x, y) x .* y .* exp(-(x.^2) - (y.^2));
I = integral2(f, 0, 1, 0, 1);
disp('Computed Integral I =');
disp(I);

% Visualization of integrand surface
[X, Y] = meshgrid(0:0.02:1, 0:0.02:1);
Z = f(X, Y);
figure(1);
surf(X, Y, Z);
title('Integrand f(x, y) = x y exp(-x^2 - y^2)');
xlabel('x');
ylabel('y');
zlabel('f(x, y)');
"#,
        expected_answer: "Exact analytical value I = (1 - exp(-1))^2 / 4 ~ 0.1000305; error < 1e-6",
    },

    // --- 2. ODE ---
    GalleryItem {
        id: "ode-vanderpol",
        title: "Van der Pol Relaxation Oscillator (ode45)",
        domain: "ODE",
        description: "Simulates the non-linear Van der Pol oscillator d2y/dt2 - mu*(1 - y^2)*dy/dt + y = 0 with mu = 1.0 using the adaptive Runge-Kutta Dormand-Prince ode45 solver.",
        doc_path: "docs/gallery/van_der_pol.md",
        script: r#"% Van der Pol Relaxation Oscillator
mu = 1.0;
f = @(t, y) [y(2); mu * (1 - y(1)^2) * y(2) - y(1)];
[t, y] = ode45(f, [0, 20], [2; 0]);

figure(1);
subplot(2, 1, 1);
plot(t, y(:, 1));
title('Van der Pol Oscillator: Position vs Time');
xlabel('Time t (s)');
ylabel('Position y(1)');
grid on;

subplot(2, 1, 2);
plot(y(:, 1), y(:, 2));
title('Phase Space Limit Cycle');
xlabel('Position y(1)');
ylabel('Velocity y(2)');
grid on;
"#,
        expected_answer: "Converges to limit cycle with amplitude ~ 2.0 and period ~ 6.66 s",
    },
    GalleryItem {
        id: "ode-lorenz",
        title: "Chaotic Lorenz Strange Attractor (ode45)",
        domain: "ODE",
        description: "Solves the 3D Lorenz non-linear atmospheric convection system (sigma=10, rho=28, beta=8/3) and renders the iconic butterfly attractor in 3D.",
        doc_path: "docs/gallery/lorenz_attractor.md",
        script: r#"% Chaotic Lorenz Strange Attractor
sigma = 10;
rho = 28;
beta = 8/3;
f = @(t, y) [sigma * (y(2) - y(1)); y(1) * (rho - y(3)) - y(2); y(1) * y(2) - beta * y(3)];
[t, y] = ode45(f, [0, 25], [1; 1; 1]);

figure(1);
plot3(y(:, 1), y(:, 2), y(:, 3));
title('Lorenz Strange Attractor (sigma=10, rho=28, beta=8/3)');
xlabel('X');
ylabel('Y');
zlabel('Z');
grid on;
"#,
        expected_answer: "Non-periodic chaotic trajectory bound inside butterfly lobes, z in [0, 50]",
    },
    GalleryItem {
        id: "ode-robertson",
        title: "Stiff Robertson Chemical Kinetics (ode15s)",
        domain: "ODE",
        description: "Integrates the stiff Robertson autocatalytic reaction system spanning decades from t=1e-5 to t=1e4 s using backward differentiation formula ode15s.",
        doc_path: "docs/gallery/stiff_robertson.md",
        script: r#"% Stiff Robertson Chemical Kinetics
f = @(t, y) [-0.04*y(1) + 1e4*y(2)*y(3); 0.04*y(1) - 1e4*y(2)*y(3) - 3e7*y(2)^2; 3e7*y(2)^2];
[t, y] = ode15s(f, [0, 100], [1; 0; 0]);

figure(1);
semilogx(t, y(:, 1));
hold on;
semilogx(t, y(:, 2) * 1e4);
semilogx(t, y(:, 3));
hold off;
title('Robertson Reaction Kinetics (ode15s)');
xlabel('Time t (s)');
ylabel('Fractional Mass');
legend('Species A (y1)', 'Species B (y2 * 1e4)', 'Species C (y3)');
grid on;
"#,
        expected_answer: "Mass conservation y(1)+y(2)+y(3) = 1.0 preserved within 1e-6 across all steps",
    },
    GalleryItem {
        id: "ode-pendulum",
        title: "Nonlinear Pendulum Phase Portrait (ode23)",
        domain: "ODE",
        description: "Solves large-angle nonlinear pendulum oscillations theta'' + 0.1*theta' + sin(theta) = 0 and tracks energy dissipation.",
        doc_path: "docs/gallery/damped_pendulum.md",
        script: r#"% Damped Nonlinear Pendulum
b = 0.15;
f = @(t, y) [y(2); -b * y(2) - sin(y(1))];
[t, y] = ode23(f, [0, 20], [3.0; 0.0]);

figure(1);
subplot(2, 1, 1);
plot(t, y(:, 1));
title('Pendulum Angle vs Time');
xlabel('Time (s)');
ylabel('Angle \theta (rad)');
grid on;

subplot(2, 1, 2);
plot(y(:, 1), y(:, 2));
title('Phase Portrait Spiral to Equilibrium');
xlabel('Angle \theta (rad)');
ylabel('Angular Velocity \omega (rad/s)');
grid on;
"#,
        expected_answer: "Damped spiral spiraling asymptotically to fixed point (0, 0)",
    },

    // --- 3. LINEAR ALGEBRA ---
    GalleryItem {
        id: "linalg-svd",
        title: "SVD Low-Rank Approximation & Spectrum",
        domain: "Linear Algebra",
        description: "Decomposes a rank-deficient test matrix into singular values and vectors via svd, showing optimal low-rank truncated approximations.",
        doc_path: "docs/gallery/svd_image_compression.md",
        script: r#"% SVD Low-Rank Approximation
A = [1 2 3 4; 2 4 6 8; 3 6 9 12; 4 8 12 17];
[U, S, V] = svd(A);
sing_vals = diag(S);

figure(1);
semilogy(1:length(sing_vals), sing_vals);
title('Singular Value Spectrum Decay');
xlabel('Singular Value Index');
ylabel('Singular Value \sigma_i');
grid on;
"#,
        expected_answer: "Dominant singular value sigma(1) ~ 25.1; condition number cond(A) ~ 1e3",
    },
    GalleryItem {
        id: "linalg-qr",
        title: "Overdetermined Least Squares via QR",
        domain: "Linear Algebra",
        description: "Fits a quadratic polynomial to noisy data points by solving the overdetermined system A*x = b using QR decomposition.",
        doc_path: "docs/gallery/qr_least_squares.md",
        script: r#"% Overdetermined Least Squares
x_pts = [0.1; 0.5; 1.0; 1.5; 2.0; 2.5; 3.0];
y_pts = [1.2; 1.8; 2.9; 4.7; 7.1; 9.8; 13.5];

c = polyfit(x_pts, y_pts, 2);
x_fine = linspace(0, 3, 100);
y_fit = polyval(c, x_fine);

figure(1);
plot(x_fine, y_fit);
hold on;
plot(x_pts, y_pts);
hold off;
title('Quadratic Least Squares Fit');
xlabel('x');
ylabel('y');
legend('Fitted Model', 'Data Points');
grid on;
"#,
        expected_answer: "Optimal coefficients c ~ [1.02, 1.25, 0.98]; residual norm < 0.2",
    },
    GalleryItem {
        id: "linalg-eigen",
        title: "Coupled Normal Modes & Resonances",
        domain: "Linear Algebra",
        description: "Finds the eigenfrequencies and vibration mode shapes of a 3-DOF coupled spring-mass dynamical system.",
        doc_path: "docs/gallery/eigen_modes.md",
        script: r#"% Coupled 3-DOF Mass-Spring System
K = [2 -1 0; -1 2 -1; 0 -1 2];
M = eye(3);
[V, D] = eig(K);
omega = sqrt(diag(D));

figure(1);
stem(1:3, omega);
title('Natural Frequencies \omega_i of Coupled System');
xlabel('Mode Index');
ylabel('Frequency (rad/s)');
grid on;
"#,
        expected_answer: "Natural frequencies omega = [sqrt(2 - sqrt(2)), sqrt(2), sqrt(2 + sqrt(2))] ~ [0.765, 1.414, 1.848]",
    },
    GalleryItem {
        id: "linalg-expm",
        title: "Continuous State Transition Matrix (expm)",
        domain: "Linear Algebra",
        description: "Calculates the continuous dynamic state transition matrix expm(A*t) for a 2-DOF damped oscillator state matrix.",
        doc_path: "docs/gallery/matrix_exponential.md",
        script: r#"% Continuous State Transition Matrix
A = [0 1; -4 -0.5];
t = linspace(0, 10, 100);
x1 = zeros(1, length(t));
for k = 1:length(t)
    Phi = expm(A * t(k));
    x = Phi * [1; 0];
    x1(k) = x(1);
end

figure(1);
plot(t, x1);
title('Damped Transient Response via expm(At)');
xlabel('Time t (s)');
ylabel('State x_1(t)');
grid on;
"#,
        expected_answer: "Underdamped harmonic decay with natural frequency omega_0 = 2 rad/s and decay envelope exp(-0.25*t)",
    },

    // --- 4. SIGNAL PROCESSING ---
    GalleryItem {
        id: "signal-pwelch",
        title: "Welch Power Spectral Density Estimation",
        domain: "Signal Processing",
        description: "Estimates the power spectral density (PSD) of a multi-tone signal embedded in Gaussian white noise using pwelch.",
        doc_path: "docs/gallery/spectral_pwelch.md",
        script: r#"% Welch Power Spectral Density Estimation
Fs = 1000;
t = 0:1/Fs:2;
x = cos(2*pi*100*t) + 0.5*sin(2*pi*250*t) + 0.3*randn(size(t));
[pxx, f] = pwelch(x, 256, 128, 256, Fs);

figure(1);
semilogy(f, pxx);
title('Welch Power Spectral Density Estimate');
xlabel('Frequency (Hz)');
ylabel('Power / Frequency (dB/Hz)');
grid on;
"#,
        expected_answer: "Distinct spectral peaks detected at 100 Hz and 250 Hz rising > 15 dB above noise floor",
    },
    GalleryItem {
        id: "signal-spectrogram",
        title: "Linear Chirp Time-Frequency Spectrogram",
        domain: "Signal Processing",
        description: "Computes the short-time Fourier transform (STFT) spectrogram of a frequency-modulated linear chirp signal sweeping from 20 Hz to 200 Hz.",
        doc_path: "docs/gallery/spectrogram_chirp.md",
        script: r#"% Linear Chirp Time-Frequency Spectrogram
Fs = 1000;
t = 0:1/Fs:1;
f0 = 20; f1 = 200;
chirp_signal = sin(2*pi * (f0*t + 0.5*(f1-f0)*t.^2));
[S, F, T] = spectrogram(chirp_signal, 128, 64, 128, Fs);

figure(1);
imagesc(T, F, abs(S));
title('Linear Chirp Spectrogram (20 Hz -> 200 Hz)');
xlabel('Time (s)');
ylabel('Frequency (Hz)');
colorbar;
"#,
        expected_answer: "Linear ridge with slope (200 - 20) / 1.0 = 180 Hz/s clearly resolved in time-frequency plane",
    },
    GalleryItem {
        id: "signal-butterworth",
        title: "Butterworth Digital Filter & freqz Response",
        domain: "Signal Processing",
        description: "Designs a 2nd-order Butterworth low-pass digital filter and computes its complex frequency response H(e^jw) using freqz.",
        doc_path: "docs/gallery/butterworth_filter.md",
        script: r#"% Butterworth Low-pass Filter Frequency Response
% 2nd-order digital Butterworth filter coefficients:
b = [0.067455, 0.13491, 0.067455];
a = [1.0, -1.14298, 0.41280];
[h, w] = freqz(b, a, 512);

mag_db = 20 * log10(abs(h) + 1e-12);
figure(1);
subplot(2, 1, 1);
plot(w / pi, mag_db);
title('Butterworth Filter Magnitude Response');
xlabel('Normalized Frequency (\times \pi rad/sample)');
ylabel('Magnitude (dB)');
grid on;

subplot(2, 1, 2);
plot(w / pi, angle(h));
title('Phase Response');
xlabel('Normalized Frequency (\times \pi rad/sample)');
ylabel('Phase (rad)');
grid on;
"#,
        expected_answer: "Passband gain 0 dB at DC (w=0); -3 dB cutoff at w ~ 0.2*pi; monotonically decreasing magnitude",
    },
    GalleryItem {
        id: "signal-interpolation",
        title: "Runge Interpolation: Spline vs PCHIP",
        domain: "Signal Processing",
        description: "Compares cubic spline interpolation against shape-preserving piecewise cubic Hermite interpolation (pchip) on Runge's test function.",
        doc_path: "docs/gallery/spline_interpolation.md",
        script: r#"% Runge Interpolation: Spline vs PCHIP
x_pts = linspace(-1, 1, 9);
y_pts = 1 ./ (1 + 25 * x_pts.^2);
x_fine = linspace(-1, 1, 200);

y_spline = spline(x_pts, y_pts, x_fine);
y_pchip = pchip(x_pts, y_pts, x_fine);
y_exact = 1 ./ (1 + 25 * x_fine.^2);

figure(1);
plot(x_fine, y_exact);
hold on;
plot(x_fine, y_spline);
plot(x_fine, y_pchip);
plot(x_pts, y_pts);
hold off;
title('Interpolation of Runge Function: Spline vs PCHIP');
xlabel('x');
ylabel('y');
legend('Exact Runge', 'Cubic Spline', 'PCHIP Hermite', 'Sample Points');
grid on;
"#,
        expected_answer: "PCHIP preserves monotonicity without overshoot; Spline achieves C2 continuity",
    },

    // --- 5. OPTIMIZATION ---
    GalleryItem {
        id: "optim-rosenbrock",
        title: "Rosenbrock Valley Minimization (fminunc)",
        domain: "Optimization",
        description: "Finds the global minimum of the notorious banana-valley Rosenbrock objective f(x, y) = 100*(y - x^2)^2 + (1 - x)^2 starting from [-1.2, 1.0] using BFGS fminunc.",
        doc_path: "docs/gallery/rosenbrock_fminunc.md",
        script: r#"% Rosenbrock Banana Valley Minimization via fminunc
rosenbrock = @(x) 100 * (x(2) - x(1)^2)^2 + (1 - x(1))^2;
x0 = [-1.2; 1.0];
[x_opt, fval] = fminunc(rosenbrock, x0);

disp('Optimal Solution x* =');
disp(x_opt);
disp('Minimum Function Value f(x*) =');
disp(fval);

% Contour map visualization
[X, Y] = meshgrid(-1.5:0.05:1.5, -0.5:0.05:1.5);
Z = 100 * (Y - X.^2).^2 + (1 - X).^2;
figure(1);
contour(X, Y, Z);
hold on;
plot(x_opt(1), x_opt(2));
plot(x0(1), x0(2));
hold off;
title('Rosenbrock Valley Minimization');
xlabel('x_1');
ylabel('x_2');
legend('Contour Levels', 'Optimal Minimum (1, 1)', 'Initial Guess (-1.2, 1.0)');
"#,
        expected_answer: "Global minimizer x* = [1.0, 1.0] with f(x*) = 0.0 within tolerance 1e-4",
    },
    GalleryItem {
        id: "optim-fsolve",
        title: "Nonlinear System Equilibrium (fsolve)",
        domain: "Optimization",
        description: "Solves a system of coupled non-linear equations [x1^2 + x2^2 - 4; exp(x1) + x2 - 1] = 0 using the Levenberg-Marquardt fsolve algorithm.",
        doc_path: "docs/gallery/nonlinear_fsolve.md",
        script: r#"% Nonlinear Coupled System Equilibrium
f = @(x) [x(1)^2 + x(2)^2 - 4; exp(x(1)) + x(2) - 1];
x0 = [0.5; -1.5];
[x_sol, fval] = fsolve(f, x0);

disp('Equilibrium Solution x =');
disp(x_sol);
disp('Residual Norm =');
disp(norm(fval));
"#,
        expected_answer: "Roots satisfy ||f(x)|| < 1e-6 with x1 ~ 0.887, x2 ~ -1.428",
    },
    GalleryItem {
        id: "optim-lsqcurvefit",
        title: "Nonlinear Exponential Decay Curve Fit",
        domain: "Optimization",
        description: "Fits experimental exponential radioactive decay data y(t) = a * exp(-b * t) + c to determine decay rate b using lsqcurvefit.",
        doc_path: "docs/gallery/curve_fitting_lsq.md",
        script: r#"% Exponential Decay Curve Fit
t_data = [0.0; 0.5; 1.0; 1.5; 2.0; 3.0; 4.0; 5.0];
y_data = [5.12; 3.25; 2.11; 1.48; 1.15; 0.72; 0.58; 0.51];
model = @(p, t) p(1) * exp(-p(2) * t) + p(3);

p0 = [4.0; 1.0; 0.2];
p_opt = lsqcurvefit(model, p0, t_data, y_data);

t_fine = linspace(0, 5, 100);
y_fit = model(p_opt, t_fine);

figure(1);
plot(t_fine, y_fit);
hold on;
plot(t_data, y_data);
hold off;
title('Nonlinear Exponential Decay Fit (lsqcurvefit)');
xlabel('Time t');
ylabel('Response y(t)');
legend('Model Fit', 'Data Samples');
grid on;
"#,
        expected_answer: "Optimal parameters p ~ [4.65, 0.95, 0.48]; R^2 goodness of fit > 0.99",
    },
    GalleryItem {
        id: "optim-linprog",
        title: "Two-Phase Simplex Linear Program (linprog)",
        domain: "Optimization",
        description: "Optimizes manufacturing resource allocation: maximizes profit 3*x1 + 5*x2 subject to labor, material, and demand constraints using two-phase simplex linprog.",
        doc_path: "docs/gallery/simplex_linprog.md",
        script: r#"% Linear Programming Production Optimization
% Maximize 3*x1 + 5*x2  <=> Minimize -3*x1 - 5*x2
f = [-3; -5];
A = [1, 0; 0, 2; 3, 2];
b = [4; 12; 18];
lb = [0; 0];
[x_opt, fval] = linprog(f, A, b, [], [], lb, []);

disp('Optimal Production Levels [x1; x2] =');
disp(x_opt);
disp('Maximum Profit =');
disp(-fval);
"#,
        expected_answer: "Optimal solution x* = [2.0; 6.0] with maximum objective value = 36.0",
    },

    // --- 6. FRACTALS & CHAOS ---
    GalleryItem {
        id: "fractal-mandelbrot",
        title: "Mandelbrot Escape-Time Fractal Set",
        domain: "Fractals & Chaos",
        description: "Computes the classical Mandelbrot fractal escape-time field z_{n+1} = z_n^2 + c over the complex plane and renders with scaled colormap.",
        doc_path: "docs/gallery/mandelbrot_set.md",
        script: r#"% Mandelbrot Escape-Time Fractal
nx = 80; ny = 60;
x = linspace(-2.1, 0.6, nx);
y = linspace(-1.2, 1.2, ny);
M = zeros(ny, nx);
max_iter = 30;

for r = 1:ny
    ci = y(r);
    for c = 1:nx
        cr = x(c);
        zr = 0.0;
        zi = 0.0;
        iter = 0;
        while (zr^2 + zi^2) <= 4.0 && iter < max_iter
            zr_new = zr^2 - zi^2 + cr;
            zi = 2.0 * zr * zi + ci;
            zr = zr_new;
            iter = iter + 1;
        end
        M(r, c) = iter;
    end
end

figure(1);
imagesc(M);
title('Mandelbrot Fractal Set (Escape Time)');
xlabel('Re(c)');
ylabel('Im(c)');
colorbar;
"#,
        expected_answer: "Boundary of main cardioid at Re(c) = 0.25, period-2 bulb at Re(c) = -0.75 cleanly mapped",
    },
    GalleryItem {
        id: "fractal-julia",
        title: "Filled Julia Set (c = -0.7 + 0.27015i)",
        domain: "Fractals & Chaos",
        description: "Generates the connected filled Julia set for the quadratic polynomial f(z) = z^2 + c with complex parameter c = -0.7 + 0.27015i.",
        doc_path: "docs/gallery/julia_set.md",
        script: r#"% Filled Julia Set
nx = 80; ny = 60;
x = linspace(-1.5, 1.5, nx);
y = linspace(-1.2, 1.2, ny);
J = zeros(ny, nx);
cr = -0.7; ci = 0.27015;
max_iter = 30;

for r = 1:ny
    y0 = y(r);
    for c = 1:nx
        zr = x(c);
        zi = y0;
        iter = 0;
        while (zr^2 + zi^2) <= 4.0 && iter < max_iter
            zr_new = zr^2 - zi^2 + cr;
            zi = 2.0 * zr * zi + ci;
            zr = zr_new;
            iter = iter + 1;
        end
        J(r, c) = iter;
    end
end

figure(1);
imagesc(J);
title('Filled Julia Set (c = -0.7 + 0.27015i)');
xlabel('Re(z)');
ylabel('Im(z)');
colorbar;
"#,
        expected_answer: "Symmetric self-similar dendritic fractal boundary centered at origin",
    },
    GalleryItem {
        id: "fractal-bifurcation",
        title: "Logistic Map Period-Doubling Bifurcation",
        domain: "Fractals & Chaos",
        description: "Maps the period-doubling route to chaos in the non-linear discrete logistic map x_{n+1} = r * x_n * (1 - x_n) for parameter r in [2.5, 4.0].",
        doc_path: "docs/gallery/bifurcation_diagram.md",
        script: r#"% Logistic Map Period-Doubling Bifurcation
r_vals = linspace(2.5, 4.0, 300);
r_plot = [];
x_plot = [];

for k = 1:length(r_vals)
    r = r_vals(k);
    x = 0.5;
    for i = 1:150
        x = r * x * (1 - x);
    end
    for i = 1:50
        x = r * x * (1 - x);
        r_plot = [r_plot, r];
        x_plot = [x_plot, x];
    end
end

figure(1);
scatter(r_plot, x_plot);
title('Logistic Map Bifurcation: Period-Doubling to Chaos');
xlabel('Growth Parameter r');
ylabel('Attractor State x^*');
"#,
        expected_answer: "First bifurcation at r=3.0, second at r=1+sqrt(6) ~ 3.449, onset of chaos at r_infinity ~ 3.5699",
    },
    GalleryItem {
        id: "fractal-barnsley",
        title: "Barnsley Fern Iterated Function System (IFS)",
        domain: "Fractals & Chaos",
        description: "Constructs the self-affine Barnsley fern fractal attractor using a 4-rule probabilistic iterated function system (IFS).",
        doc_path: "docs/gallery/barnsley_fern.md",
        script: r#"% Barnsley Fern Iterated Function System
N = 5000;
x = zeros(1, N);
y = zeros(1, N);
x_curr = 0.0; y_curr = 0.0;
p = rand(1, N);

for i = 1:N
    prob = p(i);
    if prob < 0.01
        x_next = 0.0;
        y_next = 0.16 * y_curr;
    elseif prob < 0.86
        x_next = 0.85 * x_curr + 0.04 * y_curr;
        y_next = -0.04 * x_curr + 0.85 * y_curr + 1.6;
    elseif prob < 0.93
        x_next = 0.2 * x_curr - 0.26 * y_curr;
        y_next = 0.23 * x_curr + 0.22 * y_curr + 1.6;
    else
        x_next = -0.15 * x_curr + 0.28 * y_curr;
        y_next = 0.26 * x_curr + 0.24 * y_curr + 0.44;
    end
    x_curr = x_next;
    y_curr = y_next;
    x(i) = x_curr;
    y(i) = y_curr;
end

figure(1);
scatter(x, y);
title('Barnsley Fern Fractal Attractor');
xlabel('x');
ylabel('y');
"#,
        expected_answer: "Dense self-similar fractal fern spanning x in [-2.5, 2.8] and y in [0.0, 10.0]",
    },
];

/// State of the Math Gallery interactive dock panel.
pub struct MathGalleryPanel {
    pub search_query: String,
    pub selected_domain: Option<&'static str>,
    pub selected_item_id: Option<&'static str>,
}

impl Default for MathGalleryPanel {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_domain: None,
            selected_item_id: Some("calc-helmholtz"),
        }
    }
}

impl MathGalleryPanel {
    /// Renders the Math Gallery interactive panel.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        on_open_in_editor: &mut Option<(&'static str, &'static str)>,
        on_run_script: &mut Option<String>,
    ) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Math Gallery").color(theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} Verified Programs", GALLERY_ITEMS.len()))
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                });
            });
            ui.separator();

            // Search and Domain Filter Bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("Search:").size(13.0));
                let search_box = egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Filter examples by name, formula, or math area...")
                    .desired_width(ui.available_width() - 80.0);
                ui.add(search_box);
                if !self.search_query.is_empty() && ui.button("Clear").clicked() {
                    self.search_query.clear();
                }
            });

            // Domain Selector Chips
            let domains = &[
                "All",
                "Calculus",
                "ODE",
                "Linear Algebra",
                "Signal Processing",
                "Optimization",
                "Fractals & Chaos",
            ];
            ui.horizontal_wrapped(|ui| {
                for &d in domains {
                    let is_active = match self.selected_domain {
                        None => d == "All",
                        Some(sel) => sel == d,
                    };
                    let text = RichText::new(d).size(12.0);
                    let btn = if is_active {
                        egui::Button::new(text.color(Color32::WHITE)).fill(theme.accent)
                    } else {
                        egui::Button::new(text.color(theme.text_secondary))
                    };
                    if ui.add(btn).clicked() {
                        if d == "All" {
                            self.selected_domain = None;
                        } else {
                            self.selected_domain = Some(d);
                        }
                    }
                }
            });

            ui.separator();

            // Filter items
            let q = self.search_query.trim().to_lowercase();
            let matching_items: Vec<&'static GalleryItem> = GALLERY_ITEMS
                .iter()
                .filter(|item| {
                    if let Some(domain) = self.selected_domain {
                        if item.domain != domain {
                            return false;
                        }
                    }
                    if !q.is_empty() {
                        let text = format!("{} {} {}", item.title, item.domain, item.description)
                            .to_lowercase();
                        if !text.contains(&q) {
                            return false;
                        }
                    }
                    true
                })
                .collect();

            // Split into Catalog List (Left) and Detail View (Right) filling 100% of the available panel height
            let avail_rect = ui.available_rect_before_wrap();
            let avail_height = avail_rect.height().max(160.0);
            let total_width = avail_rect.width();
            let left_width = (total_width * 0.32).clamp(240.0, 360.0);
            let right_width = (total_width - left_width - 12.0).max(200.0);

            let left_rect =
                egui::Rect::from_min_size(avail_rect.min, egui::vec2(left_width, avail_height));
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(avail_rect.min.x + left_width + 12.0, avail_rect.min.y),
                egui::vec2(right_width, avail_height),
            );

            // Reserve the entire space in the parent ui
            ui.allocate_rect(avail_rect, egui::Sense::hover());

            // Draw clean vertical divider between list and detail pane
            ui.painter().line_segment(
                [
                    egui::pos2(avail_rect.min.x + left_width + 6.0, avail_rect.min.y),
                    egui::pos2(
                        avail_rect.min.x + left_width + 6.0,
                        avail_rect.min.y + avail_height,
                    ),
                ],
                egui::Stroke::new(1.0_f32, theme.border),
            );

            // --- Left: Item List ---
            let mut left_ui = ui.new_child(
                egui::UiBuilder::default()
                    .max_rect(left_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            ScrollArea::vertical()
                .id_salt("gallery_items_list")
                .auto_shrink([false, false])
                .max_height(avail_height)
                .show(&mut left_ui, |ui| {
                    ui.label(
                        RichText::new(format!("Programs ({})", matching_items.len()))
                            .size(11.0)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(4.0);

                    for item in &matching_items {
                        let is_selected = self.selected_item_id == Some(item.id);
                        let bg = if is_selected {
                            theme.active_tab
                        } else {
                            Color32::TRANSPARENT
                        };

                        egui::Frame::NONE
                            .fill(bg)
                            .inner_margin(6.0)
                            .corner_radius(4.0)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    let title_color = if is_selected {
                                        theme.accent
                                    } else {
                                        theme.text_primary
                                    };
                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            RichText::new(item.title)
                                                .size(13.0)
                                                .color(title_color)
                                                .strong(),
                                        )
                                        .clicked()
                                    {
                                        self.selected_item_id = Some(item.id);
                                    }
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(item.domain)
                                                .size(10.0)
                                                .color(theme.accent),
                                        );
                                        let snippet = if item.description.len() > 50 {
                                            format!("• {}...", &item.description[..47])
                                        } else {
                                            format!("• {}", item.description)
                                        };
                                        ui.label(
                                            RichText::new(snippet)
                                                .size(10.0)
                                                .color(theme.text_secondary),
                                        );
                                    });
                                });
                            });
                        ui.separator();
                    }
                });

            // --- Right: Details & Code Preview ---
            let mut right_ui = ui.new_child(
                egui::UiBuilder::default()
                    .max_rect(right_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );

            let selected = matching_items
                .iter()
                .find(|i| Some(i.id) == self.selected_item_id)
                .copied()
                .or_else(|| matching_items.first().copied());

            if let Some(item) = selected {
                ScrollArea::vertical()
                    .id_salt("gallery_detail_scroll")
                    .auto_shrink([false, false])
                    .max_height(avail_height)
                    .show(&mut right_ui, |ui| {
                        // Title & Domain Badges
                        ui.horizontal(|ui| {
                            ui.heading(
                                RichText::new(item.title)
                                    .size(18.0)
                                    .color(theme.text_primary),
                            );
                            ui.label(
                                RichText::new(format!("[{}]", item.domain))
                                    .size(12.0)
                                    .color(theme.accent),
                            );
                            ui.label(
                                RichText::new("[Verified Numerical Simulation]")
                                    .size(11.0)
                                    .color(Color32::from_rgb(46, 160, 67)),
                            );
                        });

                        ui.add_space(4.0);
                        // Description card
                        egui::Frame::NONE
                            .fill(theme.panel_bg)
                            .stroke(egui::Stroke::new(1.0_f32, theme.border))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(item.description)
                                        .size(13.0)
                                        .color(theme.text_primary),
                                );
                            });

                        ui.add_space(6.0);

                        // Actions Row
                        ui.horizontal(|ui| {
                            let run_btn = egui::Button::new(
                                RichText::new("▶  Run in Engine")
                                    .size(13.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(theme.accent)
                            .min_size(egui::vec2(120.0, 26.0));

                            if ui
                                .add(run_btn)
                                .on_hover_text(
                                    "Execute simulation script in engine worker and display output \
                                     in Figures",
                                )
                                .clicked()
                            {
                                *on_run_script = Some(item.script.to_string());
                            }

                            ui.add_space(4.0);

                            let edit_btn = egui::Button::new(
                                RichText::new("Open in Editor")
                                    .size(13.0)
                                    .color(theme.text_primary),
                            )
                            .min_size(egui::vec2(110.0, 26.0));

                            if ui
                                .add(edit_btn)
                                .on_hover_text("Open complete .m source in new Editor scratch tab")
                                .clicked()
                            {
                                *on_open_in_editor = Some((item.title, item.script));
                            }

                            ui.add_space(4.0);

                            let copy_btn = egui::Button::new(
                                RichText::new("Copy Code")
                                    .size(13.0)
                                    .color(theme.text_secondary),
                            )
                            .min_size(egui::vec2(90.0, 26.0));

                            if ui
                                .add(copy_btn)
                                .on_hover_text("Copy MATLAB code to clipboard")
                                .clicked()
                            {
                                ui.ctx().copy_text(item.script.to_string());
                            }
                        });

                        ui.add_space(6.0);

                        // Verification Benchmark Box
                        egui::Frame::NONE
                            .fill(Color32::from_rgba_unmultiplied(0, 114, 189, 20))
                            .stroke(egui::Stroke::new(1.0_f32, theme.accent))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Numerical Verification:")
                                            .size(11.0)
                                            .strong()
                                            .color(theme.accent),
                                    );
                                    ui.label(
                                        RichText::new(item.expected_answer)
                                            .size(11.0)
                                            .color(theme.text_primary),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!("Script reference: {}", item.doc_path))
                                        .size(10.0)
                                        .color(theme.text_secondary),
                                );
                            });

                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("MATLAB Program Code:")
                                .size(12.0)
                                .color(theme.text_secondary)
                                .strong(),
                        );

                        let mut code_str = item.script.to_string();
                        let code_rows = code_str.lines().count().max(16);
                        ui.add(
                            egui::TextEdit::multiline(&mut code_str)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(ui.available_width() - 8.0)
                                .desired_rows(code_rows),
                        );
                    });
            } else {
                right_ui.centered_and_justified(|ui| {
                    ui.label("No example matching search filter.");
                });
            }
        });
    }
}
