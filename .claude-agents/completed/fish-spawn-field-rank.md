# Fish spawn scales with the field rank

Claimed by claude 2026-10-27. Philip chose it after villager-move-in-rank.

Source: `ac_set_ovl_gyoei.c` aSOG_gyoei_get_idx (env_rate_table 0.5/0.75/0.875/1.0.. by rank 0..6),
aSOG_gyoei_get_idx_sub (total over untried entries, each takes weight*env_rate off the running
total, `now < 0` ends with no fish, area mismatch marks the entry tried and rerolls).
Plan: make `logic::ecology::fish_select` faithful (reroll, `now < 0` rule) and use it in
`ecology::place_fish` with `FieldRank` (6 until rated), keeping area-incompatible entries in the table.

## Implemented (2026-10-27, claude)

`logic/src/ecology.rs::fish_select` is now `aSOG_gyoei_get_idx`/`_sub` as in the source: the total
covers untried entries, each takes weight*env_rate off the running total (negative = no fish),
the first with `selected >= running total` is picked, and an entry whose water the acre lacks is
marked tried and the roll repeated (a fresh `fqrand()` each time). Its old unit tests now pass the
roll as a closure (not run). Re-exported with `field_rank_rate` from `logic/src/lib.rs`.
`app/src/ecology.rs::place_fish` keeps area-incompatible entries in the table and picks with
`FieldRank` (6 until rated), replacing `weighted_pick`, so ranks 0/1/2 thin the spawns by
0.5/0.75/0.875 and the weights of fish this acre cannot hold now take part, as in the original.

Verified (Temp, disc, same synthetic town): sweeping every acre twice, a clean town (rank 5)
spawned 40 fish and the same town with 300 trash items (rank 0, env 0.5) spawned 20. No
WARN/ERROR. Not driven: ranks 1 and 2.
