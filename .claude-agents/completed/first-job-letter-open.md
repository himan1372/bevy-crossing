# First job: the letter job (job 4) and the open-quest job (job 11)

Philip: "do the next two" (2026-10-28), read as the next two jobs in the chain after the furniture delivery (`next_job_no`: SendLetter -> Open -> carpet ...).
Source: `aNRG2_stepup_j4` (an unused villager, `mNpc_SetAnimalPersonalID2Memory`, paper, `can_ask_again`), messages 0x0802 (needs FREE3 acre row, FREE5 recipient name),
0x0836 (asked again), 0x0807 (more paper), 0x080B (end), `aQMgr_move_own_errand_letter` (the recipient has a letter from the player: `m_npc.c` marks it when the post office delivers);
`aNRG2_stepup_j11` (`mQst_SetFirstJobOpenQuest`, FJOPENQUEST), 0x0827 (start), 0x0829 (cont), 0x0830 (end), `aQMgr_talk_quest_finish_firstjob_open_quest` (accepting a villager's quest finishes it),
`aQMgr_select_talk` (with the FJOPENQUEST event villagers use the ordinary quest talks, not the job greeting).

## Implemented (2026-10-28, claude)
- `logic/src/first_job.rs`: `check_open_quest(accepted)` and `open_quest_running()` (the open job's `FJOPENQUEST`); the module test covers both. 8 own tests pass.
- `app/src/shop_nook/guide.rs` (rewritten around a `Residents` view of the roster and town): job 4 (`SendLetter`) picks an unused resident, puts them in the address book (`VillagerMemory::meet`, `mNpc_SetAnimalPersonalID2Memory`), gives paper (0x2037), says 0x0802;
  running it asks again (0x0836, `can_ask_again_flag`, session state in `GuideState`); the end talk is 0x080B, then the next job (Open) starts with 0x0827; the open job's continue talk is 0x0829 and its end 0x0830 (then the carpet job is chosen and stops at its start).
  The jobs' message strings are filled from the town plan: FREE3 acre row letter (A-F), FREE4 acre number, FREE5 recipient name, FREE6 post-office acre (`aNRG2_set_str_j3/_j4`), which also fixes job 3's start text.
- `app/src/post.rs`: `Post.sent_to` (saved) records villagers holding a letter from the player (`Post::receive`, the source's `letter_info.exists`); `guide::track` finishes job 4 when the recipient is in it.
- Open job: `track` brings the next quest offer forward (`QuestState::offer_now`) and finishes the job when a quest is accepted; `first_job_talk` stands down while the open job runs, so villagers use the ordinary quest talks (`aQMgr_select_talk` with `FJOPENQUEST`).
Gates: fmt, clippy -D warnings (workspace, all targets), capture build; `cargo test -p rustimal-logic first_job`.
Verified (Temp, disc, synthetic tier-0 save): letter job chosen at progress 1 -> 0x07EF then 0x0802, recipient 29 (Anchovy, not the furniture's 35), paper in slot 1, `villager_memory` ['Anchovy']; adding Anchovy to the saved `post.sent_to` and reloading logged `progress 2 -> 0`.
Open job at progress 2: a quest offer from Freya appeared within seconds, the panel opened on talking to her, Y accepted and the job went `progress 2 -> 0`. No WARN/ERROR.
Not driven: the guide's end talk 0x080B + start 0x0827 in one run, the strings on later pages of 0x0802 (the page screenshot caught the first page while typing), the "more paper" choice path of 0x0807 and the wrong-recipient path (0x0804/0x0808),
the post office delivering a real written letter in this run (the saved `sent_to` was edited), declining the offer.
Not built: SEND_LETTER2, carpet, axe, notice and introductions jobs, the town map hand-over, the end of the whole job (`unset_first_job`).
