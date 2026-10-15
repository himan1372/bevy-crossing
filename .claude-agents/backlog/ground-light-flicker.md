# Ground light flicker

Status: implemented; Philip reports "that looked good" and requests mailbox flicker next.

Original obj_s_myhome1_window_model uses XLU_DECAL2 and G_DECAL_LEQUAL; runtime glow vertices lie on ground y=0. Current generic alpha blend had no decal depth bias, allowing coplanar ground/light depth competition.

- [x] N64MaterialKey specializes only building ground-glow materials marked FLAG_GROUND_DECAL. Positive bias constant 2 / slope 1 matches Bevy 0.19.1 reverse-Z GreaterEqual; depth testing retained, depth writes disabled.
- [x] Shared build_windows marks glow overlays for all buildings and seasonal home variants. Window glass, ordinary geometry, other blended materials and geometry placement untouched.
- [x] Workspace check/clippy all-targets/fmt/capture/default build passed; no automated tests added/run.
- [x] Disposable Temp runtime review at HUD 18:09: player home front/side, villager home styles 1/2/4 and post office. Patches intact across viewpoints; character/tree occlusion preserved. No renderer/shader errors observed. Screenshots glow-*.png stay in Temp/bevy-crossing-house-review. This is sampled visual evidence, not video/frame-by-frame flicker analysis.
- [x] Philip acceptance: "ok that looked good".

Source/API checked locally: original src/data/model/obj_s_myhome1.c and src/static/libforest/emu64/emu64.c::zmode; installed Bevy 0.19.1 pbr_material.rs specialization and render/mesh.rs depth settings. This is a bounded decal accommodation, not full N64/GX depth emulation.

Preview shut down cleanly, no save inputs. Real save SHA256 unchanged:
0CD03EE33FE4FCFAAFF893271D79A0C6C5885C34B5C9A93D1C375350AA4D49A0.
User settings/.gitignore/savegame.json preserved, no disc assets committed.
