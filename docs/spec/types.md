# RustLab Specification: Type System & Array Representation

This specification defines the foundational data types, array dimensionality rules, type promotion hierarchies, and arithmetic semantics in `rustlab`.

---

## 1. Foundational Data Types

In MATLAB, **every value is conceptually an $N$-dimensional array**. There are no naked primitive scalar types:
- The literal `42` is a $1 \times 1$ matrix of type `double`.
- The boolean `true` is a $1 \times 1$ matrix of type `logical`.
- The character `'a'` is a $1 \times 1$ matrix of type `char`.

### 1.1 Core Type Hierarchy

```
                            Value (MATLAB Array)
                                     |
    +--------------------------------+--------------------------------+
    |                                |                                |
 Numeric                          Textual                        Container
    |                                |                                |
    +-- Floating Point               +-- char (char array)            +-- cell (cell array)
    |     |-- double (f64, default)  +-- string (string object)       +-- struct (scalar/array)
    |     \-- single (f32)                                            \-- function_handle
    +-- Integer
    |     |-- Signed: int8, int16, int32, int64
    |     \-- Unsigned: uint8, uint16, uint32, uint64
    \-- Logical (bool)
```

---

## 2. Array Dimensionality & Representation

### 2.1 Minimum 2D Rule
Every array in MATLAB has at least 2 dimensions:
- Scalar: Shape $[1, 1]$
- Row Vector: Shape $[1, N]$
- Column Vector: Shape $[M, 1]$
- Matrix: Shape $[M, N]$
- Multidimensional Array: Shape $[D_1, D_2, \dots, D_k]$ ($k \ge 2$)
- Empty array: Shape $[0, 0]$ (or $[0, N]$, $[M, 0]$)

### 2.2 Trailing Singleton Stripping
MATLAB automatically collapses trailing dimensions of size 1 for $k > 2$:
- An array created with dimensions $[2, 3, 1]$ is represented with shape $[2, 3]$.
- An array with shape $[2, 3, 1, 4, 1]$ is represented with shape $[2, 3, 1, 4]$.

### 2.3 Column-Major Memory Strides
Given dimensions $[D_1, D_2, \dots, D_k]$, element at 0-based coordinate $(i_1, i_2, \dots, i_k)$ resides at memory offset:
$$\text{Offset} = \sum_{j=1}^{k} \left( i_j \times \prod_{m=1}^{j-1} D_m \right)$$
For a 2D matrix of shape $[M, N]$:
$$\text{Offset}(r, c) = c \times M + r$$

---

## 3. Numeric Types & Representations

### 3.1 Double Precision (`double`)
- **Underlying Rust representation**: `f64`.
- Standard IEEE-754 64-bit floating point.
- Default type for all untyped numeric literals and operations.
- Special values: `NaN` (Not a Number), `Inf` (Infinity), `-Inf`.

### 3.2 Single Precision (`single`)
- **Underlying Rust representation**: `f32`.
- Standard IEEE-754 32-bit floating point.

### 3.3 Complex Numbers (`Complex<T>`)
- Every floating-point and integer array in MATLAB can optionally hold complex elements:
  $$z = a + bi$$
- Stored either as:
  - **Interleaved layout**: Array of `[re0, im0, re1, im1, ...]` (modern MATLAB standard).
  - **Split layout**: Separate arrays for Real part and Imaginary part (MATLAB legacy / MAT v5 file format standard).
- Arithmetic rules follow standard complex algebra:
  - $(a + bi) + (c + di) = (a + c) + (b + d)i$
  - $(a + bi) \times (c + di) = (ac - bd) + (ad + bc)i$
  - $\frac{a + bi}{c + di} = \frac{(ac + bd) + (bc - ad)i}{c^2 + d^2}$
  - Complex conjugate transpose ($A'$): Negates imaginary components and transposes dimensions.
  - Non-conjugate array transpose ($A.'$): Transposes dimensions without altering imaginary components.

### 3.4 Integer Classes & Saturating Arithmetic
MATLAB provides 8 integer classes:
`int8`, `uint8`, `int16`, `uint16`, `int32`, `uint32`, `int64`, `uint64`.

**Key Behavior Difference from C/Rust**:
1. **Saturation**: MATLAB integer arithmetic **saturates** on overflow/underflow rather than wrapping or panicking!
   ```matlab
   uint8(250) + uint8(10)   % 255 (saturates at upper bound)
   uint8(5) - uint8(10)     % 0   (saturates at lower bound)
   int8(120) + int8(20)     % 127
   ```
2. **Rounding on Division**: Integer division does NOT truncate toward zero; it **rounds to the nearest integer** (ties round away from zero):
   ```matlab
   int32(5) / int32(2)      % 3
   int32(7) / int32(2)      % 4
   ```

### 3.5 Logical (`logical`)
- Boolean values (`true` and `false`).
- Memory footprint: 1 byte per element (`u8`).
- When used in arithmetic operations, logical values are implicitly cast to `double` ($1.0$ and $0.0$).

---

## 4. Text Types: `char` vs `string`

MATLAB historical development created two distinct text types:

| Feature | Character Array (`char`) | String Object (`string`) |
| :--- | :--- | :--- |
| **Literal syntax** | `'Hello'` | `"Hello"` |
| **Element type** | UTF-16 code units | String slice/heap object |
| **Dimensionality** | Matrix of characters ($1 \times N$) | Array of strings (e.g. $1 \times 1$ scalar string) |
| **Indexing** | `s(1)` returns character `'H'` | `s(1)` returns full string `"Hello"` |
| **Concatenation** | `['Hello', ' World']` -> `'Hello World'` | `["Hello", "World"]` -> $1 \times 2$ string array |
| **Empty form** | `''` has shape $[0, 0]$ | `""` has shape $[1, 1]$ (empty string scalar) |

`rustlab` will support both representations, faithfully preserving their indexing and concatenation semantics.

---

## 5. Containers

### 5.1 Cell Arrays (`cell`)
- Heterogeneous container holding pointers to arbitrary MATLAB values.
- Dimensionality: Follows general $N$-D array rules.
- Memory: Contiguous buffer of `Box<Value>` or `Arc<Value>`.
- Indexing duality:
  - `C(i)` slices the cell array, returning a smaller cell array.
  - `C{i}` dereferences the element, returning the value itself.

### 5.2 Structures (`struct`)
- A collection of named fields mapping to MATLAB values.
- Can be **scalar struct** (one record) or **struct array** (an array of records sharing identical field names):
  ```matlab
  s(1).name = "Alice"; s(1).score = 95;
  s(2).name = "Bob";   s(2).score = 88;
  % s is a 1x2 struct array with fields: 'name', 'score'
  ```

---

## 6. Type Coercion & Arithmetic Mixing Rules

When evaluating binary operations $A \odot B$:

1. **Integer with Double / Single**:
   - In MATLAB, mixing any integer type with a floating-point scalar converts the float to that integer type (with saturation and rounding):
     ```matlab
     int32(10) + 2.6       % int32(13)
     ```
   - Mixing an integer array with a non-scalar double is prohibited in standard MATLAB to prevent unintended precision loss.

2. **Single with Double**:
   - Converts `double` to `single`:
     ```matlab
     single(1.5) + 2.0     % single(3.5)
     ```

3. **Logical with Any Numeric Type**:
   - `logical` is converted to the other operand's type:
     ```matlab
     true + 4.5            % double(5.5)
     false + int8(10)      % int8(10)
     ```
