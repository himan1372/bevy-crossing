# New-game intro, slice 2c: the rest of Rover's script, the camera, gender and face

Part of the intro demo. Source: `ac_npc_guide_move.c_inc` (`aNGD_*`), `ac_npc_guide_animation.c_inc`, `ac_npc_sleep_obaba*.c*`, `m_player_lib`
face bank, `mPr_GetRandomCloth`. Philip's reference pictures (`plat-test-pics/REAL-GAME-PICS`: the retail train: low camera behind the player,
Rover seated on the green bench, the old woman dozing on the far right bench, windows on the left) were used to check the camera.

## Done (2026-10-29)
- Rover's whole script runs: walk in, 0x2AC6 time check, "mind if I sit" (he sits at (100, 280): sitdown clip, then the loop), 0x2AC8 name window, 0x2ACA, gender answers
  0x2AD6 / 0x2AD7 / 0x2ADF / 0x2AE0, 0x2AD8 town window, 0x2AE4 confirm, 0x2ACB-0x2ACE, then he stands (standup clip), walks the aisle (140, 290) to the door end
  (140, 130), steps onto the deck (to_deck clip), phones Nook (0x2AD0, keitai on / talk / off clips, door-open clip), walks back, the last talk 0x2AD1 (he sits again on the order),
  0x2AD2 / 0x2AD4. The orders are read from `DemoOrders::get` / `clear`, not the raised queue (the first-job guide drains that one: a race that lost orders).
- `logic/src/intro.rs` (own tests): `is_male` (cool/cute x agreed), `face_type` (`aNGD_set_pl_face_type`'s tables, random without the answer bits), `starting_cloth`
  (boy ITM_CLOTH000-007, girl 008-015), `camera_blend` (`cKF_HermitCalc`), `Sway` (the train's tremor on the camera).
- Camera (`train_camera`): eye (100, 80, 400), looks at (90, 80, 280), 40 degree lens, near 60, far 800; once Rover talks it comes round to his head over 40 frames and stays; sway added.
  The player is not drawn on the train. The sleeping old woman (`SP_NPC_SLEEP_OBABA`, unit (4, 4) less (6, 24), `kokkuri_d1`) is in.
- `PlayerLook.female` (the face bank holds the girls' faces after the boys': face + 8) and the saved `player_female`; at the end of the scene the face, shirt and gender are set.
- Driven (Temp capture copy, `BEVY_CROSSING_INTRO=play`): the whole script with names "hi" / "wopo", through to "the player is a boy: face 5, shirt 5" and "the train scene ends".
  Screenshots compared with the retail pictures: the camera, the bench, the old woman, the phone call from the door end.

## Not done
The girl branch live (the cute answer; the table is unit-tested), the time menu ("That's wrong" on the time check, TIMEIN with `lbRTC_SetTime` as a saved offset),
the town / player name windows' keyboard hints text, Rover's expressions (orders index 0), the sit-down message lock, the door model and its opening (`ac_train_door`, `obj_romtrain_door`),
the window scenery (`ac_train_window`), the camera tilt to the door, the intro music, the scene change to the station (slice 3).
