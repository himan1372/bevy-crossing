# First job: the villager picker, "met everyone" and the furniture roll (start of stage 3)

Philip: "go" (2026-10-28). Source: `mNpc_GetOtherAnimalPersonalID` (an unused resident for the delivery/letter), `mNpc_CheckFriendAllAnimal` (met every resident, used by the job-2 end and `check_hello`),
`mSP_SelectRandomItem_New` for the delivery furniture (ABC list, excluding `ng_list` = the walk-on hole and mat). Then the guide's job 3 start (0x07FB) with a quest-condition item.
(Scope after reading: `mNpc_CheckFriendAllAnimal && mSC_check_ArbeitPlayer` stays false: the mayor trophy 32 is not in the app, so the post-planting chain to Introductions is already what the source gives; no change there.)

## Implemented (2026-10-28, claude)
- `logic/src/first_job.rs`: `pick_other_villager` (`mNpc_GetOtherAnimalPersonalID`: uniform among the residents not used yet), `NO_DELIVERY_FURNITURE` (0x1444 `FTR_TAK_HOLE01`, 0x154C `FTR_KON_MAT`),
  `FAN` 0x1088 (the display item for job 3 instead of a separate enum variant); a ninth test (picker skips used, ends when all used). 8 own tests pass.
- `app/src/goods.rs::furniture_for_delivery`: one furniture piece from the shop's lists (`mSP_SelectRandomItem_New`, furniture, ABC) with the walk-on items excluded.
- `app/src/shop_nook/guide.rs`: job 3 (deliver furniture) starts like the others: picks a resident from `TownRoster` and the furniture, grants it with the quest condition (cond 2), says 0x07FB;
  while running the continue talk is 0x07FE; its end talk is 0x0801 (the source's `aNRG2_3rd_job_end_talk_proc` also hands over the town map and sets the map flag: not done). The mayor-trophy check stays false (no Soncho), so
  the post-planting chain still goes to Introductions, as the source gives for a player without that trophy.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build; `cargo test -p rustimal-logic first_job` (the module's own tests).
Verified (Temp, disc, synthetic tier-0 save, job 3 chosen at progress 1): the guide said 0x07EF then 0x07FB; the record became job 2 progress 2, recipient 55 (one of the roster's residents 186/55/25/35/29/90), item 0x11E8, used [55];
slot 1 holds 0x11E8 with the quest condition (saved pocket conditions 578 = slot 1 cond 2). No WARN/ERROR.
Not done (the rest of stage 3): the villager's side of the delivery (`ac_quest_talk_fj_init.c`: recognising the errand item, taking it, thanking the player, `progress = 0` and the guide's `J3_END` then letter job), the quest message tables (`msg_start`, looks-based), the map hand-over,
and `CheckRemoveTarget` (the recipient moved out) with its 0x07FF redirect.
