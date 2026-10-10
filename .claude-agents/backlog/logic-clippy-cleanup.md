# logic/ clippy cleanup (unblock the workspace gate)
- [ ] Run `cargo clippy --all-targets -- -D warnings`; list the errors in `logic/` (13 at last count, e.g. `manual_is_multiple_of` in logic/src/shop.rs).
- [ ] Fix each with the idiomatic change clippy suggests. Do not change `logic/`'s public API or behaviour (it's the decomp-verified game-logic rlib).
- [ ] Use `#[allow]` only where the lint is wrong for ported decomp code, with a one-line reason.
- [ ] Gate passes: check → clippy -D warnings → fmt --check → test.
