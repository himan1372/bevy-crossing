# First job: the introductions job and Tortimer

Philip: "go next two" (2026-10-28), read as the next two missing pieces: the introductions job (job 10) and Tortimer. After planting, `aNRG2_job_end_talk_proc` sends a player who has not met every resident and Tortimer to
introductions first, so this is the usual path to the furniture job.

## Implemented (2026-10-28, claude)
- `app/src/tortimer.rs` (new): Tortimer (`SP_NPC_SONCHO`, model 238 + 127, sound 2) stands at unit (10, 10) of the shrine block (the wishing well's acre in this town plan) while the job flag is on, as `m_field_make.c`
  places him only for `mEv_CheckFirstJob()`. Talking sets the arbeit trophy at once (`aNS_talk_init`, saved as `PlayerIdentity.arbeit_trophy`) and says 0x3AFC the first time, 0x3AFF + RANDOM(3) after
  (`aNS_set_norm_talk_info`; the lighthouse message 0x3B02 needs the island). Verified: he appeared at (-29.5, 10.5) and `Tortimer message 0x3afc` logged with the trophy set and saved.
- `app/src/shop_nook/guide.rs`: the introductions job (`stepup_j10`) starts with 0x0821 (no items), continues with 0x0822 and ends with 0x0828; `Residents::met_everyone` (every resident in `TownRoster.met` and the trophy) now feeds
  `next_job`, so the post-planting chain only goes to introductions when it should; the per-frame check is `aQMgr_move_own_errand_hello` (all met and the trophy).
- `app/src/first_job_talk.rs`: the job greeting counts as meeting the villager (`TownRoster.met`, `VillagerMemory`), since the interceptor runs before the ordinary talk that used to do it.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build. No kernel change.
Verified (Temp, disc, synthetic tier-0 save, job flag on): introductions chosen at progress 1 -> 0x07EF, 0x0821, record progress 2. Job running with the trophy set: greeting Ace, Anchovy, Bluebear, Freya, Jambette and then Grizzly logged
`first job progress 2 -> 0` after the sixth. No WARN/ERROR.
Not done: Tortimer's own look at the shrine was not inspected (model index assumed), the lighthouse 0x3B02 branch, the town-map hand-over at the end of job 3 (the app has no map flag or HUD map to hand over), the second-letter and second-axe jobs (only the
"recipient moved away" redirects lead to them), and the intro-demo wipe at the very end.
