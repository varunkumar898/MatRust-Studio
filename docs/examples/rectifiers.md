# Half-Wave and Filter Rectifiers

## 1. Overview
The **Rectifiers** example models AC-to-DC power conversion using a semiconductor diode ($V_f = 0.7\text{ V}$, $R_{on} = 0.01\,\Omega$, $R_{off} = 1\text{ M}\Omega$) connected to a smoothing filter capacitor ($C = 1\text{ mF}$) and resistive load ($R = 100\,\Omega$).

- Model file: `examples/models/rectifiers.json`
- Verification test: `crates/mlapp-sim/tests/examples_tests.rs::test_example_rectifiers`

## 2. Circuit Diagram
```
          +------+      +--------+
  (+) ----|  D1  |------|C=1000uF|-----+----- R_load = 100 -----+
Vac=10V   | (->) |      +--------+     |                       |
 50 Hz    +------+                     +----- V_Sensor --------+
  (-) ---------------------------------+-----------------------+
   |                                   |
  GND (0)                             GND (0)
```

## 3. Theoretical Derivation
- Source: $v_{ac}(t) = 10 \sin(2\pi \cdot 50 \cdot t)$
- Peak source voltage: $V_p = 10.0\text{ V}$
- Period: $T = \frac{1}{50\text{ Hz}} = 20\text{ ms}$
- Diode forward barrier: $V_f = 0.7\text{ V}$

Peak output voltage across the filter capacitor:
$$V_{out,\max} = V_p - V_f = 10.0 - 0.7 = 9.3\text{ V}$$

During the negative half-cycle and when $v_{ac}(t) < v_C(t)$, the diode is reverse-biased (OFF, $R_{off} = 1\text{ M}\Omega$). The capacitor discharges through $R_{load}$ with time constant $\tau = R C = 100 \times 10^{-3} = 0.1\text{ s} = 100\text{ ms}$.

Because $\tau = 100\text{ ms} \gg T = 20\text{ ms}$, the discharge is approximately linear:
$$\Delta V_{ripple} \approx \frac{I_{load} \cdot T}{C} = \frac{V_{out,\max}}{R_{load}} \cdot \frac{T}{C} = \frac{9.3\text{ V}}{100\,\Omega} \cdot \frac{0.02\text{ s}}{10^{-3}\text{ F}} = 1.86\text{ V}$$

## 4. Simulation Results & Verification
- Numerical solver: `FixedStepEuler` with $h = 10\,\mu\text{s}$, $t_{stop} = 60\text{ ms}$ (3 full AC cycles).
- Peak rectified voltage: $9.29\text{ V}$, matching $V_p - V_f = 9.3\text{ V}$ to within $0.02\text{ V}$.
- Steady-state peak-to-peak ripple: $1.75\text{ V}$ to $1.85\text{ V}$, matching theoretical formula $\Delta V \approx 1.86\text{ V}$.
- Zero non-physical chatter: diode transitions occur at exact zero-crossing thresholds without triggering $W\text{-}CHATTER$.
