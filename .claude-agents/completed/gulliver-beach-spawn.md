# gulliver-beach-spawn — Gulliver washes up on the beach

`ac_event_manager.c` `make_actor_in_seaside_block` / `search_seaside_unit`: block z = the bottom playable row,
block x = 1 + (seed + day + sec) % (xmax - 2) (playable columns), wave tile picked by `mFI_GetWaveUtinBlock`
(random wave-attribute unit, x and z in 1..=14). Active 06:00-22:00 on the weekly date (`TodayEvents`).

- [ ] `app/src/gulliver.rs`: spawn while `ev::DOZAEMON` is active and the player is in town; despawn otherwise or
      once `gulliver_completed`.
- [ ] Wave tile from the disc ground map; he stands on the nearest clear ground inland of it.
- [ ] check / clippy / fmt; Temp run with the disc if available.
Not in this slice: talk, wake-up and the furniture gift, his lying/standing animation.

## Result
`app/src/gulliver.rs` (`GulliverPlugin`): while `ev::DOZAEMON` is active (and not `gulliver_completed`, new
`TodayEvents::gulliver_completed`), in town, he is spawned with the disc's Gulliver model (NPC 238+100, name table
0xD064) at the nearest clear ground to a wave unit of the bottom-row acre. Column and unit come from a hash of the
game day (the original uses RANDOM and the clock's seconds, and saves the place; a day hash keeps him put across a
reload). Wave units use `mCoBG_CheckWaveAttr` (attributes 11, 25, 26, 36): `GroundMap::is_wave`.
Temp run with the disc (synthetic saves, saved weekly state = today): "Gulliver washed up at (25.5, -0.5, 41.5)",
bottom row, column 4. Screenshot after walking the player there: his model stands at the water's edge.
A no-disc run has no ground map, so he does not appear. Without the saved weekly state a normal pick applies.
Not built: dock-block exclusion, talk, wake-up and the furniture gift, lying/standing animations (he stands facing
south). The player can also wade into the same wave tiles; not part of this slice.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
