# Inverting Operational Amplifier

## 1. Overview
The **Inverting Op-Amp** example models an operational amplifier inverting configuration with closed-loop gain $A_v = -\frac{R_f}{R_1} = -10.0$. The op-amp core utilizes the dominant-pole macro-model (high DC open-loop gain compensated with an integrator pole), coupling a `VoltageSensor` across the virtual ground to a `VoltageControlledVoltageSource` (VCVS).

- Model file: `examples/models/inverting_opamp.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_inverting_opamp`

## 2. Circuit & Macro-Model Topology
```
           +-----------+         +------------+
  (+) -----| R1 = 10 k |----+----| Rf = 100 k |----+------------ (+)
Vin=1.0V   +-----------+    |    +------------+    |         Vout (-10V)
  (-)                       |                      |
   |                        |                      |
  GND (0)                   |                      |
                            |                      |
                  (-) ------+                      |
               VSens_Sum                           |
                  (+) ------+                      |
                            |                      |
                         GND (0)                   |
                                                   |
Macro-Model Signal Path:                           |
  VSens_Sum ---> OpAmp_Gain (20,000) ---> Integrator ---> OpAmp_VCVS (+)
                                                              (-) --- GND
```

## 3. Mathematical Derivation
At the inverting summing junction $V_s$, Kirchhoff's Current Law yields:
$$\frac{V_{in} - V_s}{R_1} + \frac{V_{out} - V_s}{R_f} = 0$$

The differential sensor measures the virtual ground potential:
$$v_{\epsilon}(t) = 0 - V_s(t) = -V_s(t)$$

The dominant-pole op-amp open-loop transfer function is:
$$A(s) = \frac{\omega_u}{s} = \frac{20000}{s}$$
$$V_{out}(s) = A(s) \cdot (-V_s(s)) = -\frac{\omega_u}{s} V_s(s)$$

Substituting $V_s$ into the KCL equation:
$$V_s \left( \frac{1}{R_1} + \frac{1}{R_f} \right) = \frac{V_{in}}{R_1} + \frac{V_{out}}{R_f}$$
$$-\frac{s}{\omega_u} V_{out} \left( \frac{R_1 + R_f}{R_1 R_f} \right) = \frac{V_{in}}{R_1} + \frac{V_{out}}{R_f}$$

Rearranging gives the closed-loop transfer function:
$$\frac{V_{out}(s)}{V_{in}(s)} = -\frac{R_f / R_1}{1 + s / \omega_c}$$
where the closed-loop bandwidth is:
$$\omega_c = \omega_u \frac{R_1}{R_1 + R_f} = 20000 \times \frac{10\text{ k}}{110\text{ k}} \approx 1818.18\text{ rad/s}$$
and the effective time constant is:
$$\tau = \frac{1}{\omega_c} \approx 0.55\text{ ms}$$

At DC ($s \to 0$), the steady-state closed-loop gain is exact:
$$V_{out} = -\frac{R_f}{R_1} V_{in} = -\frac{100\text{ k}\Omega}{10\text{ k}\Omega} \times 1.0\text{ V} = -10.000\text{ V}$$

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 100\,\mu\text{s}$, $t_{stop} = 20\text{ ms}$ ($> 36\tau$).
- Steady-state output: Reaches $-10.000\text{ V}$ with error $< 1\text{ mV}$.
- Stability: Completely eliminates instantaneous algebraic loops through dominant-pole state dynamics.
