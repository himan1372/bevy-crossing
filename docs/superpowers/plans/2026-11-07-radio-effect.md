# Task11: aerobics radio effect

Selected by Philip. Use ac_radio_move.c_inc emission32 every36 60Hz ticks at offset(2,0,-10), angle0x6000,arg0=1; ef_konpu.c lifetime72, native velocity/decay,3 disc note models,5 colour pairs,billboard wobble/growth/fade. Reuse Qrand and N64 display-list/material pipeline. Keep exercise music sync in task16 (radio note variant has no sound).

- [x] Attach owned emitter to the existing radio; implement bounded reusable note render slots with native numeric behavior in aerobics/radio_effect.rs.
- [x] Clear notes/emitter phase on F9, scene/event removal; pause simulation, preserve numeric saves, playable missing-disc fallback.
- [x] Source review, sequential check/clippy/fmt and capture/default/release builds; synthetic Temp runtime captures/cadence/pause/F9/scene/event checks. No tests requested.
- [x] Update roadmaps/handoff/task, complete claim and publish explicit files; Philip visual/audio acceptance remains open.
