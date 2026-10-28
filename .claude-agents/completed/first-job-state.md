# First job: errand state and the guide's stage logic (slice 1 of stages 1-3)

Philip: "stages 1-3" then "ok lets do that" (2026-10-28), agreed order: (1) this kernel, (2) the guide NPC + stage 1, (3) stage 2, (4) stage 3.
Source: `m_quest.c` mQst_SetFirstJob* / CheckFirstJobFin, `ac_npc_rcn_guide2_talk.c_inc` (job_start_msg_no, refuse table, set_possession, check_job_start, stepup_j1..j11, job_end next_job_no).
- [x] `logic/src/first_job.rs`: the errand state, the 11 job kinds, start/refuse messages, check_job_start, possession lists, next-job table, setup/finish; own tests.
- [x] Saved in the save file (identity), absent = no job.
- [x] Gates, docs/comms, release, commit/push.

## Implemented (2026-10-28, claude)
- `logic/src/first_job.rs` (new): `Job` (the eleven jobs in `mQst_ERRAND_FIRSTJOB_*` order), `FirstJob` (= `mQst_errand_c` for the job: job (None = START), progress, recipient,
  used villagers, item, pocket slot, wrong_cloth), `setup_job` (progress 1), `step_up(Picks)` (the `mQst_SetFirstJob*` setters, progress 2, with `StepUp { can_ask_again, open_quest_event }`),
  `finish` / `finished` (progress 0), `next_job(met_everyone)` (the `next_job_no` table plus the planting -> introductions rule), `Job::start_message / refuse_message / display / can_start`
  (`job_start_msg_no`, `job_start_refuse_msg_no`, `itemNo`, `aNRG2_check_job_start`) and `possession()` (`aNRG2_set_possession`: 7 random flower bags + 3 saplings, quest items, paper, axe).
  Six own tests pass (the progress life cycle, the start checks, the planting hand-out, the delivery bookkeeping, the chain table, the open job).
- `app/src/save.rs`: `PlayerIdentity.first_job` (`Option`, saved as a plain record, absent = no job).
Item numbers used: CLOTH016 0x2410, PAPER55 0x2037, AXE 0x2201, SAPLING 0x2900, WHITE_PANSY_BAG 0x2902, CARPET00 0x2600 (from `m_name_table.h`).
Gates: fmt, clippy -D warnings (workspace, all targets), capture build; `cargo test -p rustimal-logic first_job` (the module's own tests only).
Verified (Temp, disc, synthetic tier-0 save): a `first_job` record loaded by F9 and saved by F5 came back identical; no WARN/ERROR.
Not here: the guide NPC and talk flow (next: guide NPC + stage 1), the job's completion hooks, the villager side of the deliveries, the intro demo. The furniture job's item roll (`mSP_SelectRandomItem_New`
with the ng_list of walk-on furniture) and the "met every villager and Tortimer" check are for the caller.
