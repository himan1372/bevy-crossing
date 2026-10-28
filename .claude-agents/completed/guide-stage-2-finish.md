# Stage 2 completion (plant flowers and saplings)

Philip: "when done move on to next task" (2026-10-28). `aQMgr_move_own_errand_seed`: running job 2 finishes when no plant item (`ITEM1_CAT_PLANT` = 0x2900..0x29FF: saplings, flower bags) is left in the pockets.
Wire `FirstJob::check_seed` into `guide::track`.

## Implemented (2026-10-28, claude)
`app/src/shop_nook/guide.rs::track` now also runs `FirstJob::check_seed` for the plant job: progress stays 2 while any plant item (0x2900..0x29FF) is in the pockets and goes to 0 when none is left.
The end talk (`J2_END` 0x07F9) then sets the next job with `next_job(met_everyone)`; `met_everyone` is passed as `false` for now (the app does not yet answer `mNpc_CheckFriendAllAnimal` for the guide),
so the chain always goes to Introductions (job 10), as it does for a player who has not met everyone; that job and the delivery jobs start nothing yet and the guide falls back to the ordinary shop talk.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build.
Verified (Temp, disc, synthetic tier-0 save, job 2 running): with a flower bag and a sapling in the pockets no progress change; with empty pockets `first job progress 2 -> 0`; the arrival said 0x07EF, and after the end talk the record held
job 9 (Introductions) progress 1. No WARN/ERROR. The messages of the end talk themselves were not read back in this run (the log lines before the baseline); the 0x07F4 -> 0x07F6 end talk of job 1 was in `completed/guide-stage-1.md`.
