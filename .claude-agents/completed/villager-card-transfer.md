# Card / island remove_animal_idx transfers

Philip chose this (2026-10-29), then "kernel plus a debug trigger" (the port has no memory card,
travel or foreign-town path).

Source: `m_npc.c::mNpc_GetRemoveAnimal(transferring, moving_out)` (reached from `m_card.c` when the
player leaves for another town: the villager thinking of moving, `remove_animal_idx`, is packed into
the foreigner file; the other town's `mLd_PlayerManKindCheck` side moves them in). Pinned Rustimal
`rust/src/npc_life.rs` has `may_move_in` / `move_in_slot`; neither is in this repo yet.
`ac_quest_talk_island.c` only has the cancel-remove order (already built); the island has no
transfer code of its own.

Plan:
- `logic/src/villager_transfer.rs`: `may_move_in`, `move_in_slot` (from the pinned module).
- `TownRoster::take_leaver` (moving out: the leaver leaves, memories cleared, pending choice cleared,
  no removal date renewal) and `TownRoster::receive` (moving in: rejected if already here / the last
  villager removed / summer camper; a free slot, else the goodbye villager replaces, else slot 0;
  `appeared` set, removal date renewed).
- Debug: `BEVY_CROSSING_TRANSFER=out|in` at start; `villager-transfer.txt` in the working directory.

Not built (no source trigger here): the return trip (`mNpc_SetReturnAnimal`, `SP_NPC_GO_HONE_NPC`),
`last_removed_animal_id` and the summer camper (passed as `None`), `previous_land_name`.

## Done (2026-10-29)
- `logic/src/villager_transfer.rs` (`may_move_in`, `move_in_slot`, two own tests); `TownRoster::take_leaver` / `receive` / `goodbye_slot` and `Arrival` in `app/src/roster.rs` (two own tests).
- `goodbye_slot` is shared with `force_remove` and now skips the villager thinking of moving, as `mNpc_GetGoodbyAnimalIdx` does (`i != remove_animal_idx`).
- Debug: `BEVY_CROSSING_TRANSFER=out|in` + `villager-transfer.txt` in the working directory. Driven in a Temp copy of the capture exe: `out` packed Grizzly (leaver slot 2) in town A; `in` in town B (other seed)
  moved him in; `in` in A refused him (still lives there: A's save was not rewritten). A replacement goodbye letter goes through `PendingGoodbyes`.
- Audit: the island has no transfer code (`ac_quest_talk_island.c` only has the cancel-remove order, already built); the card path is `m_card.c` -> `mNpc_GetRemoveAnimal`.
- Not built: the return trip (`mNpc_SetReturnAnimal`, the go-home NPC event, `Save.return_animal`), `last_removed_animal_id` and the summer camper (passed as none), `previous_land_name`, any real card or travel trigger.
