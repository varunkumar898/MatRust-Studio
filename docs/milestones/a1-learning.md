# What I Learned: Milestone A1 (Language and Core Gaps)

Milestone A1 eliminates critical foundational gaps in the MATLAB language parser, numerical interpreter, and runtime standard library. This document reviews the core architectural challenges solved, explains the mechanics of each subsystem, and details the clean-room implementation strategies employed.

---

## 1. Nested `end` Context Stack

### The Challenge
In MATLAB, the token `end` is dual-purpose:
1. Block terminator (`if ... end`, `for ... end`, `function ... end`).
2. Relative subscript index representing the length of a dimension within an indexing expression (`A(2:end)`, `A(end, end)`).

When indexing expressions are nested, such as:
```matlab
A(1 : end - B(end))
```
the two `end` keywords refer to completely different dimensions:
- The inner `end` refers to the length of array `B`.
- The outer `end` refers to the dimension length of array `A`.
A single global or evaluator-wide `Option<IndexingContext>` fails because the inner evaluation overwrites the outer context.

### The Solution: An Evaluator Stack
In `rustlab-eval/src/eval.rs`, we introduced a re-entrant stack:
```rust
pub struct IndexingContext {
    pub current_dim_len: usize,
    pub total_len: usize,
}

pub struct Interpreter {
    // ...
    pub indexing_stack: Vec<IndexingContext>,
}
```
Whenever an indexing subscript `A(arg1, arg2, ...)` is evaluated:
1. For subscript $k$, compute dimension length $D_k$ (or total number of elements if 1D linear indexing).
2. Push `IndexingContext { current_dim_len: D_k, total_len }` onto `indexing_stack`.
3. Recursively call `self.eval_expr(arg_k)`.
4. Pop the context.

When `Expr::End` is evaluated, it resolves directly to `indexing_stack.last().unwrap().current_dim_len as f64`. If `indexing_stack` is empty, it raises error `MATLAB:badend`.

---

## 2. Lexical Closures and Anonymous Functions by Value

### MATLAB Semantics vs Traditional Scripting Languages
In MATLAB, anonymous functions capture free variables from the outer workspace **by value at definition time**:
```matlab
a = 10;
f = @(x) x + a;
a = 999;
f(5); % Evaluates to 15, NOT 1004!
```
Variables created or modified later in the caller's workspace do not alter the snapshot captured when `@(...)` was parsed and created.

### Clean-Room Rust Architecture
In `rustlab-math/src/value.rs`:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionHandle {
    Named(String),
    Anonymous {
        params: Vec<String>,
        body: rustlab_parse::Expr,
        closure_env: HashMap<String, Value>,
    },
}
```
When evaluating `Expr::AnonymousFunction { params, body, .. }`:
1. Find all identifiers in `body` that are not in `params`.
2. For each free variable present in `self.env`, snapshot a deep clone into `closure_env`.
3. When invoking the closure handle:
   - Instantiate a child environment `self.env.child_for_function("@anonymous")`.
   - Seed the child environment with `closure_env`.
   - Bind incoming actual arguments to `params`.
   - Evaluate `body`.

---

## 3. Boolean Mask Indexing and Masked Assignment

### The Dual Indexing Model
MATLAB supports:
- Integer subscripts: 1-based indexing (`A([1, 3])`).
- Logical masks: boolean arrays of identical size or compatible shape (`A(A > 2)`).

When a subscript evaluates to `Value::Logical(mask)`:
1. Mask Retrieval: `A(mask)` iterates through $A$ and selects each element where `mask[i] == true` in column-major order, producing a column vector.
2. Masked Assignment: `A(mask) = val`:
   - If `val` is scalar: all matching elements in $A$ are overwritten with `val`.
   - If `val` is an array: elements from `val` are drawn sequentially to populate the true slots in $A$.
Both `get_mask` and `set_mask` operate zero-copy across column-major contiguous memory slices in `DenseArray<T>`.

---

## 4. Format-Recycling Semantics (`sprintf`, `fprintf`)

### Specification
In standard C `sprintf`, each format specifier consumes exactly one argument. If arguments exceed specifiers, C ignores or errors.
In MATLAB:
1. An array argument is treated as a stream of elements in column-major order.
2. If the format string contains $K$ specifiers and the inputs supply $N > K$ values, MATLAB **recycles the entire format string** repeatedly until all elements are consumed.
   Example:
   ```matlab
   sprintf('%d ', [1 2 3 4])            % -> '1 2 3 4 '
   sprintf('%0.1f, %0.1f\n', [1 2; 3 4]) % -> "1.0, 3.0\n2.0, 4.0\n"
   ```

### Implementation
In `rustlab-stdlib/src/io_format.rs`:
1. Format parser separates literal tokens from format specifiers (`%d`, `%f`, `%s`, etc.).
2. Arguments are flattened in column-major order into a linear sequence of `FormattableItem`.
3. The format token stream is traversed in a loop, applying each specifier to the next item and appending literals, until all items have been formatted.

---

## 5. Scripts vs Function Files and Path Search

MATLAB distinguishes `.m` files based on their leading grammar construct:
- **Function file**: starts with `function ... end`. Executes in an isolated child stack frame.
- **Script file**: contains sequential statements without a top-level function header. Executes directly in the caller's workspace, modifying and inspecting caller variables.

In `Interpreter::resolve_and_load_mfile`:
1. Search configured paths (`self.search_paths`).
2. Parse `.m` source.
3. If `Stmt::Function` declarations exist, register them in `self.functions`.
4. If no function headers exist, execute statements sequentially in `self.env`.
5. Builtins `which`, `exist`, `addpath`, and `rmpath` inspect and manipulate the search path hierarchy dynamically.
