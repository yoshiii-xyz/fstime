## Scope

- [ ] The change stays within bounded filesystem traversal profiling.
- [ ] No secrets or private QA artifacts are included.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --all-targets --locked`
- [ ] `cargo clippy --all-targets --all-features --locked -- -D warnings`
- [ ] `cargo test --all-targets --locked`
- [ ] `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --locked`
- [ ] `cargo package --locked`
- [ ] `cargo audit`
