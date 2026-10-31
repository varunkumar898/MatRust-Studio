# RustLab Specification: Error Handling & Diagnostics

This specification defines the error hierarchy, canonical MATLAB error identifiers (`MException`), stack trace reporting, and diagnostic messaging for `rustlab`.

---

## 1. Error Architecture & `MException` Representation

In MATLAB, runtime exceptions are captured in an `MException` object containing:
1. **Identifier (`identifier`)**: Structured string in the format `Component:Subsystem:Mnemonic` (e.g. `MATLAB:dimagree`).
2. **Message (`message`)**: A human-readable diagnostic explaining what went wrong and how to fix it.
3. **Call Stack (`stack`)**: An ordered list of stack frames containing:
   - `file`: Path to the executing script or function.
   - `name`: Name of the function or `<script>` block.
   - `line`: Source line number where the fault occurred.
4. **Cause (`cause`)**: Optional chained exception causing this error.

In `rustlab`, this maps directly to an idiomatic Rust error type using `thiserror`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RustLabError {
    pub identifier: String,
    pub message: String,
    pub stack: Vec<StackFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    pub file: String,
    pub function_name: String,
    pub line: usize,
    pub column: usize,
}
```

---

## 2. Canonical MATLAB Error Identifiers

To guarantee full compatibility, `rustlab` adheres to the official MATLAB error identifiers and message conventions:

### 2.1 Dimension & Matrix Shape Errors
| Identifier | Description | Canonical Error Message |
| :--- | :--- | :--- |
| `MATLAB:dimagree` | Binary operation broadcasting mismatch | `Arrays have incompatible sizes for this operation.` |
| `MATLAB:innerdim` | Matrix multiplication dimension conflict | `Incorrect dimensions for matrix multiplication. Check that the number of columns in the first matrix matches the number of rows in the second matrix.` |
| `MATLAB:square` | Non-square matrix in linear solve / inverse | `Matrix must be square.` |
| `MATLAB:catenate:dimensionMismatch` | Incompatible dimensions in concatenation | `Dimensions of arrays being concatenated are not consistent.` |

### 2.2 Indexing & Access Errors
| Identifier | Description | Canonical Error Message |
| :--- | :--- | :--- |
| `MATLAB:badsubscript` | Index exceeds array bounds | `Index exceeds the number of array elements (%d).` |
| `MATLAB:subscriptNotPositiveInteger` | Non-positive or non-integer index | `Array indices must be positive integers or logical values.` |
| `MATLAB:nonExistentField` | Accessing missing struct field | `Reference to non-existent field '%s'.` |
| `MATLAB:cellRefFromNonCell` | Using `{}` on a non-cell array | `Cell contents reference from a non-cell array object.` |
| `MATLAB:dotRefOnNonStruct` | Using `.` on a non-struct value | `Expected scalar struct or object; got %s.` |

### 2.3 Variable & Function Resolution
| Identifier | Description | Canonical Error Message |
| :--- | :--- | :--- |
| `MATLAB:UndefinedFunction` | Unbound symbol or missing function | `Unrecognized function or variable '%s'.` |
| `MATLAB:minrhs` | Insufficient input arguments | `Not enough input arguments.` |
| `MATLAB:TooManyInputs` | Excess input arguments | `Too many input arguments.` |
| `MATLAB:TooManyOutputs` | Excess output arguments requested | `Too many output arguments.` |

### 2.4 Syntax & Parser Errors
| Identifier | Description | Canonical Error Message |
| :--- | :--- | :--- |
| `MATLAB:m_missing_operator` | Syntax parse error | `Invalid expression. Check for missing multiplication operator, missing or unbalanced delimiters, or other syntax error.` |
| `MATLAB:unbalanced_parens` | Missing closing parenthesis | `Character vector is missing closing delimiter: ')'.` |
| `MATLAB:unbalanced_brackets` | Missing closing bracket | `Character vector is missing closing delimiter: ']'.` |

---

## 3. Warning System

Warnings report non-fatal conditions without unwinding the stack:
- Triggered via `warning(identifier, message)`.
- Can be globally or selectively enabled, disabled, or treated as errors (`warning('error', 'MATLAB:singularMatrix')`).
- Canonical warning:
  - `MATLAB:singularMatrix`: `Matrix is singular to working precision.`
  - `MATLAB:nearlySingularMatrix`: `Matrix is close to singular or badly scaled. Results may be inaccurate.`
