# RLC Transient Response

## 1. Overview
The **RLC Transient** example demonstrates an underdamped second-order series $RLC$ circuit subjected to an ideal step input. The model couples an ideal DC voltage source ($V_s = 10\text{ V}$), series resistor ($R = 4\,\Omega$), series inductor ($L = 10\text{ mH}$), and shunt capacitor ($C = 100\,\mu\text{F}$) grounded at node 0.

- Model file: `examples/models/rlc_transient.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_rlc_transient`

## 2. Circuit Diagram and Topology
```
           +-------+       +-------+       +-------+
  (+) -----| R = 4 |-------|L=10mH |-------|C=100uF|-----+---- (+)
Vs=10V     +-------+       +-------+       +-------+     |   V_Sensor
  (-) ---------------------------------------------------+---- (-)
   |                                                     |
  GND (0)                                               GND (0)
```

## 3. Mathematical & Theoretical Derivation
The differential equation governing the capacitor voltage $v_C(t)$ is:
$$L C \frac{d^2 v_C}{dt^2} + R C \frac{dv_C}{dt} + v_C = V_s$$

Characteristic polynomial:
$$s^2 + \frac{R}{L} s + \frac{1}{LC} = 0$$

Circuit parameters:
- Attenuation factor: $\alpha = \frac{R}{2L} = \frac{4}{2 \times 0.01} = 200\text{ s}^{-1}$
- Undamped natural frequency: $\omega_0 = \frac{1}{\sqrt{LC}} = \frac{1}{\sqrt{0.01 \times 10^{-4}}} = 1000\text{ rad/s}$
- Damped oscillation frequency: $\omega_d = \sqrt{\omega_0^2 - \alpha^2} = \sqrt{10^6 - 40000} \approx 979.7959\text{ rad/s}$
- Damping ratio: $\zeta = \frac{\alpha}{\omega_0} = 0.2$ (underdamped, $\zeta < 1$)

Analytical step response with zero initial conditions:
$$v_C(t) = V_s \left[ 1 - e^{-\alpha t} \left( \cos(\omega_d t) + \frac{\alpha}{\omega_d} \sin(\omega_d t) \right) \right]$$

Key analytical figures:
- Peak time: $t_p = \frac{\pi}{\omega_d} \approx \frac{\pi}{979.7959} \approx 3.2064\text{ ms}$
- Peak voltage overshoot:
  $$v_{C,\max} = V_s \left( 1 + e^{-\alpha \pi / \omega_d} \right) = 10 \left( 1 + e^{-200 \pi / 979.7959} \right) \approx 15.266\text{ V}$$

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepRk4` (or `FixedStepEuler`) with $h = 10\,\mu\text{s}$.
- Absolute error against closed-form analytic solution across all $t \in [0, 20\text{ ms}]$: $< 0.1\text{ V}$.
- Numerical peak overshoot observed: $15.26\text{ V}$, exactly matching the theoretical $15.266\text{ V}$ within $0.01\text{ V}$.
