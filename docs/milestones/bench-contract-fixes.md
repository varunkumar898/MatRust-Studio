# Milestone: Benchmark Contract Fixes

**Branch:** `gui/polish-1` → create `bench/contract-fixes` from current HEAD
**Scope:** Fix four benchmark contract misses from BACKLOG §3:
1. `movmean` (1e6 doubles, k=3): MISSED at 16.739 ms vs ~10 ms target
2. Interpreter `A + B` (1e6 doubles): MISSED at 3.533 ms vs ≤3.0 ms target
3. `cummax` (1e6 doubles): UNSTABLE at 11.556 ms vs ~10 ms target
4. `zscore` (1e6 doubles): MARGINAL at 10.197 ms vs ~10 ms target

## 1. Root Causes (from BACKLOG §3)

### movmean
- Running-sum algorithm introduces periodic exact recomputations every 1024 steps
- Finite-window boundary checks add branching per element
- Slice-based window management (`&vals[(i-left)..=(i+right)]`) adds allocation/index overhead
- **Fix**: Remove periodic recomputation in steady state (finite windows never drift meaningfully); remove slice slicing; use direct raw indexing

### Interpreter A + B
- AST tree-walk evaluation: dynamic dispatch, environment hash map lookups, allocation checks
- **Fix**: Add a specialized tight eval path for simple binary assignment `C = A + B` that bypasses environment lookups and uses direct borrowed arithmetic

### cummax
- Memory bandwidth variability, page fault latency, thread scheduling
- **Fix**: Unroll loop by 4 elements to hide memory latency during sequential write pass

### zscore
- Two-pass algorithm traverses 8 MB array twice
- Std dev division and final normalization pass
- **Fix**: Fuse normalization into variance computation where possible; further unroll

## 2. Error IDs
No new error IDs. All changes are internal performance optimizations.

## 3. Test Plan
- Existing golden tests for movmean, cummax, zscore must still pass (correctness unchanged)
- New regression benchmarks in `crates/rustlab-cli/src/bench.rs` bench_b21_data_and_csv
- Benchmark contract: movmean ≤10 ms, A+B ≤3.0 ms, cummax ≤10 ms stable, zscore ≤10 ms

## 4. Risks
- Removing periodic recomputation could cause floating-point drift in pathological cases; acceptable since MATLAB itself does not guarantee exact reproducibility across runs
- Specialized eval path must handle type promotion correctly (e.g., int32 + double → double)

## 5. Implementation Steps (R15: ~150 lines each)
1. Design note + baseline benchmark run from cargo clean
2. Fix movmean steady-state loop (remove recompute, remove slices)
3. Test movmean correctness + benchmark
4. Fix cummax loop unrolling (4x)
5. Test cummax correctness + benchmark
6. Fix zscore pass fusion + unrolling
7. Test zscore correctness + benchmark
8. Fix interpreter A+B tight eval path
9. Test interpreter correctness + benchmark
10. Full from-clean verify + benchmarks
11. Update docs/STATE.md, commit, tag
