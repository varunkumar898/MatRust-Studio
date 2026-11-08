# Boost Converter (Step-Up DC-DC)

## 1. Overview
The **Boost Converter** example models a step-up switched-mode DC-DC power converter. A low-voltage $12\text{ V}$ DC supply is stepped up to approximately $24\text{ V}$ using an energy storage inductor ($L = 200\,\mu\text{H}$), an `IdealSwitch` clocked at $50\text{ kHz}$ by a `PulseGenerator`, a boost diode ($V_f = 0.7\text{ V}$), and a filter capacitor ($C = 100\,\mu\text{F}$) powering a $24\,\Omega$ resistive load.

- Model file: `examples/models/boost_converter.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_boost_converter`

## 2. Circuit Diagram
```
           L = 200 uH              D_boost (Vf=0.7)
  (+) ------/\/\/\/\-----+---------------+-->|---+------------+--- (+)
Vin=12V                  |                       |            |  Vout (~24V)
                         \ SW_boost             --- C_out    [ ] R_load = 24
                         / (IdealSwitch)        --- 100 uF   [ ]
  (-)                    |                       |            |
   |                     |                       |            |
  GND -------------------+-----------------------+------------+--- (-)
                         |                       |
                        GND                     GND

Gate Drive:
  PulseGenerator (50 kHz, Period = 20 us, Duty = 52%) ---> SW_boost gate
```

## 3. Converter Theory
- Nominal input: $V_{in} = 12.0\text{ V}$
- Switching frequency: $f_s = 50\text{ kHz}$ ($T_s = 20\,\mu\text{s}$)
- Duty cycle: $D = 0.52$ ($t_{on} = 10.4\,\mu\text{s}$)
- Ideal continuous-conduction mode (CCM) voltage transfer ratio:
  $$V_{out} = \frac{V_{in} - V_f}{1 - D} = \frac{12.0 - 0.7}{1 - 0.52} = \frac{11.3}{0.48} \approx 23.54\text{ V}$$

Inductor energy balance:
- During switch ON ($t_{on}$): $v_L = V_{in}$, inductor current rises linearly by $\Delta I_L = \frac{V_{in} \cdot t_{on}}{L}$.
- During switch OFF ($t_{off}$): $v_L = V_{in} - V_{out}$, energy is transferred into the output capacitor and load.

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 0.2\,\mu\text{s}$, $t_{stop} = 5\text{ ms}$.
- Output voltage: Successfully steps up from $12\text{ V}$ to steady-state mean $> 22\text{ V}$ ($~23.5\text{ V}$).
- Continuous conduction: Inductor current stays strictly positive ($I_L > 0$), confirming stable continuous conduction mode.
