# Milestone A3 Learning Guide: Numeric Classes, N-D Indexing, and Sparse Matrices

## 1. Why MATLAB Needs Saturating Integer Arithmetic

In standard systems languages (such as C, C++, and Rust in release mode), integer operations wrap around on overflow (e.g., in unsigned 8-bit, $250 + 10 = 4$). In debug mode, Rust panics.

However, in numerical computing and signal processing, numerical saturation is the expected behavior:
- Audio samples, image pixels, and sensor ADC readings represent physical limits. When an image pixel exceeds max brightness ($255$), it should clip to white ($255$), not wrap around to black ($4$).
- Unary negation of the minimum signed integer (e.g. $-128$ for `int8`) cannot be represented in two's complement without saturation: in MATLAB, `-int8(-128)` and `abs(int8(-128))` saturate to `int8(127)`.

### How RustLab Implements Saturating Math:
RustLab utilizes Rust's standard `saturating_add`, `saturating_sub`, `saturating_mul`, and `saturating_neg` primitives, wrapping them in safe macros to guarantee that operations never panic or wrap unexpectedly.

---

## 2. Round Half Away from Zero

When converting continuous floats to discrete integers, different rounding modes exist:
- **Truncation (`fix`)**: discards fractional part ($2.9 \to 2$, $-2.9 \to -2$).
- **Floor**: rounds toward $-\infty$ ($2.1 \to 2$, $-2.1 \to -3$).
- **Ceiling**: rounds toward $+\infty$ ($2.1 \to 3$, $-2.1 \to -2$).
- **Round Half Away from Zero**: ties at $.5$ round away from zero ($2.5 \to 3$, $-2.5 \to -3$).

MATLAB type conversion constructors (`int8`, `uint16`, `int32`, etc.) strictly enforce **round half away from zero**. In RustLab, this is implemented as:
```rust
let rounded = if val >= 0.0 {
    (val + 0.5).floor()
} else {
    (val - 0.5).ceil()
};
```

---

## 3. Multidimensional Strides and Cartesian Slicing

In a column-major tensor of shape $[D_0, D_1, D_2]$, the linear offset of coordinate $(c_0, c_1, c_2)$ is:
$$\text{offset} = c_0 \cdot 1 + c_1 \cdot D_0 + c_2 \cdot (D_0 D_1)$$

When taking an arbitrary slice $A(r_0, r_1, r_2)$ where each $r_i$ is a vector of indices:
1. The shape of the sliced output tensor is $[|r_0|, |r_1|, |r_2|]$.
2. For every linear index in the sliced tensor, we convert the offset to output coordinates $(k_0, k_1, k_2)$ using column-major coordinate decoding.
3. The original coordinates are retrieved by lookup: $(r_0[k_0], r_1[k_1], r_2[k_2])$.
4. This decouples slicing from tensor dimensionality, providing seamless support for 2D, 3D, 4D, and N-D arrays.

---

## 4. Compressed Sparse Column (CSC) Storage

Dense storage stores every matrix element: an $N \times N$ matrix requires $8 N^2$ bytes. For $N = 10,000$, dense storage requires $800\text{ MB}$.

In contrast, Compressed Sparse Column (CSC) format stores only the non-zero elements:
- `col_ptr`: an array of length $N+1$ where `col_ptr[j]` points to the index in `row_ind` and `values` where column $j$ begins.
- `row_ind`: the row index of each non-zero entry.
- `values`: the scalar floating-point value of each non-zero entry.

### Duplicate Coordinate Accumulation
When users construct a sparse matrix with repeated indices, e.g., in finite element or graph calculations:
```matlab
S = sparse([1 1], [1 1], [10 20], 3, 3)
```
MATLAB automatically sums the duplicates: `S(1, 1)` becomes `30`. RustLab implements this by sorting coordinate triplets and accumulating matching coordinates before building the `col_ptr` offsets.
