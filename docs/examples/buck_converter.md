# Synchronous / Asynchronous Buck Converter

## 1. Overview
The **Buck Converter** example models a closed-loop step-down switched-mode DC-DC power supply. An unregulated $24\text{ V}$ DC rail is converted down to a regulated $5.0\text{ V}$ output using pulse-width modulation (PWM) switching at $50\text{ kHz}$ with feedback error amplification (PID controller).

- Model file: `examples/models/buck_converter.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_buck_converter`

## 2. Circuit & Control Diagram
```
              SW1 (IdealSwitch)       L = 100 uH
  (+) ---------+---/  ---+------------+--/\/\/\/\--+--------- (+)
Vin=24V                  |                         |        Vout (5V)
                         | D_freewheel            --- C_out
  (-)                    | (Diode, Vf=0.7)        --- 47 uF   R_load = 5 Ohm
   |                     |                         |         V_Sensor
  GND -------------------+-------------------------+--------- (-)
                         |                         |
                        GND                       GND

Control Domain:
  V_ref (5.0V) ---> (+)
                    Sum ---> PID_ctrl ---> PWM_Comp (>=) ---> Gate (SW1)
  V_Sensor --------> (-)                     ^
                                             |
                         Sawtooth (50 kHz) --+
```

## 3. Converter Theory
- Nominal input: $V_{in} = 24.0\text{ V}$
- Reference target: $V_{ref} = 5.0\text{ V}$
- Switching frequency: $f_s = 50\text{ kHz}$ ($T_s = 20\,\mu\text{s}$)
- Inductance: $L = 100\,\mu\text{H}$
- Capacitance: $C = 47\,\mu\text{F}$
- Load resistance: $R = 5.0\,\Omega$ ($I_{out} = 1.0\text{ A}$)

Ideal continuous-conduction mode (CCM) duty cycle:
$$D \approx \frac{V_{out}}{V_{in}} = \frac{5.0}{24.0} \approx 0.2083$$

Peak-to-peak inductor ripple current:
$$\Delta I_L = \frac{(V_{in} - V_{out}) D}{f_s L} = \frac{(24 - 5) \times 0.2083}{50000 \times 10^{-4}} = \frac{3.958}{5} \approx 0.79\text{ A}$$

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 0.2\,\mu\text{s}$ (100 steps per PWM cycle).
- Steady-state output: Settles to $5.00\text{ V} \pm 0.1\text{ V}$ with zero steady-state error under closed-loop integral action.
- Pre-factorization fast path: 2 distinct switch state combinations ($2^1 = 2$ topologies) pre-factored into cache, executing with zero heap allocation during step iterations.
