# Migrate rustimal-crossing modules to bevy-crossing

2026-10-16, muse. Pilot complete: furniture.rs migrated (commit 38c184b1).

## Background

Philip clarified 2026-10-16: bevy-crossing is Animal Crossing in Bevy. The 194
Rust modules in rustimal-crossing were developed with extensive research specifically
to be moved here. The C wiring (Waves A-F) was the interim validation step.

## Scope

Port 194 rustimal-crossing Rust modules into bevy-crossing's `logic/`:

- **113 new modules** — exist only in rustimal-crossing, pure additions
- **81 overlapping modules** — same names but DIVERGED content; need per-module
  merge (rustimal versions are newer: have wiring-wave fixes, refinement tests)
- **10 bevy-only** — infrastructure (gx_backend, interpreter, etc.), do not touch

## Pilot (done)

`furniture.rs`: stripped `#[no_mangle] pub extern "C"` wrappers, exposed native
`pub fn`. Includes the 2026-10-16 `save_angle` bitflag fix. Pattern for the rest.

## Per-module steps

1. Fetch canonical version from rustimal-crossing `rust/src/<mod>.rs`
2. Compare with bevy-crossing `logic/src/<mod>.rs` (if exists)
3. Merge: rustimal wins unless bevy has ECS-specific adaptations
4. Strip C ABI: remove `#[no_mangle]`, change `pub extern "C" fn` → `pub fn`
5. Ensure inner functions are `pub` for cross-module use
6. Run `cargo test` for the module
7. Push, update this file with progress

## Verification

Codex noted 2026-10-16: pilot was inspected but not verified. Each module needs:
- `cargo test` passes
- `cargo clippy` clean
- Spot-check against C decomp behavior (the wiring/ docs in rustimal-crossing
  show exactly when each function is called)

## Progress

- [x] furniture.rs (pilot, 2026-10-16)
- [ ] Remaining 193 modules
