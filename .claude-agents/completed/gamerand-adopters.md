# Move the local random generators onto the qrand kernel

Philip: "gamerand" (2026-10-28), after `completed/shop-bargain-rng-kernel.md` and `completed/town-gen-qrand.md`. Source: the original draws all of these from the one global
`qrand` state (`fqrand()` / `RANDOM(n)`).
- Draws that were already non-reproducible (a fresh clock seed or per-entity noise) now use the shared `GameRand` resource: grab bag contents
  (`pockets/grab_bag.rs`), the raffle draw and ball launch (`raffle.rs`), face blinks and mouth picks (`face.rs`).
- Draws that were deliberately reproducible from a saved or calendar value keep a per-seed `Qrand::new(...)` stream, as `town_gen` does: villager roster order and move-in pick
  (`roster.rs`), the weekly turnip prices (`shop/turnips.rs`), the monthly raffle lineup (`raffle.rs::lineup`). Using the global stream there would reroll on every reload.
Gates only; no behaviour change intended for the per-seed ones (the stream differs from the old xorshift, so a given seed gives different values).

## Implemented (2026-10-28, claude)
`GameRand` is now registered once in `app/src/main.rs` (it was in `ShopPlugin`).
- On `GameRand`: grab bag contents (`pockets/grab_bag.rs`, `Opening::start` takes the `Qrand`; `pockets_input` gets a `ResMut<GameRand>`), the raffle draw
  (`RANDOM(100)`) and the ball launch (`fqrand`, replacing `rand_unit`) in `raffle.rs`, and the face blink/mouth rolls in `face.rs` (the per-face xorshift field is gone).
- On per-seed `Qrand` streams (reproducible on reload, like `town_gen`): villager roster order and move-in pick (`roster.rs`, `Rng` wraps `Qrand`), the weekly turnip prices
  (`shop/turnips.rs`), the monthly raffle lineup (`raffle.rs::lineup`, seeded by the month).
Values differ from the old xorshift/LCG streams for the same inputs (the generator changed), so a given town seed gives different residents and a different week of turnip prices
than before; nothing is saved from those streams except the results already stored.
Gates: fmt, clippy -D warnings (workspace, all targets); capture build. No tests run.
Verified (Temp, disc, synthetic tier-0 save, game clock 15:00): the game starts, loads the save and enters the shop with no WARN/ERROR/panic; Nook's talk window and face render.
Not driven: opening a grab bag, a raffle draw and ball launch, blink/mouth motion over time, a new town's roster and the turnip week (all compiled and wired only).
Philip's feature set unchanged: no new gameplay. Left on local generators: none from this list.
