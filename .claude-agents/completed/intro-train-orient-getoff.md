# New-game intro: the train faces the right way, the player steps off with the real clip

Philip: "train is all f'd fix it make it orientated the right way and the stepping off ani" (2026-10-29).
- The train's models lie along x already (the 90 degree actor angle I added turned them end-on): rotation is identity now; the car's side with the door faces the platform, the locomotive leads.
- Stepping off plays `cKF_ba_r_ply_1_outtrain1`. Its root translation is movement (`cKF_ANIMATION_TRANS_XZ | TRANS_Y`), not pose: `NpcClip::start_moving` keeps the root at its first frame and reports `motion` (model units) and `turn` (the root's x rotation, undrawn like `base_model_rotation.x`); the arrival adds `motion * 0.01 / 40` to the player's position. It lands at about (2180, 820).
