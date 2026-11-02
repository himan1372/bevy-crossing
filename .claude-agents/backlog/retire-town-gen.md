# retire-town-gen — the retail layout becomes the town's only source of truth

Leftover from town-from-disc. With the disc loaded, the town comes from the
retail generator (logic::random_field) and the acre templates, but the old
town_gen plan (`TownPlan` cells, `river_tiles`) is still built first and
kept in step (walkability, homes, tree/rock cells, fish water).

- [ ] List what still reads `town.plan` / `river_tiles` when the disc is loaded.
- [ ] Move each to the retail data (`RetailLayout`, `GroundMap`, `FieldItems`).
- [ ] Keep town_gen only for the no-disc placeholder town.
- [ ] Gate passes; live check; Philip playtest.

## Audit 2026-11-02 (claude): what still reads `town.plan` / `river_tiles`

- `town.river_tiles`: `ecology.rs:680` (fish water), `town.rs:126,255,511` (water test, walkability, build).
- `town.plan.acres[..]` (acre feature / cells): `house.rs:222,245`, `npc.rs:211`, `town.rs:120,485`, `town_map.rs:148`.
- `town.plan.house_acres` / `house_units`: `npc.rs:289-291`, `shop_nook/guide.rs:77,86`, `tortimer.rs:51`.
- `town.plan.seed`: `field_items.rs:159`. `plan.villager_count` and the `&mut TownPlan` fix-ups: `disc_town.rs` (974, 1314, 1355, 1403, 1439, 2064).
- Next: replace each group with `RetailLayout` / `GroundMap` / `FieldItems` data, start with the small ones (seed, house_acres, tortimer), then `river_tiles`, then acres.
