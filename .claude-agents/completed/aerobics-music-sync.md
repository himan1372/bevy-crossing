# Task16: native aerobics music synchronization

2026-11-08 codex. Philip explicitly selected16. Claude had completed the clip-only routine without music; this follow-up completes shared-clock integration and replaces clip-completion timing. One reused subagent implemented/reviewed routine and pose seeking; parent owns audio integration and verification.

Source: m_bgm.c849/852 (BGM27 in shrine BLOCK), jaudio_NES/game/radio.c (sequenceDA active BGM handle), internal/track.c1838/2013/2171 and audiostruct.h567 (u16 counter, increment/reset/opcodeC5), ac_taisou_npc0_talk/anime.c_inc. Reuse existing logic::taisou ratio/delay/animation/failure/radio kernels, preserving the unsorted action6 table.

Implementation:
- Engine group counter now advances on actual sequence tempo ticks, resets at sequence start and opcodeC5. Read-only enabled sequence/counter/tempo snapshot drives AerobicsClock; no synthetic wallclock.
- Active shrine-block radio requests BGM27; intro/interior/fanfare precedence and existing ducking remain. Current BGM handle/sequenceDA supplies measure/progress/tempo. F9 restarts only active aerobics music, scene exit clears clock. Missing audio leaves participants neutral.
- NPC poses seek the actual music phase using native boundaries, per-transition delay/morph and variant tables. Normal callers leave new NpcClip.sync_progress unset. Copper leads; at most one villager fails/notices/rejoins, Tortimer's native tired branch returns via wait state. Failure ownership validates Entity and clears on load/removal/unavailable clock. FrameCount gaps resynchronize native states0/2 after pause.
- No save schema changes, dependencies, extracted assets or tests. Routine and clock reflection support bounded runtime inspection.

Verification:
- Sequential workspace check, all-target Clippy -D warnings and fmt pass; capture1m56s, default2m56s, release9m40s. Forced rustc metadata and immediate isolated copy defeat previously observed shared-target stale artifacts; final release contains AerobicsClock.
- Synthetic Temp/bevy-aerobics-20261108, BRP25744, real save never loaded.563 snapshots over160s observed all ten measures,9->0 loop, stable six morning actors, never multiple failures, notice recovery and three Tortimer tired->wait recoveries. Audio engine reports music peaks around.50, no nonfinite samples or limiting; this is not Philip listening acceptance.
- Fresh preview passes exact pause-state freeze/resume synchronization, F9 phase restart to measure0 with failures cleared, shrine-block exit/return and07:00 event/radio/clock removal. Compared possession fields exactly unchanged (tail-report.json).
- Sports fair23Sep09:15: real clock and five exercise actors, poses/resume and numeric retention pass. Full-line screenshot inspected using a paused diagnostic camera adjustment confined to Temp. No-disc clock unavailable, routine neutral and numeric possessions retained; inspected screenshot.
- First long preview lost its primary render window after full-cycle sampling. Its screenshot-only failure is recorded; input/visual checks rerun in fresh preview. Evidence: full-music-loop-report.json, tail-report.json, sports-report.json, no-disc-report.json, PNGs and logs.
- Owned previews closed. Actual save SHA256 unchanged:0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9.

Limits: notice ASE2 sweat and player's own exercise controls remain separate. Full dialogue/cutscene regression and Philip visual/listening acceptance remain open; this does not close M4. No tests requested or run.
