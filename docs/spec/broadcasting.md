# RustLab Specification: Implicit Expansion & Broadcasting Rules

This specification formalizes the implicit expansion (broadcasting) semantics implemented by `rustlab` for elementwise operators and built-in functions.

---

## 1. Concept Overview

Prior to MATLAB R2016b, elementwise binary operations required arrays to have identical shapes, requiring manual calls to `bsxfun`. Modern MATLAB performs **implicit expansion** automatically whenever two arrays have compatible shapes.

An operation $C = A \odot B$ implicitly expands both $A$ and $B$ to a common shape $S$, where each element $C(i_1, i_2, \dots)$ is computed from the corresponding (or broadcasted) elements of $A$ and $B$.

---

## 2. Dimensional Compatibility Rules

Two arrays $A$ and $B$ are compatible for implicit expansion if, for every dimension $k = 1, 2, \dots, \max(\text{ndims}(A), \text{ndims}(B))$:

1. **Equal Sizes**:
   $$\text{size}(A, k) = \text{size}(B, k)$$
2. **Singleton Expansion**:
   $$\text{size}(A, k) = 1 \quad \text{or} \quad \text{size}(B, k) = 1$$

If an array has fewer dimensions than the other, its missing trailing dimensions are treated as having size $1$.

### Resulting Output Shape
For each dimension $k$:
$$\text{size}(C, k) = \max(\text{size}(A, k), \text{size}(B, k))$$

If any dimension has differing sizes and neither is $1$, the operation throws:
```
Error: Arrays have incompatible sizes for this operation.
```

---

## 3. Concrete Broadcasting Scenarios

### 3.1 Scalar and Array
A scalar has shape $[1, 1]$. It expands along all dimensions to match any target array:
```matlab
A = [1 2; 3 4];   % Shape: [2, 2]
B = 10;           % Shape: [1, 1]
C = A + B;        % Shape: [2, 2] -> [11 12; 13 14]
```

### 3.2 Column Vector and Row Vector (Outer Operations)
- $A$ has shape $[M, 1]$ (column vector).
- $B$ has shape $[1, N]$ (row vector).
- Both expand to produce a 2D matrix of shape $[M, N]$:
```matlab
A = [10; 20; 30];    % Shape: [3, 1]
B = [1, 2];          % Shape: [1, 2]

C = A + B;
% Dimension 1: max(3, 1) = 3
% Dimension 2: max(1, 2) = 2
% Result Shape: [3, 2]
% C = [
%   11, 12
%   21, 22
%   31, 32
% ]
```

### 3.3 Matrix and Row Vector
Adds the row vector to every row of the matrix:
```matlab
M = [1 2 3; 4 5 6];  % Shape: [2, 3]
v = [10 20 30];      % Shape: [1, 3]
C = M + v;
% Result Shape: [2, 3]
% C = [11 22 33; 14 25 36]
```

### 3.4 Matrix and Column Vector
Adds the column vector to every column of the matrix:
```matlab
M = [1 2 3; 4 5 6];  % Shape: [2, 3]
c = [10; 20];        % Shape: [2, 1]
C = M + c;
% Result Shape: [2, 3]
% C = [11 12 13; 24 25 26]
```

### 3.5 Higher-Dimensional Expansion
- Array $A$ has shape $[4, 1, 5]$.
- Array $B$ has shape $[1, 3, 5]$.
- Array $C = A .* B$ has shape $[4, 3, 5]$.

---

## 4. Empty Array Semantics with Broadcasting

In MATLAB, empty arrays (where at least one dimension is $0$) also follow implicit expansion rules:

1. **Singleton dimension with zero dimension**:
   - Array $A$ shape $[0, 1]$ and Array $B$ shape $[0, 5]$:
     - Dimension 1: $\max(0, 0) = 0$
     - Dimension 2: $\max(1, 5) = 5$
     - Result shape: $[0, 5]$
2. **Incompatible empty arrays**:
   - Array $A$ shape $[0, 2]$ and Array $B$ shape $[0, 3]$:
     - Dimension 2 has $2 \ne 3$ and neither is $1$.
     - Throws dimension mismatch error even though both arrays contain 0 elements.

---

## 5. Memory-Efficient Implementation Strategy in Rust

To execute broadcasting with zero unnecessary memory copies:

1. **Stride Trick**:
   - For a dimension of size $1$ that expands to size $D_k$, its memory stride along that dimension is set to **0**:
     $$\text{Stride}_k = 0$$
   - Incrementing the coordinate in that dimension does not advance the underlying pointer, allowing the single element to be read repeatedly without duplicating the memory buffer!
2. **Direct Output Allocation**:
   - Allocate the final output buffer of shape $[\max(D_{A1}, D_{B1}), \dots]$ directly once.
   - Iterate through coordinates $(i_1, \dots, i_k)$ using the computed strides for $A$ and $B$.
