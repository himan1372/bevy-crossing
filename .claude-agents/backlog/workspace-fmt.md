# Workspace rustfmt (Philip chose option 1)
- [ ] Mark hand-laid decomp tables `#[rustfmt::skip]` so their row layout survives.
- [ ] Run `cargo fmt` on the whole workspace (one mechanical commit, no code changes).
- [ ] Gate passes: check → clippy -D warnings → fmt --check → test.
