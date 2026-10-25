# RustLab Error Catalog Specification

This catalog documents all runtime, mathematical, parsing, and standard library error identifiers in RustLab, along with their exact messages and upstream references (MathWorks MATLAB Documentation or GNU Octave Manual).

---

## 1. Mathematical & Array Operation Errors (`rustlab-math`)

| Identifier | Exact Message | Citation / Source URL |
| :--- | :--- | :--- |
| `MATLAB:integerTypeMismatch` | `Integers can only be combined with integers of the same class, or scalar doubles.` | [MathWorks: Arithmetic with Integer and Float](https://www.mathworks.com/help/matlab/matlab_prog/arithmetic-with-integer-and-float.html) |
| `MATLAB:subsasgn:dimensionMismatch` | `In an assignment  A(I) = B, the number of elements in B and I must be the same.` | [MathWorks: Subscripted Assignment](https://www.mathworks.com/help/matlab/math/matrix-indexing.html) |
| `MATLAB:dimagree` | `Array dimensions must match for binary array op or logical indexing.` / `Arrays have incompatible sizes for this operation` | [MathWorks: Compatible Array Sizes for Basic Operations](https://www.mathworks.com/help/matlab/matlab_prog/compatible-array-sizes-for-basic-operations.html) |
| `MATLAB:innerdim` | `Incorrect dimensions for matrix multiplication. Check that the number of columns in the first matrix ({cols_a}) matches the number of rows in the second matrix ({rows_b})` | [MathWorks: mtimes](https://www.mathworks.com/help/matlab/ref/mtimes.html) |
| `MATLAB:badsubscript` | `Index exceeds the number of array elements ({max_len}): requested 1-based index {index}` | [MathWorks: Array Indexing](https://www.mathworks.com/help/matlab/math/array-indexing.html) |
| `MATLAB:subscriptNotPositiveInteger` | `Array indices must be positive integers or logical values: invalid index {index}` | [MathWorks: Array Indexing](https://www.mathworks.com/help/matlab/math/array-indexing.html) |
| `MATLAB:square` | `Matrix must be square: shape is {rows}x{cols}` | [MathWorks: inv / det](https://www.mathworks.com/help/matlab/ref/inv.html) |
| `MATLAB:getReshapeDims:notSameNumel` | `To RESHAPE the number of elements must not change: from {original} to {requested}` | [MathWorks: reshape](https://www.mathworks.com/help/matlab/ref/reshape.html) |

---

## 2. Linear Algebra Standard Library Errors (`rustlab-stdlib::linalg`)

| Identifier | Exact Message | Citation / Source URL |
| :--- | :--- | :--- |
| `MATLAB:eig:notSquare` | `Matrix must be square.` | [MathWorks: eig](https://www.mathworks.com/help/matlab/ref/eig.html) |
| `MATLAB:eig:noConvergence` | `Eigenvalue decomposition failed to converge.` / `requires feature 'faer'` | [MathWorks: eig](https://www.mathworks.com/help/matlab/ref/eig.html) |
| `MATLAB:inv:notSquare` | `Matrix must be square.` | [MathWorks: inv](https://www.mathworks.com/help/matlab/ref/inv.html) |
| `MATLAB:det:notSquare` | `Matrix must be square.` | [MathWorks: det](https://www.mathworks.com/help/matlab/ref/det.html) |
| `MATLAB:trace:notSquare` | `Matrix must be square.` | [MathWorks: trace](https://www.mathworks.com/help/matlab/ref/trace.html) |
| `MATLAB:lu:notSquare` | `Matrix must be 2D.` | [MathWorks: lu](https://www.mathworks.com/help/matlab/ref/lu.html) |
| `MATLAB:chol:notSquare` | `Matrix must be square.` | [MathWorks: chol](https://www.mathworks.com/help/matlab/ref/chol.html) |
| `MATLAB:expm:notSquare` | `Matrix must be square.` | [MathWorks: expm](https://www.mathworks.com/help/matlab/ref/expm.html) |
| `MATLAB:svd:noConvergence` | `SVD computation failed to converge.` | [MathWorks: svd](https://www.mathworks.com/help/matlab/ref/svd.html) |
| `MATLAB:pinv:matrixRequired` | `Input must be a 2D matrix.` | [MathWorks: pinv](https://www.mathworks.com/help/matlab/ref/pinv.html) |
| `MATLAB:pinv:noConvergence` | `SVD failed to converge in pinv.` | [MathWorks: pinv](https://www.mathworks.com/help/matlab/ref/pinv.html) |
| `MATLAB:rank:noConvergence` | `SVD failed to converge in rank.` | [MathWorks: rank](https://www.mathworks.com/help/matlab/ref/rank.html) |
| `MATLAB:cond:noConvergence` | `SVD failed to converge in cond.` | [MathWorks: cond](https://www.mathworks.com/help/matlab/ref/cond.html) |
| `MATLAB:orth:noConvergence` | `SVD failed to converge in orth.` | [MathWorks: orth](https://www.mathworks.com/help/matlab/ref/orth.html) |
| `MATLAB:null:noConvergence` | `SVD failed to converge in null.` | [MathWorks: null](https://www.mathworks.com/help/matlab/ref/null.html) |

---

## 3. General Builtin & File I/O Errors (`rustlab-stdlib`)

| Identifier | Exact Message | Citation / Source URL |
| :--- | :--- | :--- |
| `MATLAB:minrhs` | `{fn_name} requires at least {N} input(s).` | [MathWorks: nargin](https://www.mathworks.com/help/matlab/ref/nargin.html) |
| `MATLAB:cat:badDim` | `Dimension argument must be a positive integer scalar.` | [MathWorks: cat](https://www.mathworks.com/help/matlab/ref/cat.html) |
| `MATLAB:size:badDim` | `Dimension argument must be a positive integer scalar.` | [MathWorks: size](https://www.mathworks.com/help/matlab/ref/size.html) |
| `MATLAB:diag:invalidDims` | `First input must be a 1D vector or 2D matrix.` | [MathWorks: diag](https://www.mathworks.com/help/matlab/ref/diag.html) |
| `MATLAB:fliplr:not2D` | `Input matrix must be 2-D.` | [MathWorks: fliplr](https://www.mathworks.com/help/matlab/ref/fliplr.html) |
| `MATLAB:flipud:not2D` | `Input matrix must be 2-D.` | [MathWorks: flipud](https://www.mathworks.com/help/matlab/ref/flipud.html) |
| `MATLAB:rot90:not2D` | `Input matrix must be 2-D.` | [MathWorks: rot90](https://www.mathworks.com/help/matlab/ref/rot90.html) |
| `MATLAB:strncmp:badN` | `Number of characters must be a non-negative integer scalar.` | [MathWorks: strncmp](https://www.mathworks.com/help/matlab/ref/strncmp.html) |
| `MATLAB:load:ioError` | `Cannot open MAT-file for reading: {err}` | [MathWorks: load](https://www.mathworks.com/help/matlab/ref/load.html) |
| `MATLAB:save:ioError` | `Cannot write to MAT-file: {err}` | [MathWorks: save](https://www.mathworks.com/help/matlab/ref/save.html) |
| `MATLAB:badFID` | `Invalid file identifier. Use fopen to generate a valid file identifier.` | [MathWorks: fclose](https://www.mathworks.com/help/matlab/ref/fclose.html) |
| `MATLAB:mat2str:invalidInput` | `mat2str input must be a numeric matrix.` | [MathWorks: mat2str](https://www.mathworks.com/help/matlab/ref/mat2str.html) |
| `MATLAB:error` | `{user_message}` | [MathWorks: error](https://www.mathworks.com/help/matlab/ref/error.html) |

---

## 4. Evaluation & Parser Errors (`rustlab-eval` / `rustlab-parse`)

| Type | Identifier / Description | Citation / Source URL |
| :--- | :--- | :--- |
| `UndefinedSymbol` | `Unrecognized function or variable '{name}'` | [MathWorks: Unrecognized function or variable](https://www.mathworks.com/help/matlab/matlab_prog/common-syntax-errors.html) |
| `TooManyInputs` | `Too many input arguments: expected at most {expected}, received {found}` | [MathWorks: nargin](https://www.mathworks.com/help/matlab/ref/nargin.html) |
| `TooManyOutputs` | `Too many output arguments: function provides at most {expected}, requested {found}` | [MathWorks: nargout](https://www.mathworks.com/help/matlab/ref/nargout.html) |
| `InvalidCondition` | `Invalid loop or branch condition` | [MathWorks: if](https://www.mathworks.com/help/matlab/ref/if.html) |
| `ParseError` | `Parse error: unexpected token / syntax error` | Unverified internal parser diagnostic |
