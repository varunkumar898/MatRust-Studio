# RustLab Error and Warning Catalog Specification (B2.1.1)

This catalog documents all runtime, mathematical, parsing, standard library error identifiers, and
diagnostic warnings in RustLab, along with their exact messages and upstream references (MathWorks
MATLAB Documentation or GNU Octave Manual).

**Verification policy (A3.1.5 / B2.1.1).** Every identifier below carries either a source URL or the
literal word **unverified** in the *Verification* column. `unverified` means that no primary MathWorks
page was found that prints the exact identifier string; the linked page documents the underlying rule
or the closest public behaviour. A URL in the *Citation* column does not by itself prove the
identifier string.

**Integer-mismatch identifier, settled once (A3.1.5).** RustLab uses exactly one identifier:
`MATLAB:integerTypeMismatch`. It is produced by `MathError::integer_class_mismatch()` and emitted by
every mixed-integer-class arithmetic path. No alias (`MATLAB:mixedIntegerTypes`,
`MATLAB:integerClassMismatch`, etc.) exists in the codebase. The message text is from the MathWorks
integer-arithmetic rule below; the exact identifier string itself is **unverified**.

---

## 1. Mathematical & Array Operation Errors (`rustlab-math`)

| Identifier | Exact Message | Citation / Source URL | Verification |
| :--- | :--- | :--- | :--- |
| `MATLAB:integerTypeMismatch` | `Integers can only be combined with integers of the same class, or scalar doubles.` | [MathWorks: Arithmetic with Integer and Float](https://www.mathworks.com/help/matlab/matlab_prog/arithmetic-with-integer-and-float.html) | unverified (identifier); rule documented |
| `MATLAB:subsassigndimmismatch` | `In an assignment  A(I) = B, the number of elements in B and I must be the same.` | [MathWorks: Matrix Indexing](https://www.mathworks.com/help/matlab/math/matrix-indexing.html) | UNVERIFIED (identifier); documented assignment-count rule |
| `MATLAB:subsasgn:dimensionMismatch` | `Subscripted assignment dimension mismatch.` | [MathWorks: Matrix Indexing](https://www.mathworks.com/help/matlab/math/matrix-indexing.html) | UNVERIFIED (numeric slice-assignment path) |
| `MATLAB:dimagree` | `Array dimensions must match for binary array op or logical indexing.` | [MathWorks: Compatible Array Sizes](https://www.mathworks.com/help/matlab/matlab_prog/compatible-array-sizes-for-basic-operations.html) | UNVERIFIED (settled logical-mask size mismatch) |
| `MATLAB:innerdim` | `Incorrect dimensions for matrix multiplication. Check that the number of columns in the first matrix ({cols_a}) matches the number of rows in the second matrix ({rows_b})` | [MathWorks: mtimes](https://www.mathworks.com/help/matlab/ref/mtimes.html) | unverified |
| `MATLAB:badsubscript` | `Index exceeds the number of array elements ({max_len}): requested 1-based index {index}` | [MathWorks: Array Indexing](https://www.mathworks.com/help/matlab/math/array-indexing.html) | unverified |
| `MATLAB:subscriptNotPositiveInteger` | `Array indices must be positive integers or logical values: invalid index {index}` | [MathWorks: Array Indexing](https://www.mathworks.com/help/matlab/math/array-indexing.html) | unverified |
| `MATLAB:square` | `Matrix must be square: shape is {rows}x{cols}` | [MathWorks: inv](https://www.mathworks.com/help/matlab/ref/inv.html) | unverified |
| `MATLAB:getReshapeDims:notSameNumel` | `To RESHAPE the number of elements must not change: from {original} to {requested}` | [MathWorks: reshape](https://www.mathworks.com/help/matlab/ref/reshape.html) | unverified |

**Settled mask-count identifier (G0.1-VERIFY).** `A(mask) = B` with a
non-scalar `B` uses exactly one identifier and one message:
`MATLAB:subsassigndimmismatch` /
`In an assignment  A(I) = B, the number of elements in B and I must be the same.`
The identifier string is **UNVERIFIED** against a MathWorks page; the message
is the documented assignment-count rule. The separate numeric slice-assignment
path keeps `MATLAB:subsasgn:dimensionMismatch` and its own
`Subscripted assignment dimension mismatch.` message. The logical-mask size
mismatch (mask shape not equal to the array shape) is a different error:
`MATLAB:dimagree` with the single message shown above. No claim of online
MathWorks identifier verification is made.

---

## 2. Linear Algebra Standard Library Errors (`rustlab-stdlib::linalg`)

| Identifier | Exact Message | Citation / Source URL | Verification |
| :--- | :--- | :--- | :--- |
| `MATLAB:eig:notSquare` | `Matrix must be square.` | [MathWorks: eig](https://www.mathworks.com/help/matlab/ref/eig.html) | unverified |
| `MATLAB:eig:noConvergence` | `Eigenvalue decomposition failed to converge.` / `requires feature 'faer'` | [MathWorks: eig](https://www.mathworks.com/help/matlab/ref/eig.html) | unverified |
| `MATLAB:inv:notSquare` | `Matrix must be square.` | [MathWorks: inv](https://www.mathworks.com/help/matlab/ref/inv.html) | unverified |
| `MATLAB:det:notSquare` | `Matrix must be square.` | [MathWorks: det](https://www.mathworks.com/help/matlab/ref/det.html) | unverified |
| `MATLAB:trace:notSquare` | `Matrix must be square.` | [MathWorks: trace](https://www.mathworks.com/help/matlab/ref/trace.html) | unverified |
| `MATLAB:lu:notSquare` | `Matrix must be 2D.` | [MathWorks: lu](https://www.mathworks.com/help/matlab/ref/lu.html) | unverified |
| `MATLAB:chol:notSquare` | `Matrix must be square.` | [MathWorks: chol](https://www.mathworks.com/help/matlab/ref/chol.html) | unverified |
| `MATLAB:expm:notSquare` | `Matrix must be square.` | [MathWorks: expm](https://www.mathworks.com/help/matlab/ref/expm.html) | unverified |
| `MATLAB:svd:noConvergence` | `SVD computation failed to converge.` | [MathWorks: svd](https://www.mathworks.com/help/matlab/ref/svd.html) | unverified |
| `MATLAB:pinv:matrixRequired` | `Input must be a 2D matrix.` | [MathWorks: pinv](https://www.mathworks.com/help/matlab/ref/pinv.html) | unverified |
| `MATLAB:pinv:noConvergence` | `SVD failed to converge in pinv.` | [MathWorks: pinv](https://www.mathworks.com/help/matlab/ref/pinv.html) | unverified |
| `MATLAB:rank:noConvergence` | `SVD failed to converge in rank.` | [MathWorks: rank](https://www.mathworks.com/help/matlab/ref/rank.html) | unverified |
| `MATLAB:cond:noConvergence` | `SVD failed to converge in cond.` | [MathWorks: cond](https://www.mathworks.com/help/matlab/ref/cond.html) | unverified |
| `MATLAB:orth:noConvergence` | `SVD failed to converge in orth.` | [MathWorks: orth](https://www.mathworks.com/help/matlab/ref/orth.html) | unverified |
| `MATLAB:null:noConvergence` | `SVD failed to converge in null.` | [MathWorks: null](https://www.mathworks.com/help/matlab/ref/null.html) | unverified |

---

## 3. General Builtin & File I/O Errors (`rustlab-stdlib`, `rustlab-data`)

| Identifier | Exact Message | Citation / Source URL | Verification |
| :--- | :--- | :--- | :--- |
| `MATLAB:minrhs` | `{fn_name} requires at least {N} input(s).` | [MathWorks: nargin](https://www.mathworks.com/help/matlab/ref/nargin.html) | unverified |
| `MATLAB:maxrhs` | `Too many input arguments.` | [MathWorks: nargin](https://www.mathworks.com/help/matlab/ref/nargin.html) | UNVERIFIED (identifier) |
| `MATLAB:cat:badDim` | `Dimension argument must be a positive integer scalar.` | [MathWorks: cat](https://www.mathworks.com/help/matlab/ref/cat.html) | unverified |
| `MATLAB:size:badDim` | `Dimension argument must be a positive integer scalar.` | [MathWorks: size](https://www.mathworks.com/help/matlab/ref/size.html) | unverified |
| `MATLAB:diag:invalidDims` | `First input must be a 1D vector or 2D matrix.` | [MathWorks: diag](https://www.mathworks.com/help/matlab/ref/diag.html) | unverified |
| `MATLAB:fliplr:not2D` | `Input matrix must be 2-D.` | [MathWorks: fliplr](https://www.mathworks.com/help/matlab/ref/fliplr.html) | unverified |
| `MATLAB:flipud:not2D` | `Input matrix must be 2-D.` | [MathWorks: flipud](https://www.mathworks.com/help/matlab/ref/flipud.html) | unverified |
| `MATLAB:rot90:not2D` | `Input matrix must be 2-D.` | [MathWorks: rot90](https://www.mathworks.com/help/matlab/ref/rot90.html) | unverified |
| `MATLAB:strncmp:badN` | `Number of characters must be a non-negative integer scalar.` | [MathWorks: strncmp](https://www.mathworks.com/help/matlab/ref/strncmp.html) | unverified |
| `MATLAB:load:ioError` | `Cannot open MAT-file for reading: {err}` | [MathWorks: load](https://www.mathworks.com/help/matlab/ref/load.html) | unverified |
| `MATLAB:save:ioError` | `Cannot write to MAT-file: {err}` | [MathWorks: save](https://www.mathworks.com/help/matlab/ref/save.html) | unverified |
| `MATLAB:badFID` | `Invalid file identifier. Use fopen to generate a valid file identifier.` | [MathWorks: fclose](https://www.mathworks.com/help/matlab/ref/fclose.html) | unverified |
| `MATLAB:mat2str:invalidInput` | `mat2str input must be a numeric matrix.` | [MathWorks: mat2str](https://www.mathworks.com/help/matlab/ref/mat2str.html) | unverified |
| `RUSTLAB:readmatrix:fileNotFound` | `Failed to open file '{filename}': {err}` | RustLab-specific file error | unverified |
| `RUSTLAB:readmatrix:raggedData` | `Row {line} has {found} fields, expected {expected}.` | RustLab-specific ragged line error | unverified |
| `RUSTLAB:readmatrix:badPath` | `Filename must be a character vector or string scalar.` | RustLab-specific path error | unverified |
| `RUSTLAB:data:badInput` | `{fn_name} requires a 2-D numeric array.` / `Percentile/quantile values must be finite.` | RustLab-specific data error | unverified |
| `RUSTLAB:data:badOption` | `Unsupported option '{name}'.` | RustLab-specific option error | unverified |
| `MATLAB:error` | `{user_message}` | [MathWorks: error](https://www.mathworks.com/help/matlab/ref/error.html) | unverified |

---

## 4. Evaluation & Parser Errors (`rustlab-eval` / `rustlab-parse`)

| Type | Identifier / Description | Citation / Source URL | Verification |
| :--- | :--- | :--- | :--- |
| `UndefinedSymbol` | `Unrecognized function or variable '{name}'` | [MathWorks: Common Syntax Errors](https://www.mathworks.com/help/matlab/matlab_prog/common-syntax-errors.html) | unverified |
| `TooManyInputs` | `Too many input arguments: expected at most {expected}, received {found}` | [MathWorks: nargin](https://www.mathworks.com/help/matlab/ref/nargin.html) | unverified |
| `TooManyOutputs` | `Too many output arguments: function provides at most {expected}, requested {found}` | [MathWorks: nargout](https://www.mathworks.com/help/matlab/ref/nargout.html) | unverified |
| `InvalidCondition` | `Invalid loop or branch condition` | [MathWorks: if](https://www.mathworks.com/help/matlab/ref/if.html) | unverified |
| `ParseError` | `Parse error: unexpected token / syntax error` | No public source for RustLab's internal parser diagnostic | unverified |

---

## 5. System Diagnostic Warnings (`rustlab-stdlib::linalg`)

| Warning Identifier | Exact Warning Message | Action Taken | Citation / Source URL | Verification |
| :--- | :--- | :--- | :--- | :--- |
| `MATLAB:singularMatrix` | `Warning: Matrix is singular to working precision.` | Warning printed to stderr; returns matrix of `Inf` values without halting execution. Applies to both `inv(A)` and `A \ b`. | [MathWorks: inv](https://www.mathworks.com/help/matlab/ref/inv.html)<br>[MathWorks: mldivide](https://www.mathworks.com/help/matlab/ref/mldivide.html) | verified (exact message text documented in MathWorks MATLAB references; identifier unverified) |
