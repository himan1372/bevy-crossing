# The island resident's furniture trade and her room

`app/src/island_trade.rs`; touches `dialogue.rs`, `house.rs`, `interior.rs` (`spawn_villager_shell`), `npc.rs` (`IslandNpc` carries the
islander's ordinal), `roster.rs`, `save.rs` (`PlayerIdentity.island_room`), `kappn.rs` (a new visit re-arms the first-talk check).
- Her room is the islander's own layout from the NPC room bank (`l_island_npc_best_fg_id`): her best furniture and reserved markers
  (`RSV_ISLAND_FTR0..15`) in 16 slots. She starts with an empty room; a best piece appears once the player gives its twin (`have`
  bit), a marker shows the normal piece given for its size (two 1x1, one 1x2, one 2x2).
- Talk (`aQMgr_decide_ftr_msg_no`, single-player subset): the first talk of a visit always checks, later ones check 85 % of the time
  (the rest is the mail roll, which falls back to plain chat): 0 a best piece is carried (`0x35E0`, Yes `0x35E9`: she takes it),
  1 a normal piece (`0x35E1`, Yes `0x35EF`: she takes it and pays 100-3000 Bells), 5 what she dreams of (`0x35E5`), 7 a piece the
  player gave (`0x35E7`), 8 a shirt (`0x35E8`, Yes `0x35EB`). The base per personality is `0x35E0 0x35FB 0x382F 0x3844 0x3616 0x381A`.
  The messages' own demo orders (`SET_DEMO_ORDER` quest kind: decide-trade 18-21, trade 13 / 22 / 23 / 24) are read from the
  follow-up pages and carried out (`aQMgr_order_trade`).
- Run in game (Temp saves, disc): carrying a best piece (0x10EF) the first talk was `0x35E0`, Yes -> "This Mama bear is beautiful, but
  your soul is even more beautiful" ("The island resident takes 0x10ef"), her room then shows the piece; carrying a normal piece
  (0x1008, a blue wardrobe) `0x35E1`, Yes -> takes it and pays 140 Bells, the room shows it at a 1x1 marker, then the "dream" message.
  Her room in the game after the trade: the piece against the north wall on the toy carpet.
Unverified: checks 7 and 8 and the shirt trade in game, 1x2 and 2x2 pieces, saving / loading the room's state (it rides in
`PlayerIdentity`), her wardrobe's own turn (markers' direction), conditions of pocket items (presents are not excluded).
Not built: the checks that need another player's memory (2, 3, 4, 6: the present piece), her wearing the shirt, the hand-over
animations and the bag of Bells (the items change hands at once), her mail and player-action talks.

## Verification in game (2026-11-06, claude; no code change)
Temp save, disc, debug build with `remote,capture`, islander ordinal 10 (Annalise). Her layout (`fgnpcdata` record `0x02C4`) has slots
0 `0x1584`, 1 marker 2x2, 2 marker 1x1 (second), 3 `0x1154`, 4 `0x1EAD`, 5 `0x1EB1`, 6 `0x157C`, 7 marker 1x2, 8 `0x158C`, 9 `0x1133`,
10 marker 1x1 (first), 11 `0x1134`.
- Check 0/best piece: `0x1154` (retro stereo) taken, 140 Bells paid by the message's own order, `have` bit 3 set, her room shows it.
- Check 1/normal pieces: a 1x1 (`0x1008`, 2940 Bells) went to the second 1x1 marker (`trade[1]`, slot 2, reading order); a 1x2 bed
  (`0x1358`, 640 Bells) to the 1x2 marker (`trade[2]` = `0x135B`, turned by the marker's direction). `have` = 140 (bits 2, 3, 7).
- Check 8/shirt (`0x2410`): taken (no room entry, as intended), pockets empty afterwards.
- Check 5 (dream, "black knight") and check 7 (recall) were rolled and returned a message; check 7's text was not inspected.
- Save / load: F5 writes `island_room` (`have`, `trade`); after a restart her room shows the stereo, wardrobe and bed (5 cells);
  the bed lies against the east wall with the marker's turn.
Still unverified: a 2x2 piece, check 7's wording, presents, the first 1x1 marker (`trade[0]`).
Found, not fixed: the shared quest system offers the island resident a villager errand ("cabin wall at the shop", "Annalise has a
quest") that blocks talks until answered; the original's errands are for town villagers. QA traps: the `first_job` flag of a copied
save greets before any trade check (clear `identity.events` / `first_job`); a villager standing at her door turns E into a door entry.

Fixed 2026-11-06 (`island-no-villager-quests`): `quest.rs::quest_cooldown_tick` now leaves the island resident out of both the giver
pick and the list of names a quest can mention. Check/clippy/fmt pass; live run offered a quest from a town villager (Eloise), the
negative case is by construction (one random pick per quest, not sampled to exhaustion).
