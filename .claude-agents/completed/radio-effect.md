# Radio effect (Next30 task11)

2026-11-07 codex, explicitly selected by Philip. Native references: src/actor/ac_radio_move.c_inc; src/effect/ef_konpu.c; src/data/model/ef_onpu01_00.c. Reused logic/src/radio.rs emission counter/constants/culling and GameRand Qrand. Source review by one reused subagent found no blockers.

Implementation in aerobics/radio_effect.rs, connected through existing aerobics radio spawn/plugin:
- Effect32 at unrotated offset(2,0,-10)/40, angle0x6000,arg0=1. Tick37 first, then36-tick period;72-tick notes.
- Native random3 shapes/5 colours/roll, velocity X25 thenY135, move then sqrt(.8) decay; scale .00156->.0078 over18 ticks, opposing sinusoidal stretch phase2047 per tick, fade60..72.
- Three original onpu display lists/textures at runtime, billboarded and depth-tested. Models01/02 keep their own env50 override; model00 receives native five-colour env. No extracted assets or dependencies.
- Fixed3-slot pool with3 shape variants per slot, immutable mesh/texture cache and reusable per-slot materials. Pause freezes motion; F9 clears notes/counters; event/scene/range removal hides notes. Neighbor-acre culling follows the existing borderless radio kernel.
- Native arg0=1 emits no note sound. Exercise BGM/synchronization/player exercises remain task16. No save-format changes or gameplay transactions.

Verification: workspace check, Clippy all-targets -D warnings and fmt pass. Default build passed2m55s. Shared output was stale after an outside worktree build; initial preview lacked radio code and is not evidence. Forced capture rustc metadata rebuild/copy and bounded synthetic Temp QA pending below. No tests requested/run; actual save hash0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9. Preserve Claude's adjacent radio attach_shadow hunk and other props.

Bounded runtime QA passes in synthetic Temp/bevy-radio-20261107, BRP25743:
- Corrected capture rustc rebuild1m44s copied immediately, confirmed radio-note code in executable. Initial stale binary preview excluded.
- Native morning aerobics31July06:xx. Twelve transform/visibility snapshots over3s show moving/scaling notes and only1-2 visible notes, stable9-entity pool. Inspected two consecutive screenshots: native colored note shapes above radio.
- Pause freezes exact transform/visibility snapshots; resume continues. F9 early snapshot all hidden, later re-emits using identical pool IDs. Beyond-neighbor-acre teleport hides notes; return reuses IDs. F6 to07:00 removes radio and leaves all notes hidden.
- Saved possession fields compared unchanged (exact compared keys in radio-qa-report.json). Missing-disc fallback radio exists, no effect entities allocated, numeric possessions retained; inspected screenshot.
- Own preview closed; real save SHA256 unchanged. No tests requested/run. Native timing and all five palette values are source-reviewed; bounded screenshots do not constitute exhaustive frame-perfect or Philip visual acceptance. Exercise music/routine remains task16.

Release passes8m37s (forced rustc metadata to prevent stale shared artifacts); verified radio-note code in final release executable. Explicit task-file publication follows. Philip visual acceptance and M4 remain open.
