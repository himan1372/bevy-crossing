# The job's guide and stage 1 (and the start of stage 2)

Philip: "do those" / "do" / "when done move on to next task" (2026-10-28), item 3. `app/src/shop_nook/guide.rs`: while `identity.events.first_job` Nook is the guide (`ac_npc_rcn_guide2*.c_inc`):
first visit of a session = `aNRG2_think_init_proc` (START: wearing the work uniform already -> job 2 and 0x07EE, 0x0837; else job 1 and 0x07EE; later sessions 0x07EF), then `say_hello_talk_proc`
(job start message after the greeting, items granted, or the refuse message when pockets are full); E on Nook = `aNRG2_norm_talk_request` (job_start / continue / end talks and the next job);
E on goods = 0x081C; changed out of the uniform on a later job = 0x0832; `aQMgr_move_own_errand_cloth` each frame. Jobs 1 and 2 only; the later jobs need villager picks.
Not done: the item hand-over visual and the order-timed emotes, the guide NPC's own model/schedule (he stays the shop's Nook), the forced stair talk 0x0834.

## Implemented (2026-10-28, claude)
- `app/src/shop_nook/guide.rs` (new): `arrival` (first visit of a session, scolding 0x0832), `talk` (E on Nook), `GOODS_MESSAGE` 0x081C, `track` (per-frame `check_cloth`, logs the progress change), `GuideState`.
- `app/src/shop_nook.rs`: `Step::Guide(chain)` (the guide's messages in order); while `identity.events.first_job` the arrival block runs the guide instead of the other start talks, E on a display says 0x081C,
  and E on Nook runs `guide::talk` (falling through to the ordinary shop talk for jobs after the second). The job record is created at START on the first guide visit.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build. No tests run.
Verified (Temp, disc, synthetic tier-0 save, loan 1, empty pockets, job flag on, no job record): arrival says 0x07EE then 0x07F1; the record became job 0 (change clothes) progress 2, item 0x2410, and the uniform is in slot 1;
wearing it (carry onto the player) logged `first job progress 2 -> 0`; talking to Nook said the end talk then 0x07F6, job 1 (plant flowers) progress 2 and the pockets held 7 flower bags (0x2902-0x290A) and 3 saplings (0x2900). No WARN/ERROR.
Not done: the item hand-over visual (the items are given directly), the emotes the order codes ask for (orders are raised, nothing reacts), the refusal path with full pockets and the "already wearing the uniform" start were not driven,
the guide's own model/schedule and the forced stair talk (0x0834), and everything from job 3 (needs villager picks and the quest manager). The plant job's completion check (`check_seed`) is not wired (needs the plant item range).
