# New-game intro, slice 2d: camera height, window scenery, door, phone, the old woman

Philip's feedback on 2c (2026-10-29): the camera was too high; no landscape outside the windows; the door and the phone were missing; the old woman was
backwards or messed up; Rover's head should be above the speech bubble, the camera level with him when he sits, and it should move smoothly.

## Done
- Camera: it is level with Rover's head (52 game units while he stands, 38 once he sits on the bench), the height and the point it looks at are eased every frame
  (8 % of the gap per 60 Hz tick) so nothing jumps; it still comes round to Rover over 40 frames (`cKF_HermitCalc`) and leans out past the bench while he is at the door end
  (`camera_tilt_*`: eye.x + sin * 20, eye.y - sin * 5). The retail pictures (Philip's REAL-GAME-PICS) were the reference: Rover's face above the box, the bench edge-on.
- Window scenery (`ac_train_window.c`): `rom_train_out_*` models (tunnel wall, sky, clouds, trees, glass shine) at the field origin in 0.05 scale, the trees' palette by the date
  (`aTrainWindow_GetTreePalletIdx`, `logic/src/intro.rs::tree_palette`, segment 10), the trees' tile slides 5 quarter texels per frame (the mesh's UVs are shifted each frame; the mesh keeps
  `MAIN_WORLD` usage), and the tunnel wall goes a second and a half after Rover sits (`sunlight_flag`). Not done: the tunnel's own scroll and the sky / cloud shading by the time of day.
- The train door (`ac_train_door.c`, `cKF_bs_r_obj_romtrain_door` + its animation, at unit (3, 2) from the train layouts 0xC9 / 0xCA): held at frame 1, played at half speed when Rover reaches
  frame 9 of his to-deck clip and frame 22 of the door-open clip, sounds 0x412 / 0x413 at frames 2 and 27. New `app/src/prop.rs`: a skeleton drawn as one entity per drawing joint, posed from an animation.
- Rover's phone (`ac_t_keitai.c`, `cKF_bs_r_tol_keitai_1`, `keitai_on1` / `off1`): on his right hand joint (20), growing out over frames 10-20 and away over 33-41.
- The old woman: the models face +Z at yaw 0 (I had used `looking_to`, which turned her away and made Rover walk in backwards); she now faces the car, dozing.

## Not done / not checked
The door and the phone were not seen close up (Rover stands behind the near bench at the door end and the box covers him: the retail camera leans further than I could verify);
the old woman's eyes show the "+" pattern (the clip's eye texture indices are not applied: `npc_1_*_tex_index`); the tunnel scroll; the cloud animation; the window shading by the time of day.
