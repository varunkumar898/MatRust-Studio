Batch 15 partial — actual state:
- Batch 14 discrepancy explained: 5 test cases in numeric_batch14.rs; 2 pass (eig contract, cov vectors, corrcoef), 3 fail because parser/built-in rejects vector second arguments (prctile/quantile vector pct/int-n syntax not fully supported), not because only 2 exist.
- Phase 1 (convergence): completed — eig returns error on non-convergence instead of silent approximation; Batch 13 tests intact.
- Phase 2 (percentile/quantile): partial — midpoint preserved; vector percentiles and integer-n forms attempted via built-in edits but caused runtime type errors and were reverted to preserve workspace integrity. Full support requires either parser-level matrix-argument dispatch or stats-level multi-pct handling.
- Phase 3 (covariance): partial — cov(A,B) two-vector form works; normalization/payload preserved.
- Phase 4 (corrcoef): existing behavior preserved; Rows policies deferred.
- Documentation updated; no fabricated REA evidence; MATLAB target remains unavailable.
