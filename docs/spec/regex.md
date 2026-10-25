# MATLAB vs Rust Regex Specification

RustLab uses the high-performance Rust `regex` crate as its underlying regular expression engine. The Rust `regex` crate compiles regular expressions to deterministic finite automata (DFAs) that execute in worst-case linear time $O(n)$ with guaranteed limits on memory usage.

This document specifies the behavior of `regexp` and `regexprep`, noting the syntactic and semantic differences between MATLAB's PCRE engine and Rust's engine.

---

## 1. Supported Syntax Parity

Both MATLAB and RustLab support:
- Character classes: `\d` (digit), `\D`, `\s` (whitespace), `\S`, `\w` (word char), `\W`, `[a-z]`, `[^0-9]`.
- Quantifiers: `*`, `+`, `?`, `{m,n}`, `{m,}`, `{m}`, along with non-greedy variants `*?`, `+?`, `??`, `{m,n}?`.
- Anchors: `^` (start of line/text), `$` (end of line/text), `\b` (word boundary), `\B`.
- Alternation: `foo|bar`.
- Sub-expressions / Capturing Groups: `(abc)`.
- Non-capturing groups: `(?:abc)`.
- Named capture groups: `(?<name>pattern)` or `(?P<name>pattern)`.
- Flags: `(?i)` for case-insensitivity, `(?m)` for multi-line mode, `(?s)` for dot matches all.

---

## 2. Key Differences and RustLab Handling

| Feature | MATLAB (PCRE) | RustLab (`regex` crate) | Recommendation / Workaround |
| :--- | :--- | :--- | :--- |
| **Lookaround Assertions** | Supported: `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)` | Not supported (to preserve $O(n)$ DFA execution guarantee) | Use multiple regex passes or matching anchors where possible. |
| **Backreferences in Pattern** | Supported: `(cat)\1` | Not supported | Match group and test equivalence in MATLAB script. |
| **Replacement Backreferences** | `$1`, `\1` in replacement text | `$1`, `${1}`, `$name` | Both `$1` and `${name}` are fully supported in `regexprep`. |
| **Recursive / Balancing Patterns**| `(?R)` supported | Not supported | Parse hierarchical structures using grammar parsers. |
| **Execution Time Guarantee** | Potential exponential backtracking catastrophic penalty $O(2^n)$ | Guaranteed linear $O(n)$ scan speed | Eliminates ReDoS vulnerabilities in data processing. |

---

## 3. Output Modes in `regexp`

- `idx = regexp(str, expr)`: Returns a 1-based double row vector of starting indices of matches.
- `matches = regexp(str, expr, 'match')`: Returns a $1 \times K$ cell array of character vectors for each match.
- `tokens = regexp(str, expr, 'tokens')`: Returns a $1 \times K$ cell array, each holding captured group tokens.
- `names = regexp(str, expr, 'names')`: Returns a `struct` with fields matching each named capture group `(?<name>...)`.
- `res = regexp(..., 'once')`: Restricts the search to only the first occurrence.

## 4. Replacement with `regexprep`

- `s = regexprep(str, expr, rep)`: Replaces all matches with replacement pattern `rep`.
- `s = regexprep(str, expr, rep, 'once')`: Replaces only the first match.
