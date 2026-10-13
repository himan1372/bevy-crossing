# retire-town-gen — the retail layout becomes the town's only source of truth

Leftover from town-from-disc. With the disc loaded, the town comes from the
retail generator (logic::random_field) and the acre templates, but the old
town_gen plan (`TownPlan` cells, `river_tiles`) is still built first and
kept in step (walkability, homes, tree/rock cells, fish water).

- [ ] List what still reads `town.plan` / `river_tiles` when the disc is loaded.
- [ ] Move each to the retail data (`RetailLayout`, `GroundMap`, `FieldItems`).
- [ ] Keep town_gen only for the no-disc placeholder town.
- [ ] Gate passes; live check; Philip playtest.
