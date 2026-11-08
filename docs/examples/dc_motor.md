# Electromechanical DC Motor Velocity Control

## 1. Overview
The **DC Motor** example models a permanent magnet DC (PMDC) servomotor coupled to a rotational mechanical load with closed-loop proportional-integral (PI) velocity control. The electrical armature and mechanical rotor are coupled bidirectionally: armature current produces electromagnetic torque via $K_t$, while rotor angular velocity induces back-EMF voltage via $K_e$.

- Model file: `examples/models/dc_motor.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_dc_motor`

## 2. Block Diagram & Circuit Topology
```
Electrical Armature:
  (+) --- V_arm (VCVS) --- R_arm (2 Ohm) --- L_arm (1 mH) --- I_sens --- V_bemf (VCVS) --- GND
  (-) --- GND

Mechanical Rotor:
  I_sens ---> Gain_Kt (0.05 N*m/A) ---> (+)
                                         TorqueSum ---> Gain_InvJ (1000) ---> Integ_Omega ---> Omega
  Omega  ---> Gain_B  (0.005 N*m*s) ---> (-)                                     |
                                                                                +---> V_bemf (0.05 V*s/rad)
Control Loop:                                                                   |
  Omega_ref (50 rad/s) ---> (+)                                                 |
                            ErrSum ---> PI_Controller (P=1.0, I=10.0) ---> V_arm
  Omega (feedback) -------> (-)
```

## 3. Mathematical Model & Derivation
Armature circuit Kirchhoff's Voltage Law:
$$V_a(t) = R_a i_a(t) + L_a \frac{di_a(t)}{dt} + e(t)$$
$$e(t) = K_e \omega(t)$$

Mechanical rotor Newton-Euler dynamics:
$$J \frac{d\omega(t)}{dt} = T_e(t) - B \omega(t) - T_L(t)$$
$$T_e(t) = K_t i_a(t)$$

Parameters:
- Armature resistance: $R_a = 2.0\,\Omega$
- Armature inductance: $L_a = 1.0\text{ mH} = 0.001\text{ H}$
- Back-EMF constant: $K_e = 0.05\text{ V}\cdot\text{s/rad}$
- Torque constant: $K_t = 0.05\text{ N}\cdot\text{m/A}$
- Rotor inertia: $J = 0.001\text{ kg}\cdot\text{m}^2$
- Viscous friction: $B = 0.005\text{ N}\cdot\text{m}\cdot\text{s/rad}$

Steady-State Operation at $\omega^* = 50.0\text{ rad/s}$:
- Frictional drag torque: $T_{drag} = B \omega^* = 0.005 \times 50.0 = 0.25\text{ N}\cdot\text{m}$
- Steady-state armature current: $I_a^* = \frac{T_{drag}}{K_t} = \frac{0.25}{0.05} = 5.0\text{ A}$
- Back-EMF: $E^* = K_e \omega^* = 0.05 \times 50.0 = 2.5\text{ V}$
- Required armature voltage: $V_a^* = E^* + R_a I_a^* = 2.5 + 2.0 \times 5.0 = 12.5\text{ V}$

Under PI control $u(t) = K_p (\omega_{ref} - \omega) + K_i \int (\omega_{ref} - \omega) dt$, the integral action adjusts $V_a \to 12.5\text{ V}$, guaranteeing zero steady-state tracking error ($\omega \to 50.00\text{ rad/s}$).

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 100\,\mu\text{s}$, $t_{stop} = 1.0\text{ s}$.
- Final velocity: Tracks setpoint $\omega_{ref} = 50.0\text{ rad/s}$ with steady-state error $< 0.1\%$.
- Cross-domain conservation: Torque $T_e = K_t I_a$ and back-EMF $E = K_e \omega$ enforce power conservation $E \cdot I_a = T_e \cdot \omega$.
