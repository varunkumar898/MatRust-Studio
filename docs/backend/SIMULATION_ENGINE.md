# MatRust Studio Simulation Engine

## 1. Overview

The simulation engine provides mathematical modeling and time-dependent dynamic simulation capabilities for continuous and discrete control systems.

## 2. Capabilities

### 2.1 Transfer Function Models
- `sys = tf(num, den)`: Creates a continuous-time linear time-invariant transfer function:
  $$H(s) = \frac{\text{num}(s)}{\text{den}(s)} = \frac{b_m s^m + \dots + b_1 s + b_0}{a_n s^n + \dots + a_1 s + a_0}$$
- Stored as a first-class structured model in the workspace containing numerator/denominator polynomials and metadata.

### 2.2 Time Response Simulation
- `[y, t] = step(sys)` / `step(sys, tfinal)`: Computes the step response of transfer function `sys`.
- Analytical & Numerical Solvers:
  - **First-Order Systems** ($H(s) = \frac{K}{\tau s + 1}$):
    $$y(t) = K (1 - e^{-t/\tau})$$
  - **Second-Order Systems** ($H(s) = \frac{\omega_n^2}{s^2 + 2\zeta\omega_n s + \omega_n^2}$):
    - Underdamped ($\zeta < 1$): $y(t) = 1 - \frac{e^{-\zeta\omega_n t}}{\sqrt{1-\zeta^2}} \sin(\omega_d t + \phi)$
    - Critically damped ($\zeta = 1$): $y(t) = 1 - (1 + \omega_n t) e^{-\omega_n t}$
    - Overdamped ($\zeta > 1$): Sum of decayed real exponential modes.
  - **Higher-Order Systems**: Controllable canonical state-space transformation simulated via adaptive Runge-Kutta numerical integration.
- Step response outputs can be directly assigned to variables or plotted using the shared figure service: `plot(t, y)`.
