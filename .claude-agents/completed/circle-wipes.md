# Circle wipes

Philip: "do circle wipes and other job items" (2026-10-28). Build the triforce/circle wipe (`fbdemo_triforce`, `ef_wipe1/2/3_modelT`) used as CIRCLE_LEFT/RIGHT, and use it where the source does (the job send-away's arrival).

## Result (2026-10-28, claude)
Source: `m_fbdemo_triforce.c` (`txt` scroll 548 -> 0, step 0.5*(14+12t) per 60 Hz frame, `guPerspective` 60 deg / `guLookAt` z 400, model scale 0.019, `gfx_tex_scroll2(txt, 0, 16, 4)` in segment 9), `m_play.c` (type bits 0x80 = circle left -> `ef_wipe2_modelT`, 0x40 = right -> `ef_wipe3_modelT`), `ac_npc_guide2_move.c_inc` (the job send-away: out = fade black, the next scene's wipe = CIRCLE_LEFT; only the guide NPCs, the mask cat and the player-select NPC use circles).
- `scene_wipe.rs`: `SceneWipe::circle_next()` makes the next arrival a circle: `ef_wipe2_modelT` run through the display-list interpreter, drawn by its own perspective camera (render layer 2, order 20), its texture window scrolled by rewriting the mesh's U every frame (the mesh keeps its main-world copy); black bars beside the 4:3 picture on wider windows until the wipe has nearly opened.
- `guide::send_away` asks for it, so the job's send-away is now: fade to black, out of the shop, the town opens as a circle, then the cheer.
Verified (Temp capture copy, synthetic save with job 6 done, disc): screenshot sequence: shop, black with a small figure beside the tiny opening circle (part of the wipe model), the circle growing over the town, the bars gone and the town clear, then the cheer. An earlier run panicked (a mesh whose render-only data was gone; a query conflict between the cover and bars): both fixed. Gates: fmt, clippy -D warnings, capture build. Real save unchanged.
Not done: circle right, the closing (fade out) direction as a circle, the circle at ordinary doors (they never use one), the exact opening speed on a 30 Hz game frame.
