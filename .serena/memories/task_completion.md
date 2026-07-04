- For Rust code changes, completion gate is:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features`
  3. `cargo test`
- If touching dependencies or security-sensitive paths, also run `cargo audit`.
- If changes affect long-running/integration behavior, consider `cargo nextest run --all --all-features`.
- For presenter/output changes, compare generated example artifacts with `assets/` fixtures.