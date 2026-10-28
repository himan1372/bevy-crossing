# town_gen on the shared qrand kernel

Philip: "move town_gen onto GameRand" (2026-10-28). `logic/src/town_gen.rs` kept a private copy of the qrand LCG (`Rng(u32)`). It sits in the logic
crate and is seeded per town from the saved seed, so it must stay deterministic and cannot read the app's global `GameRand`; it takes the shared
`rustimal_logic::qrand::Qrand` kernel instead (same generator `GameRand` wraps). Output must be unchanged.

## Implemented (2026-10-28, claude)
`logic/src/town_gen.rs`: `Rng` now wraps `crate::qrand::Qrand` (`fraction` = `fqrand`); the duplicate LCG constants are gone. The stream is the same
recurrence and mantissa trick as before, so towns generated from a seed do not change. It stays a per-seed stream, not the app's `GameRand` (the logic crate
cannot see it, and a town must be reproducible from its saved seed). The module's own test was adjusted to read the state through `qrand()` and passes
(`cargo test -p rustimal-logic town_gen::tests::random_state`; no other tests run). Gates: fmt, clippy -D warnings. No runtime QA (no behaviour change by construction).
