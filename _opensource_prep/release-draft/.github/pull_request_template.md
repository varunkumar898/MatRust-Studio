## Summary

<!-- Explain the change and link the related issue. -->

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`
- [ ] Relevant tests pass (`cargo test --workspace` when appropriate)
- [ ] Reviewed test-name additions/removals and updated `docs/test-manifest.txt` if needed
- [ ] Updated user or developer documentation
- [ ] Regenerated `mlapp.h` if the C API changed

## Clean-room and privacy

- [ ] No proprietary MathWorks files, real `.mlapp`/`.slx` samples, GPL implementation code, secrets, or private data were added
- [ ] Compatibility behavior is based on permitted public sources or analytical tests; sources are noted
- [ ] Any substantial AI-generated contribution was reviewed and disclosed
