# First job: the villager side of the delivery (and its message table)

Philip: "do villager side and message table" (2026-10-28). Source: `ac_quest_manager.c` `l_set_errand_data[quest_kind]` (msg_start: DELIVER_FTR complete-init 0x08EF, complete-end 0x08F0; carpet 0x08FB/0x08FC;
axe 0x0907/0x0908; the others 0), `ac_quest_talk_fj_init.c` (`aQMgr_talk_fj_*`: own-quest check, the three-choice greeting at 0x2A6 + looks*2 with choice strings 0x68 / hello-by-time 0x3E-0x40 / hint 0x16A + RANDOM(5),
hand over the quest item, COMPLETE_INIT, reward_to progress = 0, COMPLETE_END, all message numbers + looks * 2).

## Implemented (2026-10-28, claude)
- `app/src/first_job_talk.rs` (new, `FirstJobTalkPlugin`): while the job flag is on, talking to any villager is intercepted (claims the `TalkEvent` through `QuestClaim`, ordered after `interact` and in `QuestInputSet`, before the ordinary `on_talk`).
  The villager says the greeting `0x2A6 + looks*2` with three answers: for the errand's recipient holding the quest item "Delivery!" (select-bank 0x68), the time-of-day hello (0x3E/0x3F/0x40), "Uh, sorry..." (0x16A + RANDOM(5));
  for everyone else the hello, "Any tips?" (0x2F) and "Uh, sorry...". Hello and "Any tips?" hand the talk back to the ordinary dialogue (a `TalkEvent` re-sent once, skipped by the interceptor); the third answer says the hint `0x254A + looks*2`.
  "Delivery!" runs `COMPLETE_INIT` then `COMPLETE_END` (`l_set_errand_data`: furniture 0x08EF/0x08F0, carpet 0x08FB/0x08FC, axes 0x0907/0x0908, plus `looks*2`) and finishes the errand (`progress = 0`); as in `aQMgr_talk_fj_reward_to` the reward is `quest_item`
  = the errand's own item (`QUEST_ITEM_CURRENT_ITEM`), so the furniture stays in the pocket as an ordinary item (the quest condition is cleared).
- `logic/src/first_job.rs`: `FIRST_JOB_GREETING` (message and choice-string numbers); `app/src/messages.rs::GameText::choice` (select bank); `dialogue::disc_message_context` is `pub(crate)`.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build.
Verified (Temp, disc, synthetic tier-0 save, job 3 running with recipient 35 (Ace, `looks` from the disc) and 0x11E8 in slot 1 with the quest condition): the first talk to Ace logged `greeting from Ace #35 (delivery: true)`, then `delivery to Ace`; screenshots showed Ace's own
lines ("Ohh, so that's the story...", "...hook you up with this rad folding chair...", "Hey, no big thing, baby!..."); after the flow the F5 save held the job at progress 0, the item still in slot 1 and no WARN/ERROR. A first run without a recipient showed `delivery: false`
and the ordinary path was handed back. Found by running: (a) a reader and a writer of `TalkEvent` in one system conflict (B0002: use `ResMut<Messages>` + cursor); (b) the interceptor must be ordered after `interact` or the ordinary talk wins.
Not done: choosing the item in the pockets (the source opens the inventory with the item preselected; here the quest item is taken from its slot), the NPC take-out/hand-over visuals and the "hand the item" player animation, the letter job (`SEND_LETTER`
has its own flow), the carpet/axe deliveries are table-driven but their jobs do not start yet, and the hello/tips/hint branches were not driven in this run.
