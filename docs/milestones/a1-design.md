# Milestone A1 Design Note: Language & Core Gaps

**Milestone:** A1  
**Author:** RustLab Systems Architecture Team  
**Objective:** Close essential language and runtime core gaps: evaluator context stack for `end`, boolean mask indexing and assignment, IEEE-754 NaN/Inf semantics, first-class function handles with lexical closures, functional iterators, calling conventions (`nargin`/`varargin`), diagnostics (`error`/`assert`), formatted I/O (`sprintf`/`disp`), string/regex library, and script vs function file path resolution.

---

## 1. Architectural Decisions & Alternatives

### 1.1 Evaluator Context Stack for Dynamic `end`
- **Problem:** When an expression contains nested indexing like `A(1:end - B(end))`, passing a single optional `idx_ctx: Option<&IndexingContext>` down the call tree causes the outer `end` (associated with `A`) to be shadowed or lost when the inner index `B(end)` is evaluated.
- **Solution:** Maintain an explicit `indexing_stack: Vec<IndexingContext>` on the `Interpreter`.
  - When evaluating an index operation `callee(args)`:
    - Step 1: Evaluate `callee` with the current stack.
    - Step 2: Push `IndexingContext` for each argument dimension onto `indexing_stack`.
    - Step 3: Evaluate the argument expression.
    - Step 4: Pop from `indexing_stack`.
  - `Expr::End`: Resolves against `indexing_stack.last()`.
  - Nested expressions like `B(end)` push and pop their own context, so that when execution returns to `A`'s argument expression, `A`'s context is restored at the top of the stack.
  - Supports:
    - `A(1:end - B(end))`
    - `A(end, end)`
    - `C{end}`
    - `s.field(end)`

### 1.2 Boolean Mask Indexing & Masked Assignment
- **Semantics:**
  - In MATLAB, `A(mask)` where `mask` is a `logical` array of the same size as `A` (or a vector matching total element count) extracts all elements where `mask[i] == true` into a column vector.
  - Masked assignment `A(mask) = val`:
    - If `val` is scalar: sets all masked positions in `A` to `val`.
    - If `val` is a vector with length equal to `nnz(mask)`: assigns values sequentially in column-major order.
- **Implementation:**
  - Extended in `rustlab-math/src/array.rs` (`get_mask`, `set_mask`) and integrated into `rustlab-eval` (`eval_index_or_call` and `eval_assignment`).

### 1.3 IEEE-754 NaN/Inf Comparison & Builtins
- **Semantics:**
  - In IEEE-754 and MATLAB: `NaN == NaN` is `false`, `NaN < x` is `false`, `NaN > x` is `false`.
  - Only `NaN ~= NaN` is `true`.
  - Builtins: `isnan(x)`, `isinf(x)`, `isfinite(x)` return logical arrays of identical shape.
- **Implementation:**
  - Audit equality operators in `Value::eq` and `Value::not_eq` to preserve IEEE-754 semantics for floats.
  - Register `isnan`, `isinf`, `isfinite` builtins in `rustlab-stdlib`.

### 1.4 First-Class `Value::FunctionHandle` & Lexical Closures
- **Problem:** Currently, anonymous functions are stored under a static string key `"@anonymous"`, causing collisions and preventing lexical closures.
- **Solution:**
  - Add variant to `Value`:
    ```rust
    Value::FunctionHandle(Arc<FunctionHandle>)
    ```
  - `FunctionHandle` enum:
    - `Named(String)`: e.g. `@sin`, `@my_func`.
    - `Anonymous { params: Vec<String>, body: Expr, closure_env: HashMap<String, Value> }`.
  - At creation time (`@(x) x.^2 + a`):
    - Identify free variables in `body` not in `params`.
    - Snapshot their current values from `Interpreter.env` into `closure_env` by value.
    - Subsequent mutations to `a` in the outer workspace do not affect the closure, exactly matching MATLAB semantics.

### 1.5 Functional Iterators (`cellfun`, `arrayfun`, `structfun`)
- **Semantics:**
  - `cellfun(@fn, C, 'UniformOutput', false/true, 'ErrorHandler', @err_fn)`: applies `@fn` to each cell in `C`.
  - `arrayfun(@fn, A, ...)`: applies `@fn` to each scalar element in `A`.
  - `structfun(@fn, S, ...)`: applies `@fn` to each field in scalar struct `S`.
- **Implementation:**
  - Parse `'UniformOutput'` (default `true`) and optional `'ErrorHandler'` name-value pairs.
  - If `UniformOutput == true`, concatenate scalar outputs into a dense numeric or logical array; if `false`, pack into a cell array.

### 1.6 Calling Conventions (`nargin`, `nargout`, `varargin`, `varargout`)
- **Semantics:**
  - `nargin` and `nargout` builtins query the active function stack frame.
  - If a function signature declares `varargin` as the last input parameter, all surplus arguments are collected into a `Value::Cell`.
  - If a function signature declares `varargout` as the last output parameter, the return cell array is unpacked into multiple caller variables.

### 1.7 Diagnostics (`error`, `warning`, `assert`)
- **Semantics:**
  - `error(id, msg)` and `error(msg)` raise a structured `EvalError::UserError`.
  - In `try ... catch err`, `err` is bound to a `Value::Struct` containing fields `.identifier`, `.message`, and `.stack`.
  - `warning(msg)` prints diagnostic to stderr without terminating execution.
  - `assert(cond, id, msg)` validates condition and throws error if false.

### 1.8 Formatted I/O & Text Recycling (`sprintf`, `fprintf`, `disp`, `num2str`, `mat2str`)
- **Semantics:**
  - Format-recycling: in MATLAB, `sprintf('%d %d\n', [1, 2, 3, 4])` recycles format string `"%d %d\n"` across columns, yielding:
    ```
    1 2
    3 4
    ```
  - Standard specifiers: `%d`, `%i`, `%u`, `%f`, `%e`, `%g`, `%s`, `%c`, `%x`.
  - `num2str(A)` and `mat2str(A)`.

### 1.9 String & Regex Library
- **Semantics:**
  - Functions: `strsplit`, `strjoin`, `strrep`, `strtrim`, `upper`, `lower`, `strcmp`, `strcmpi`, `strncmp`, `contains`, `startsWith`, `endsWith`, `regexp`, `regexprep`.
  - Uses the `regex` crate for pattern matching.
  - Difference from MATLAB: Rust `regex` uses standard Perl/PCRE-like syntax, whereas MATLAB supports special conditional patterns and lookbehinds. Documented in `docs/spec/regex.md`.

### 1.10 Script vs Function File & Path Resolution
- **Semantics:**
  - Search path list: `path` in `Interpreter`.
  - Commands: `addpath(dir)`, `rmpath(dir)`, `which(name)`, `exist(name)`.
  - Dynamic loading: when an unresolved identifier `foo(...)` is called, search paths for `foo.m`:
    - If `foo.m` begins with `function ...`, parse and execute in isolated child scope.
    - If `foo.m` is a script, execute statements in caller scope.
