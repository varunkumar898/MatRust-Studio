# RustLab Specification: MATLAB Grammar & Syntax

This specification defines the formal syntax, lexical rules, and evaluation semantics of the MATLAB-compatible language implemented by `rustlab`.

---

## 1. Lexical Structure

### 1.1 Character Encoding & Identifiers
- **Encoding**: UTF-8.
- **Case Sensitivity**: MATLAB is case-sensitive (`foo`, `Foo`, and `FOO` denote distinct identifiers).
- **Identifiers**:
  - Must start with an ASCII letter `[a-zA-Z]`.
  - Can be followed by any alphanumeric character or underscore `[a-zA-Z0-9_]`.
  - Maximum standard length in MATLAB is 63 characters (governed by `namelengthmax`).

### 1.2 Whitespace & Comments
- **Whitespace**: Space (`0x20`), Tab (`0x09`), and Carriage Return (`0x0D`) outside of string literals are ignored except as matrix element delimiters.
- **Line Comments**: Begin with `%` and continue to the end of the line.
- **Block Comments**:
  ```matlab
  %{
    Multi-line block comment.
    Must start with %{ on its own line (ignoring whitespace).
    Must end with %} on its own line.
  %}
  ```
- **Line Continuation**: Three or more consecutive periods (`...`) at the end of a line treat the following newline as whitespace, continuing the statement.

### 1.3 Statement Separation & Output Printing
- Statements are separated by:
  - Semicolon (`;`): Executes the statement and suppresses display of the result.
  - Comma (`,`) or Newline (`\n`): Executes the statement and automatically prints the result formatted as `ans = ...` (or `var = ...`).

---

## 2. Literals

### 2.1 Numeric Literals
- **Floats**: Default numeric literal is double-precision IEEE-754 (`f64`).
  - Standard decimal: `42`, `3.14159`, `.5`, `10.`
  - Scientific notation: `1.2e-4`, `3E+5`
- **Imaginary Numbers**: A numeric literal immediately suffixed with `i` or `j`:
  - `3i`, `2.5j`, `1e-3i`
  - Note: `3*i` performs multiplication with the variable `i`, whereas `3i` is a lexed complex scalar `0.0 + 3.0i`.
- **Binary/Hexadecimal**:
  - Hex: `0x1A2B`, `0XFF`
  - Binary: `0b1010`, `0B1100`

### 2.2 Text Literals
- **Character Vectors (Single Quotes)**: `'hello'` produces a `1 x 5` `char` array (row vector of UTF-16 code units).
  - Escaping single quote: `''` inside single quotes produces `'` (e.g. `'Don''t'` -> `Don't`).
- **Strings (Double Quotes)**: `"hello"` produces a `1 x 1` `string` scalar object.
  - Escaping double quote: `""` inside double quotes produces `"` (e.g. `"She said ""hi"""`).

---

## 3. Data Constructors

### 3.1 Matrices & Vectors (`[...]`)
- Elements on the same row are separated by spaces or commas (equivalent to horizontal concatenation `horzcat`):
  ```matlab
  A = [1 2 3];
  B = [1, 2, 3];    % identical to [1 2 3]
  ```
- Rows are separated by semicolons or newlines (equivalent to vertical concatenation `vertcat`):
  ```matlab
  C = [1 2; 3 4];
  D = [
      1, 2
      3, 4
  ];
  ```
- Empty matrix: `[]` has dimensions `0 x 0`.

### 3.2 Cell Arrays (`{...}`)
- Constructs an array of heterogeneous pointers/cells:
  ```matlab
  C = {1, "text", [1 2; 3 4]; 42, {1, 2}, true};
  ```

---

## 4. Operator Precedence & Associativity

The following table lists all operators supported by `rustlab` in descending order of precedence (highest to lowest):

