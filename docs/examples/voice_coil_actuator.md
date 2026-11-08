# Voice Coil Actuator (Electromechanical Transducer)

## 1. Overview
The **Voice Coil Actuator** example models a linear electromagnetic transducer (such as in acoustic speakers, hard disk drive head positioners, and fast steering mirrors) coupled to a second-order mass-spring-damper suspension. An AC voltage source excites the coil, inducing Lorentz force and driving linear oscillations with electromechanical damping from velocity back-EMF.

- Model file: `examples/models/voice_coil_actuator.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_voice_coil_actuator`

## 2. Block Diagram & Circuit Topology
```
Electrical Domain:
  (+) --- V_ac (10V, 10Hz) --- R_coil (8 Ohm) --- L_coil (1 mH) --- I_sens --- V_bemf (VCVS) --- GND
  (-) --- GND

Mechanical Domain:
  I_sens ---> Gain_BL (5.0 N/A) ---> (+)
                                     ForceSum ---> Gain_InvM (10) ---> Integ_Vel ---> Integ_Pos ---> X_out
  Integ_Vel ---> Gain_C (2.0 N*s/m) -> (-)                                |
  Integ_Pos ---> Gain_K (1000 N/m) --> (-)                                +---> V_bemf (5.0 V*s/m)
                                                                          +---> V_out
```

## 3. Mathematical Model & Derivation
Electrical equation:
$$v_{ac}(t) = R_{coil} i(t) + L_{coil} \frac{di(t)}{dt} + e(t)$$
$$e(t) = (B \cdot l) \cdot \dot{x}(t)$$

Mechanical equation (Newton's second law):
$$m \ddot{x}(t) + c \dot{x}(t) + k x(t) = F_L(t)$$
$$F_L(t) = (B \cdot l) \cdot i(t)$$

Parameters:
- Coil resistance: $R_{coil} = 8.0\,\Omega$
- Coil inductance: $L_{coil} = 1.0\text{ mH} = 0.001\text{ H}$
- Force / back-EMF coupling factor: $B \cdot l = 5.0\text{ N/A} = 5.0\text{ V}\cdot\text{s/m}$
- Moving mass: $m = 0.1\text{ kg}$
- Viscous damping coefficient: $c = 2.0\text{ N}\cdot\text{s/m}$
- Suspension spring stiffness: $k = 1000.0\text{ N/m}$

Mechanical Resonance Characteristics:
- Undamped natural frequency:
  $$\omega_n = \sqrt{\frac{k}{m}} = \sqrt{\frac{1000}{0.1}} = 100\text{ rad/s} \quad (f_n \approx 15.915\text{ Hz})$$
- Mechanical damping ratio:
  $$\zeta_m = \frac{c}{2 \sqrt{k m}} = \frac{2}{2 \sqrt{100}} = 0.1$$
- Total effective electromechanical damping:
  $$c_{eff} = c + \frac{(B \cdot l)^2}{R_{coil}} = 2.0 + \frac{25}{8} = 2.0 + 3.125 = 5.125\text{ N}\cdot\text{s/m}$$
  $$\zeta_{total} = \frac{c_{eff}}{2 \sqrt{k m}} = \frac{5.125}{20} \approx 0.256$$

The back-EMF feedback provides strong intrinsic electromagnetic damping, suppressing resonant peak amplitude.

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 100\,\mu\text{s}$, $t_{stop} = 0.2\text{ s}$.
- Stability: Position $x(t)$ and velocity $v(t)$ oscillate boundedly with zero divergence.
- Transducer reciprocity: Verified coupling factor $B \cdot l = 5.0$ in both forward actuation ($F = B l i$) and reverse sensing ($e = B l \dot{x}$).
