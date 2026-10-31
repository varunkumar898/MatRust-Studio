# Contributing to MatRust Studio

Thank you for your interest in contributing to MatRust Studio / RustLab!

## Clean-Room Requirements & Legal Integrity
To maintain absolute legal and copyright integrity:
1. **Never copy or decompile proprietary code**: All MATLAB compatibility behavior must be derived solely from public documentation (such as MathWorks Help Center guides, ISO/IEC 29500-2 specifications, and open standards) or black-box observations.
2. **No GPL code**: MatRust Studio is licensed under permissive MIT / Apache-2.0. Do not incorporate or adapt code from GPL-licensed software (e.g. GNU Octave source code).
3. **No proprietary binaries or assets**: Never commit `.p` files, MathWorks `.mlapp` proprietary sample files, or copyrighted icons/fonts.

## Development Workflow
1. Fork and create a feature branch (`git checkout -b feat/my-feature`).
2. Adhere strictly to the workspace quality gates:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo clippy --workspace --all-targets --no-default-features -- -D warnings
   cargo test --workspace
   ```
3. Ensure no unwrap/panic/todo! macros are added in non-test library code.
4. Open a Pull Request detailing the changes, citing public reference sources for any new functions or syntax behaviors.