| Precedence | Operator | Description | Associativity |
| :--- | :--- | :--- | :--- |
| **1 (Highest)** | `()`, `{}`, `.`, `.` | Parentheses, cell extraction, member access, dynamic field | Left-to-right |
| **2** | `.'`, `'` | Array transpose (`.'`), Complex conjugate transpose (`'`) | Left-to-right |
| **3** | `.^`, `^` | Elementwise power (`.^`), Matrix power (`^`) | **Right-to-left** |
| **4** | `+`, `-`, `~` | Unary plus, Unary minus, Logical NOT (`~`) | Right-to-left |
| **5** | `:` | Colon range construction (`start:step:end`) | Left-to-right |
| **6** | `.*`, `./`, `.\`, `*`, `/`, `\` | Elementwise and matrix multiplication, right/left division | Left-to-right |
| **7** | `+`, `-` | Binary addition and subtraction | Left-to-right |
| **8** | `<`, `<=`, `>`, `>=`, `==`, `~=` | Relational comparisons (elementwise) | Left-to-right |
| **9** | `&` | Elementwise logical AND | Left-to-right |
| **10** | `\|` | Elementwise logical OR | Left-to-right |
| **11** | `&&` | Short-circuit logical AND (scalar only) | Left-to-right |
| **12 (Lowest)** | `\|\|` | Short-circuit logical OR (scalar only) | Left-to-right |

---

## 5. Indexing Semantics

MATLAB indexing rules are among the most distinctive aspects of numerical computing.

### 5.1 1-Based Indexing
All array indices start at `1`. Indexing `A(0)` or indexing with negative numbers throws `IndexOutOfBounds` (`Index exceeds the number of array elements`).

### 5.2 Column-Major Memory Layout & Linear Indexing
Arrays are stored in column-major order (Fortran convention).
For an $M \times N$ matrix $A$, element $(r, c)$ resides at linear index:
$$\text{Linear Index} = (c - 1) \times M + r$$

- Subscript Indexing: `A(row, col)`
- Linear Indexing: `A(linear_idx)` treats any $N$-D array as a 1D column vector:
  ```matlab
  M = [10 30; 20 40];
  % M(1, 1) == 10
  % M(2, 1) == 20
  % M(1, 2) == 30
  % M(2, 2) == 40
  % Linear indexing: M(1)==10, M(2)==20, M(3)==30, M(4)==40
  ```

### 5.3 The `end` Keyword
Inside an indexing expression, `end` evaluates to the maximum valid index along that dimension (or the total length for linear indexing):
```matlab
v = [10, 20, 30, 40, 50];
x = v(end);         % 50
sub = v(2:end-1);   % [20, 30, 40]
```

### 5.4 Colon Operator (`:`)
- Range syntax: `start:end` (default step 1) or `start:step:end`.
- Full dimension slice: `A(:, 2)` selects all rows in column 2.
- Vectorization slice: `A(:)` returns all elements reshaped into a single column vector.

### 5.5 Logical Indexing
Indexing with a logical array selects all elements where the mask is `true`:
```matlab
A = [1 5 2 8 3];
mask = A > 3;       % [0 1 0 1 0] (logical)
B = A(mask);        % [5 8]
```

### 5.6 Cell Indexing (`()` vs `{}`)
- `C(1, 2)`: Returns a **new cell array** containing that slice.
- `C{1, 2}`: Extracts the **underlying content** stored within that cell.

### 5.7 Dynamic Array Growth on Assignment
Assigning to an index beyond current bounds automatically grows the array, filling unassigned intermediate positions with zeros (or empty strings/cells):
```matlab
x = [1, 2];
x(5) = 10;
% Result: x is [1, 2, 0, 0, 10]
```

---

## 6. Control Flow

### 6.1 Conditionals (`if ... elseif ... else ... end`)
A condition is considered `true` if it is non-empty and every element is non-zero (all-true semantics):
```matlab
if condition1
    statements1
elseif condition2
    statements2
else
    statements3
end
```

### 6.2 Loops
- **`for` loop**: Iterates over columns of an array expression:
  ```matlab
  for i = 1:5
      disp(i)
  end
  for col = [1 2 3; 4 5 6]
      % In each iteration, 'col' is a 2x1 column vector!
  end
  ```
- **`while` loop**:
  ```matlab
  while condition
      statements
  end
  ```
- **Jump statements**: `break` (exit inner loop), `continue` (advance to next iteration), `return` (exit function).

### 6.3 Pattern Matching (`switch ... case ... otherwise ... end`)
```matlab
switch expr
    case value1
        statements1
    case {value2, value3}  % Cell array matches any of the values
        statements2
    otherwise
        statements_default
end
```

### 6.4 Exception Handling (`try ... catch ... end`)
```matlab
try
    risky_operation()
catch ME
    disp(ME.identifier)
    disp(ME.message)
end
```

---

## 7. Functions & Scope

### 7.1 Function Syntax
```matlab
function [out1, out2] = my_function(in1, in2, varargin)
    % Function body
    out1 = in1 + in2;
    out2 = in1 * in2;
end
```
- Multiple output arguments: caller determines how many outputs are evaluated via `nargout`:
  ```matlab
  [a, b] = my_function(2, 3);
  a = my_function(2, 3);      % out2 is not calculated if gated by nargout
  ```
- Special variables:
  - `nargin`: Number of actual input arguments passed.
  - `nargout`: Number of expected output arguments requested.
  - `varargin`: Cell array holding excess inputs.
  - `varargout`: Cell array holding dynamic outputs.

### 7.2 Anonymous Functions
First-class function handles constructed via `@`:
```matlab
f = @(x, y) x.^2 + y.^2;
val = f(3, 4); % 25
```

### 7.3 Variable Scoping
- **Scripts**: Execute directly in the caller's workspace (no isolated stack frame). Variables created in scripts persist in the caller.
- **Functions**: Execute in their own private stack frame. Local variables disappear upon return.
- **`global` & `persistent`**:
  - `global X`: References a shared global symbol table.
  - `persistent X`: Retains value between successive invocations of the enclosing function.
