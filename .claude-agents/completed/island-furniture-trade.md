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
