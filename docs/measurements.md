# Measurement and Spectral Analysis Architecture

*Single source of truth for measurement definitions, sign conventions, discrete windowing, latency, and limits.*

## 1. Overview
MatRust Studio provides allocation-free, streaming measurement blocks in the continuous/discrete signal-flow domain:
- `Rms`: True Root-Mean-Square value over a sliding fundamental period window $T = 1 / f_{base}$.
- `Mean`: Running average (DC component) over a sliding fundamental period window $T = 1 / f_{base}$.
- `PowerMeasurement`: Instantaneous voltage and current measurement computing Active Power ($P$), fundamental Reactive Power ($Q_1$), Apparent Power ($S$), Power Factor ($PF$), and Displacement Power Factor ($DPF$).
- `Thd`: Total Harmonic Distortion up to harmonic $H$ relative to the fundamental component.
- `SpectrumResult` & `spectrum()`: Offline frequency analysis via radix-2 FFT and arbitrary-length DFT.
- `SimulationOutput::write_csv`: Deterministic RFC 4180 CSV export.

---

## 2. Mathematical Formulations & Definitions

### 2.1 True RMS (`Rms`)
Over a sliding window of duration $T = 1 / f_{base}$ with discrete sample period $T_s$ and window sample count $N_w = \text{round}(T / T_s)$:
$$U_{rms}[k] = \sqrt{\frac{1}{N_w} \sum_{n=0}^{N_w - 1} u^2[k - n]}$$
The sum of squares is accumulated using Neumaier compensated summation and recomputed exactly from the circular ring buffer every $N_w$ steps to ensure zero drift over millions of simulation steps. The sum is clamped to $\ge 0$ before computing the square root.

### 2.2 Running Mean (`Mean`)
$$U_{dc}[k] = \frac{1}{N_w} \sum_{n=0}^{N_w - 1} u[k - n]$$
Accumulated via Neumaier compensated summation with exact periodic recomputation every $N_w$ steps.

### 2.3 Power Measurement (`PowerMeasurement`)
Processes instantaneous differential voltage $v(t)$ and branch current $i(t)$:
- **Active Power ($P$)**:
  $$P[k] = \frac{1}{N_w} \sum_{n=0}^{N_w - 1} v[k - n] \cdot i[k - n]$$
  Updated every sample step.
- **Apparent Power ($S$)**:
  $$S[k] = V_{rms}[k] \cdot I_{rms}[k]$$
  Updated every sample step.
- **Power Factor ($PF$)**:
  $$PF[k] = \begin{cases} 1.0 & \text{if } S[k] < 10^{-9} \\ \text{clamp}\left(\frac{P[k]}{S[k]}, -1.0, 1.0\right) & \text{otherwise} \end{cases}$$
  Updated every sample step.
- **Fundamental Reactive Power ($Q_1$)**:
  Computed from single-bin discrete Fourier analysis of $v$ and $i$ at the fundamental frequency $f_{base}$ over one complete window:
  $$A_v = \sum_{n=0}^{N_w - 1} v[k - n] \cos\left(\frac{2\pi n}{N_w}\right), \quad B_v = \sum_{n=0}^{N_w - 1} v[k - n] \sin\left(\frac{2\pi n}{N_w}\right)$$
  $$A_i = \sum_{n=0}^{N_w - 1} i[k - n] \cos\left(\frac{2\pi n}{N_w}\right), \quad B_i = \sum_{n=0}^{N_w - 1} i[k - n] \sin\left(\frac{2\pi n}{N_w}\right)$$
  $$Q_1 = \frac{2}{N_w^2} (A_v B_i - B_v A_i)$$
  Updated once per window ($N_w$ steps) and held constant across the subsequent window.
- **Displacement Power Factor ($DPF$)**:
  $$DPF = \cos(\theta_v - \theta_i) = \frac{A_v A_i + B_v B_i}{\sqrt{A_v^2 + B_v^2} \sqrt{A_i^2 + B_i^2}}$$
  Clamped to $[-1.0, 1.0]$. Updated once per window ($N_w$ steps) and held.

### 2.4 Total Harmonic Distortion (`Thd`)
Extracts harmonic RMS magnitudes $V_h$ for $h = 1, \dots, H$ using precomputed discrete Fourier transform basis tables:
$$\text{THD} = \frac{\sqrt{\sum_{h=2}^H V_h^2}}{V_1}$$
- When `Output = Percent` (default), the ratio is multiplied by $100.0$.
- When `Output = Ratio`, the decimal ratio is returned directly.
- If $V_1 < 10^{-9}$, output is $0.0$ and diagnostic `I-THD-NOFUND` is logged.

---

## 3. Sign Convention & Passive Sign Convention
All electrical powers and phases adhere to the **passive sign convention**:
- Voltage is measured from terminal $p$ to terminal $n$ ($v = V_p - V_n$).
- Current $i$ enters terminal $p$ and leaves terminal $n$.
- For an inductive load where current lags voltage ($\theta_i < \theta_v$), the phase difference $\theta_v - \theta_i > 0$, yielding **$Q_1 > 0$** and **$P > 0$**.
- For a capacitive load where current leads voltage ($\theta_i > \theta_v$), **$Q_1 < 0$**.

---

## 4. Latency and Startup Behaviour
1. **Zero Direct Feedthrough**: Measurement blocks have no algebraic feedthrough (`default_feedthrough = false`). The output at time step $k$ strictly reflects samples up to step $k-1$ (one-sample latency). They function as state blocks breaking algebraic loops (`E-ALG`) and bridge feedback loops (`W-DELAY`).
2. **Startup Period ($t < T$)**: Until the circular ring buffer receives a full window of $N_w$ samples, the output of `Rms` and `Mean` is held at `InitialValue` (default 0.0). `PowerMeasurement` outputs $[0, 0, 0, 1, 1]$. `Thd` outputs $0.0$.
3. **Window Update Latency**:
   - $P$, $S$, and $PF$ update every sample step after $t \ge T$.
   - $Q_1$, $DPF$, and $\text{THD}$ update once per fundamental window ($N_w$ steps) and are held until the next full window completes.

---

## 5. Parameter Validation & Limits
- **$N_w \ge 4$**: A minimum of 4 samples per fundamental period is enforced. If $N_w < 4$, compile fails with `E-PARAM`.
- **`W-WINDOW`**: If $|T / T_s - N_w| > 10^{-6} N_w$, warning `W-WINDOW` is emitted, informing the user that the step size does not divide the fundamental period into an exact integer of samples.
- **`W-THD-LIMIT`**: Harmonic order $H$ is clamped to the discrete Nyquist limit $\lfloor (N_w - 1) / 2 \rfloor$. If the requested $H$ exceeds this limit, warning `W-THD-LIMIT` is emitted.
- **Zero Allocations (Charter P5)**: All ring buffers, tables, and summation accumulators are pre-allocated during diagram compilation. Stepping through 10,000 steps allocates zero bytes on the heap.
