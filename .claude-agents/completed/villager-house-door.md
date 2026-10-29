# Villager house door driven

Philip: "the villager house door, or a move-out system" (2026-10-28). Part 1: drive a villager's front door with the
door fade (`house::house_enter` villager branch, shares `NextState<HouseState>` with the player's house). Part 2 (move-out) is
scoped after this: see the notes at the end.

## Done (2026-10-28)
- `house.rs` logs each villager's door position (`villager N door at Vec3(..)`).
- Driven (Temp capture copy, hour 14): standing 1.2-2.0 south of villager 0's door, a short W tap and E: `Entered a villager's house` with `scene wipe: out` then `in` 0.5 s later (the same fade as every other door).

## Move-out: what the source says (not built)
- Retail villagers do not leave on their own. `mNpc_ForceRemove` (`m_start_data_init.c`, at start) removes one only when all 15 slots are full and `force_remove_date` is 10+ days old: it picks `mNpc_GetGoodbyAnimalIdx` (fewest memories/letters/towns), destroys the house, sends the goodbye letter (`mNpc_SetGoodbyAnimalMail`).
- `remove_animal_idx` (the villager scheduled to go, set by `mNpc_SetRemoveAnimalNo`) is used by the villagers' "moving" talk quest and the card/e-Reader/island transfers (`mNpc_GetRemoveAnimal`).
- The port's roster is a list in move-in order where resident `i` lives at home spot `i`, so removing one needs a slot that can stay empty (retail's `Animals[15]`), and the roster only grows today (`RESIDENTS_MAX` 15). Not started.
