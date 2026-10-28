# First job: carpet, axe and notice jobs (and the end of the job)

Philip: "do those 3" (2026-10-28): jobs 5 (deliver carpet), 6 (deliver axe), 7 (post a notice) of `ac_npc_rcn_guide2*.c_inc`, after `completed/first-job-letter-open.md`.

## Implemented (2026-10-28, claude)
- `app/src/shop_nook/guide.rs`: the three jobs start through the same `say_hello` path. Carpet (`aNRG2_stepup_j5`): an unused resident (not the furniture's or the letter's) and a carpet rolled from the shop's lists
  (`goods::delivery_item(Kind::Carpet, ..)`, which replaces `furniture_for_delivery`), start 0x080C, running 0x080F, end 0x0812. Axe (`_j6`): `FirstJob::step_up` gives it to the letter's recipient (`used[1]`), the axe goes in the pockets with the quest
  condition, start 0x0813, running 0x0815, end 0x0818. Notice (`_j7`): start 0x0818, running 0x0819, end 0x081A.
- The end of the whole job: after 0x081A (`aNRG2_all_job_end_talk_proc`) the guide calls `PersonalEvents::unset_first_job` (FIRSTJOB off, HRAWAIT on) and drops the errand, so the next day Nook greets with HRATALK
  (`completed/shop-hratalk.md`) and the shop goes back to ordinary Nook. The part-time restrictions (`completed/part-time-job-core.md`) end with it.
- The villagers' side for carpet and axe is the existing delivery flow (`first_job_talk.rs`, messages 0x08FB/0x08FC and 0x0907/0x0908 + looks*2).
- Notice job completion: STAND-IN. `guide::notice_board` finishes the job when the action button is pressed facing a bulletin board (field items 0x0007/0x000B, the board at town acre (2, 1), cells (7-8, 7)) while the notice job runs.
  The notice overlay and editor (`m_notice_ovl.c`, `mNT_finish_notice_first_job` after the "post it?" check) are not built, so nothing is written or posted.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build. No tests run (no kernel change).
Verified (Temp, disc, synthetic tier-0 save, job flag on, `used` [35, 29]): carpet start 0x07EF -> 0x080C with recipient 25 and a carpet (0x2616) in slot 1; axe start 0x0813 with recipient 29 and the axe (0x2201); notice start 0x0818;
carpet delivery to Ace: `greeting ... (delivery: true)`, `delivery to Ace`, progress 0, the carpet stays; the same for the axe; notice board: the log shows `progress 2 -> 0 (notice posted)` at the board; the end talk 0x081A after progress 0,
after which Nook talked as the ordinary shopkeeper (the loan talk), i.e. the job was over. No WARN/ERROR.
Not done: the axe delivery's reply-letter display (`aQMgr_talk_fj_show_letter`), the carpet/axe "recipient moved away" redirects (0x0810, 0x0816, `CheckRemoveTarget`), the second letter, second axe and introductions jobs and the chain's use of them,
the notice board overlay/editor, the town-map hand-over after job 3, the intro demo wipe at the end (`aNRG2_all_job_end`), the saved `HRAWAIT` -> `HRATALK` day after the end was not driven, and the persisted end state was not read back
(the dialogue kept F5 from saving in my run).
