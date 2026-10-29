# The "I'm moving" talk (`remove_animal_idx`)

Philip: "im moving talk" (2026-10-28). Source: `mNpc_SetRemoveAnimalNo` (`mNpc_DecideRemoveAnimalNo_Friend`,
`mNpc_DecideRemoveAnimalNo`), `aQMgr_check_client_remove` / `aQMgr_decide_msg_remove` (`l_remove_yes` + RANDOM(3)) in the
game-event branch of the normal talk, `aQMgr_order_cancel_remove` (quest order value 2: the villager stays, another is picked).
Messages 0x11FB-0x11FD (and the other personalities' bases), answers 0x11FE/0x11FF and 0x1200/0x1201.

Plan: `TownRoster.leaver` (a slot, saved) chosen at start; the villager it names says the moving talk in the event branch of
`talk::choose`; the quest order raised by "would you miss me?" yes cancels it and picks another (`DemoOrders` gets a quest queue).

## Done (2026-10-28)
- `logic/src/force_remove.rs::pick_leaver` (`mNpc_SetRemoveAnimalNo` for one player: someone met first, else anyone but the one just talked out of it; nothing for a town of five or fewer), 4 own tests.
- `TownRoster.leaver` (`remove_animal_idx`, saved as `remove_slot`), chosen at start; `is_leaver`, `choose_leaver`, `cancel_leaver`; a leaving villager's slot clears it.
- `talk::choose`: in the game-event branch the leaver says `REMOVE_YES[looks] + RANDOM(3)` (0x11FB-0x11FD for the first personality) instead of the rumour (`aQMgr_decide_msg_game_ev`); the disc messages and their Yes/No answers are shown by the existing talk path. The quest order of "would you miss me?" (0x1200: index 1, value 2) is taken from a new quest queue in `DemoOrders` (`take_quest`, apart from the guide's `take_raised`) and `roster::cancel_moving` picks someone else (`aQMgr_order_cancel_remove`).
- Verified: unit tests (the topic comes only for the leaver, all three messages appear; the order cancels and picks another, an unrelated quest order does nothing); live in a Temp capture copy: a save with `remove_slot` makes that villager log `thinking of moving` on every talk. The moving message itself was not seen: the villagers I could reach (Bluebear, Anchovy, Grizzly) all rolled "happy" for the session, and the happy mood draws from the other topic tree (as in the source), so about 40 talks each never reached the event branch. Not driven: the answers, the order and the new pick in the running game.
- Not done: 0x11FE ("take me with you") and 0x11FD's request to be taken to another village need the island / other-town trips; the `0x11FB` answer's yes has no consequence.
- QA note: a villager inside a house (Freya in the first run) cannot be talked to; teleporting beside a villager with KeyD can walk the player into their own house door, so face them with a short KeyW from the south instead.
