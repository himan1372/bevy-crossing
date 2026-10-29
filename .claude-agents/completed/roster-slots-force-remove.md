# Roster slots and `mNpc_ForceRemove`

Philip: "roster" (2026-10-28), after the move-out scoping in `completed/villager-house-door.md`.
Source: `m_npc.c` `mNpc_ForceRemove`, `mNpc_GetGoodbyAnimalIdx`, `mNpc_CheckGoodbyAnimalMemoryNum`,
`mNpc_RenewRemoveHistory`, `mNpc_SetGoodbyAnimalMail` / `mNpc_SetGoodbyMailData` (letter 0x20E + looks*3 + RANDOM(3));
`mSDI_StartInitAfter` (ForceRemove runs before `mNpc_Grow` at start).

Plan: each resident keeps a home slot (`TownRoster.slots`, saved) so removing one leaves its house spot empty and the
others stay put; `logic/src/force_remove.rs` (the goodbye choice, the due check); `remove_day` (saved) renewed on a
move-in and a removal; the goodbye letter goes to the mailbox / post office after the start.

## Done (2026-10-28)
- `logic/src/force_remove.rs`: `due` (full town, removal date 10+ days old), `goodbye_slot` (`mNpc_GetGoodbyAnimalIdx` with `mNpc_CheckGoodbyAnimalMemoryNum`, RANDOM among ties), `goodbye_mail` (0x20E + looks*3 + roll); 3 own tests.
- `roster.rs`: `slots` (a resident's home slot, saved as `villager_slots`), `remove_day` (`force_remove_date`, saved as `last_remove_day`, renewed by a move-in and a removal), `force_remove`, `free_slot` (a newcomer takes the first empty slot); `init_roster` runs ForceRemove before the day's move-in like `mSDI_StartInitAfter`. Memory counts are what the port has: met / letter sent (`Post.sent_to`) / one town.
- Homes: `place_homes_on_reserved` puts each occupied slot on its reserved spot and leaves the rest without a house (`house_acres` u8::MAX); `npc.rs`, `resident_house_looks` and the guide's acre strings look a resident up by slot.
- `post.rs::goodbye_letters`: the goodbye letter (disc mail 0x20E + looks*3 + RANDOM(3), FREE0 player, FREE1 villager, FREE3 town) goes to the mailbox or the post office.
- Driven (Temp capture copy, disc, 15 residents, one unmet, removal date long ago): "Monique moved out", the letter 0x21d in the mailbox, and in the same start Pierce moved in to the empty slot (retail order: ForceRemove then Grow); with a second unmet villager the move-in is blocked, 14 homes and rooms are built, no errors; F5 saved villagers, slots and both dates. Real save hash unchanged.
- Not done: `remove_animal_idx` (the "I'm moving" talk and card/island transfers), the house demolition sign/animation, the leaving villager's other memories (`Post.sent_to` keeps the name), a first-job recipient who left is handled by the redirects. Memory counts beyond met/letter are one-player approximations.
- Pre-existing: `roster::a_new_villager_moves_in_once_a_day` fails on master too (the test expects `GROW_STARTER` for the picked villager but picks `GROW_MOVE_IN`).
