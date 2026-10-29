# New-game intro demo — slice 1: gate, state, finish_intro

Philip chose the intro demo (2026-10-29): build it fully, then add a way to skip it. Sources: ACGC-PC-Port (== ac-decomp,
https://github.com/ACreTeam/ac-decomp). Retail new game (verified in the source, not the subagent's first guesses):
`SCENE_START_DEMO` is the train interior with Rover (`SP_NPC_GUIDE`, `ac_npc_guide*.c_inc`: asks the time (TIMEIN -> `lbRTC_SetTime`
stores `time_delta`), the player name (LEDIT), gender (Rover overrides the forced male of `mSDI_StartInitNew`), the town name (LEDIT), a phone
call, then the door), leaving at `SCENE_FG` (1979, 200, 760); `ac_intro_demo*.c_inc` (train arrives, player gets off, station master talks,
walks out), the `rcn_guide` (Nook) house tour and choice, payment, then the first job. START_DEMO2 (second player) / START_DEMO3 (Rover visiting),
the title and the p_sel options talk are out of scope.

## Slices (each its own claim, commit and push)
1. New-game gate, intro state, `finish_intro` (this note).
2. The train with Rover: talk spine (NPC0 order index 9), LEDIT names, gender and face, the time menu, phone call, door.
3. Arrival in town: `train1` pulls in, player gets off, station master talks, walk out, camera (pinned Rustimal `train.rs` has the data).
4. Nook's house tour and choice, the payment demand, hand-off into the existing first job.
5. Visual and music polish (train window, intro BGMs).
6. Skip, last: `BEVY_CROSSING_*` env var and an in-intro key, both calling `finish_intro`; confirm the trigger with Philip.

## Slice 1 plan
- `PlayerIdentity.intro_done` (serde default true: existing saves never enter the intro; `Default` is false = a fresh game).
- `app/src/intro.rs`: `Intro` resource (stage), `finish_intro` (the end state: first_job event on, `map_flag` off, the 1000-bell item with the quest
  condition, loan DEBT0, `intro_done`), env-gated (`BEVY_CROSSING_INTRO=skip|play`) until the whole intro exists so a fresh game does not change yet.

## Done (2026-10-29)
- `PlayerIdentity.intro_done` (serde default true; `Default` false), `app/src/intro.rs` (`Intro::Off`, `finish_intro`, `begin_intro` in Startup after `apply_loaded_inventory`), two own tests.
- A fresh game still plays as before: `intro_done` is set silently unless `BEVY_CROSSING_INTRO=skip` (runs `finish_intro`) is given; `=play` will start the intro once slice 2 exists.
- Driven: a fresh Temp dir with `BEVY_CROSSING_INTRO=skip` logged "intro skipped: the part-time job starts". fmt, clippy `-D warnings`, default build clean.
- Not in `finish_intro` yet (slices 3-4 own them): `mHS_set_use` / the house choice, the item-collect bits (cloth, `SUM_CASSE01`, flooring, wallpaper), the first job errand start, `mEv_UnSetFirstIntro`.
