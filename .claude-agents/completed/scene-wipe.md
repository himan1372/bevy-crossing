# Scene wipe

Philip: "hand over and wipe" (2026-10-28). The source's screen wipe on scene changes (`fbdemo_wipe*`, `fb_wipe_mode`) that the app lacks (doors cut instantly). Start with an audit of which transitions use which wipe.

## Result (2026-10-28, claude)
Source: `m_play.c` (`Game_play_fbdemo_wipe_*`, wipe type table), `m_fbdemo_fade.c`, `m_fbdemo_triforce.c`, `m_scene.c` (doors use FADE_TYPE_OUT with `WIPE_TYPE_FADE_BLACK`), `ac_npc_guide2_move.c_inc` (send-away: out = FADE_BLACK, the next scene's wipe = CIRCLE_LEFT).
- New `app/src/scene_wipe.rs`: `SceneWipe` and a full-screen black cover (top UI z). Fade out: alpha 255*frame/30 over 30 frames; arrival: black for 10 frames (`fbdemo_fade_startup`'s timer) then fades in over 30 frames. Every change of `ShopLocation` / `HouseState` (all doors, QuickLoad) starts the arrival fade.
- The fade out waits for nothing at the ordinary doors (they still cut and then fade in); the part-time job's send-away does the whole thing: `guide::send_away` fades out first and only then sends the player out; `job_end` starts its 0.8 s wait when the fade in has finished.
Verified (Temp capture copy, synthetic save with job 6 done, disc): log `scene wipe: out` -> 0.5 s later the exit and `scene wipe: in` -> the cheer 1.5 s after; screenshots: shop, a dimmed town (the fade in), clear, then the green cheer. No panic (an earlier version panicked before the game states existed: now guarded). Gates: fmt, clippy -D warnings, capture build. Real save unchanged.
Not built: the circle wipes (`ef_wipe2/3_modelT`, the scrolled texture, CIRCLE_LEFT is what the send-away's arrival really uses), the fade out at ordinary doors (each door system teleports in the same frame it changes state; delaying them is a separate refactor), the exact retail timings of the 10-frame hold and ramp (read from the PC code: frame rate x game frame; chosen as 30-frame ramps), input lock during fades, title/save-menu wipes.
