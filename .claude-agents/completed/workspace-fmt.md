# Workspace rustfmt (Philip chose option 1)
- [x] Mark hand-laid decomp tables `#[rustfmt::skip]` so their row layout survives.
- [x] Run `cargo fmt` on the whole workspace (one mechanical commit, no code changes).
- [x] Gate passes: check → clippy -D warnings → fmt --check → test.
