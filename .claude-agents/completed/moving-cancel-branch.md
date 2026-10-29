# Drive the moving talk's cancel branch, then the roster test

Philip: "branch then roster tests" (2026-10-29). Drive 0x1506 -> 0x150A (peppy, answer 2) live to see `roster::cancel_moving`
fire; then fix `roster::a_new_villager_moves_in_once_a_day` (failing since before the slot work).

## Done (2026-10-29)
- Cancel branch driven live (Temp capture copy, `BEVY_CROSSING_VILLAGER_MOOD=normal`, save with `remove_slot` 1 = Bluebear): the talk reached 0x1506 ("help me or I'll U-turn": answers "Sure!" / "Don't have time!"), the second answer led to 0x150A and the log showed `talked out of moving; now Some(0)` (`roster::cancel_moving`, a new villager picked). Screenshot of the question seen. The saved `remove_slot` after it was not re-read (F5 would not write while the QA state had a dialogue/quest panel up); the field is saved with the rest of the roster.
- Harness trap: a villager's quest offer panel (Freya's "shovel") blocks talks until N declines it.
- Roster test: `a_new_villager_moves_in_once_a_day` asserted the new villager's grow flag was `GROW_STARTER`, but a move-in may be a starter or a `GROW_MOVE_IN` villager (`VillagerTables::can_move_in`); it now asserts that. Whole `bevy-crossing` test run: 76 passed, 21 ignored, 0 failed.
