# Supported MATLAB Functions Catalog: `mlapp-rs`

This document details the built-in function library available in the native runtime, error diagnostic behavior for unsupported functions, and the optional GNU Octave bridge.

---

## 1. Core Built-In Functions

| Function | Category | Description | Source / Status |
| :--- | :--- | :--- | :--- |
| `size`, `length`, `numel`, `ndims` | Matrix Properties | Returns matrix dimensions and element counts | Clean-room Rust |
| `zeros`, `ones`, `eye`, `rand`, `randn` | Matrix Constructors | Allocates 2D and N-D numerical tensors | Clean-room Rust |
| `linspace`, `reshape`, `repmat` | Shape Manipulation | Linearly spaced vectors and tensor reshaping | Clean-room Rust |
| `sum`, `prod`, `cumsum`, `mean`, `std`, `min`, `max` | Statistics & Reductions | Summary metrics with NaN handling | Clean-room Rust |
| `abs`, `sqrt`, `exp`, `log`, `log10` | Elementary Math | Element-wise arithmetic | Clean-room Rust |
| `sin`, `cos`, `tan`, `asin`, `acos`, `atan` | Trigonometry | Circular and inverse functions | Clean-room Rust |
| `sort`, `find`, `unique`, `any`, `all` | Search & Sorting | Relational and boolean index searching | Clean-room Rust |
| `strcat`, `sprintf`, `num2str`, `str2double` | String & Formatting | Formatted printing and numeric conversion | Clean-room Rust |
| `disp`, `fprintf` | Console Output | Formatted message output | Clean-room Rust |
| `uialert`, `uiconfirm` | App Designer Dialogs | Dispatches alert modals to host UI | Native Host Modal |
| `drawnow`, `pause` | Event Loop | Flushes UI mutation queues, sleeps worker | Thread safe |
| `plot`, `scatter`, `bar` | 2D Graphics | Plots data series into `UIAxes` | `mlapp-plot` |
| `xlabel`, `ylabel`, `title`, `grid`, `hold` | Axes Styling | Configures axes labels, bounds, and styling | `mlapp-plot` |
| `inv`, `det`, `mldivide (\)`, `mrdivide (/)` | Linear Algebra | Decompositions and matrix equations | `faer` / native |

---

## 2. Unsupported Function Policy

Unsupported functions **never fail silently**. Calling an unrecognized function immediately raises a loud diagnostic:

```
RuntimeError: Unsupported MATLAB function 'besselk'.
Recommendation: Enable the 'octave-bridge' cargo feature to forward unimplemented functions to GNU Octave.
```

---

## 3. Optional GNU Octave Bridge (Tier 4)

Behind cargo feature `octave-bridge`:
- Spawns a sandboxed `octave-cli` subprocess.
- Marshals inputs and outputs via JSON / binary pipe.
- Does **not** link against GPL code: process boundary preserves clean license isolation.
- Used strictly as an on-demand fallback when the native runtime does not yet implement a specialized toolbox function.
