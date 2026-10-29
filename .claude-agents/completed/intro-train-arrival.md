# New-game intro, slice 3b: the train comes in

Philip: "train" (2026-10-29). Source: `m_train_control.c` (`mTRC_demo_init`, `mTRC_trainControl`), `ac_train0.c` / `ac_train1.c` (+ `_move`, `_draw`), `ac_intro_demo_move.c_inc`
(`aID_first_set_init`, `aID_train_birth_wait_init`), pinned Rustimal `train.rs` for the constants.

## Done
- `logic/src/train_control.rs` (own tests): the controller's nine actions with the USA speeds and rates. The new game's train (`Train::demo()`, x 2037, slow speed) slows and stops within 40 units of the
  waiting train's x (2367); a waiting train pulls out when let go and is gone past x 4400.
- `app/src/intro_arrival.rs`: when the train scene ends the screen fades, the town opens as the left circle wipe (`transition.wipe_type`) with the player standing in the car, `Intro::Arrival`, the music
  `BGM_INTRO_ARRIVE` (id 47 in `scene_music`). The locomotive (`cKF_bs_r_obj_train1_1` + its wheel animation at speed / 40 * 10 frames a tick, at most 0.5), the tender (`obj_train1_2_model`) and the passenger car
  (`cKF_bs_r_obj_train1_3`, `_open` / `_close`: the door opens as the train stops, held open while it waits) follow the kernel on the rails at z = 740 (world: x / 40 - 56, z / 40 - 64, 20 units above the ground),
  the palettes `aSTR_PAL_TRAIN1_A1` / `A2` (57 / 58). The car and tender trail the locomotive by 125 units each. The player stands at the car's offset (60, 20, 20). The camera holds on the platform at
  (2180, 820) (`Camera2_request_main_demo_fromNowPos2`: the town camera's 620 units out at 45 degrees, centred 35 units under the ground there). Input is locked while it runs.
- `prop::Prop::spawn` takes a palette (the structure palette) and `pose` any query filter.
- Driven live (Temp capture copy): the full Rover script, the fade, "arriving at the station", the train slides in along the rails at the top of the picture, the player stands at the car's door, the train stops
  ("the train has stopped"), after three seconds of open door the player is on the platform and the game goes on as a played one (`finish_intro`, so the part-time job starts).

## Not done (the rest of slice 3)
Porter (the station master) and his force talk (0x07DD), stepping off the train (`getoff_train`, the player's `intrain_wait1` / `outtrain1` clips: the player stands in the T pose of the town's default clip),
walking to (2220, 840) and out of the station (z 970), the engineer in the cab (`SP_NPC_ENGINEER`), smoke (`eEC_EFFECT_KISHA_KEMURI`) and steam, the tender's spring, the train's sounds (43, the whistle), the camera
at the train's first frames, the platform's collision attribute at (2180, 820).
