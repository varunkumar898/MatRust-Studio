## What changed?

<!-- Describe the change and link the issue. -->

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo test --workspace --no-default-features`
- [ ] Updated `docs/test-manifest.txt` and `docs/test-count.md` if test inventory/count changed.
- [ ] Updated relevant documentation and examples.
- [ ] Regenerated `crates/mlapp-ffi/include/mlapp.h` with cbindgen if the C ABI changed.
- [ ] Confirmed this contribution follows the clean-room policy and includes no proprietary files, copied source text/code, secrets, or private paths.
