# Migrate event_day.rs (Rustimal event-day data) into logic/

2026-11-03, claude. First slice of migrate-rustimal-modules after the furniture pilot. `rust/src/event_day.rs` (pure data: event-ID encoding, the 134-row event schedule, visitor -> NPC kind map,
event-map table, Halloween NPC base, Franklin's present mask, tokyoso states, Jingle wish categories, New Year positions) has no crate dependencies. Plan: copy, drop ABI, register, keep its tests,
check what the app already models for these events (game_time holiday dates, npc_event_mail) before wiring anything. Next slices: game_math(+tables) then calendar.rs; then the event actors
(totakeke, gulliver, fireworks, ...). Source: himan1372/rustimal-crossing working checkout f7a1619a (pinned inventory commit 122867f7).

## Done 2026-11-03 (claude)
`logic/src/event_day.rs` copied from rustimal-crossing `rust/src/event_day.rs` with the `pc_ed_*` C-ABI block removed (nothing here needs a retained ABI) and registered in `logic/src/lib.rs`.
Its 7 own tests pass; `cargo clippy -p rustimal-logic --all-targets -D warnings` and fmt are clean. Not wired into the app: nothing calls it yet (the app's holiday dates live in `game_time`,
`npc_event_mail`, `personal_events`). Nothing was compared against the original C beyond what the source's own header claims (verified against ac-decomp there); no new behaviour in the game.
Next slices: `game_math` + `game_math_tables` (needed by `calendar.rs`), then `calendar.rs`, then the event actors (totakeke, gulliver, fireworks, ...).
