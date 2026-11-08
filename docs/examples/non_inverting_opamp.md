# Non-Inverting Operational Amplifier

## 1. Overview
The **Non-Inverting Op-Amp** example models a closed-loop non-inverting operational amplifier configuration with theoretical gain $A_v = 1 + \frac{R_f}{R_1} = 1 + \frac{100\text{ k}}{10\text{ k}} = 11.0$.

- Model file: `examples/models/non_inverting_opamp.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_non_inverting_opamp`

## 2. Circuit & Macro-Model Topology
```
  (+) Vin (1.0V) ----------------- (+)
                                   VSens_Diff
  (-) --- GND                      (-) ---------------------+
                                                            |
                                                   +--------+--------+
                                                   |                 |
                                             +-----------+     +------------+
                                             | R1 = 10 k |     | Rf = 100 k |
                                             +-----------+     +------------+
                                                   |                 |
                                                  GND                |
                                                                     |
Macro-Model:                                                         |
  VSens_Diff ---> OpAmp_Gain (20,000) ---> Integrator ---> OpAmp_VCVS (+)
                                                              (-) --- GND
```

## 3. Mathematical Derivation
The differential voltage sensor measures the error voltage between the input and the feedback divider:
$$v_{\epsilon}(t) = V_+ - V_- = V_{in}(t) - V_-(t)$$

The resistive divider sets:
$$V_-(t) = \frac{R_1}{R_1 + R_f} V_{out}(t)$$

Op-amp macro-model dynamics:
$$V_{out}(s) = \frac{\omega_u}{s} (V_{in}(s) - V_-(s))$$

Substituting $V_-$:
$$V_{out}(s) = \frac{\omega_u}{s} \left( V_{in}(s) - \frac{R_1}{R_1 + R_f} V_{out}(s) \right)$$
$$V_{out}(s) \left[ 1 + \frac{\omega_u R_1}{s (R_1 + R_f)} \right] = \frac{\omega_u}{s} V_{in}(s)$$

Closed-loop transfer function:
$$\frac{V_{out}(s)}{V_{in}(s)} = \frac{1 + R_f / R_1}{1 + s / \omega_c}$$
where $\omega_c = \omega_u \frac{R_1}{R_1 + R_f} \approx 1818.18\text{ rad/s}$ ($\tau \approx 0.55\text{ ms}$).

At DC ($s \to 0$):
$$V_{out} = \left( 1 + \frac{R_f}{R_1} \right) V_{in} = \left( 1 + \frac{100\text{ k}\Omega}{10\text{ k}\Omega} \right) \times 1.0\text{ V} = 11.000\text{ V}$$

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 100\,\mu\text{s}$, $t_{stop} = 20\text{ ms}$.
- Steady-state output: Settles to $+11.000\text{ V}$ with error $< 1\text{ mV}$.
- Virtual short: $V_+ - V_- \to 0\text{ V}$ as $t \to \infty$.
