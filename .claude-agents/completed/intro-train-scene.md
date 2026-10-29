# New-game intro, slice 2a: the train room, Rover walks in and talks

Part of the intro demo (see `completed/intro-new-game-gate.md`). Source: `data/scene/start_demo1.c`, `data/field/mvactor/start_demo1.c`,
`actor/npc/ac_npc_guide*.c_inc` (Rover = `SP_NPC_GUIDE`), `rom_train_in_model` / `_modelT` on the disc.

Rover's script (messages from `message_data.bin`, control codes read with `mFont_cont_info_tbl`): 0x2AC6 time check ("is it 3:45 pm on ...?")
-> yes 0x2AC7 ("mind if I sit here?") -> yes 0x2AC8 ("what's your name?" + order NPC0 index 9 value 1 = open the name editor) / no 0x2AC9;
the name answers 0x2ACA (do you like the name? -> 0x2AD6 boy / 0x2AD7 girl / 0x2AE3 again), 0x2AD8 (where are you headed?), town name (LEDIT),
0x2ACB-0x2ADF the moving questions, 0x2AD0 / 0x2ADE the phone call to Nook, 0x2AD1-0x2AD4 the last talk. Orders 0x18-0x1F, 0xFE in `{09:..}` are Rover's
expressions; index 9 is the spine (1 name editor / 2 .. see `aNGD_sex_select_wait2`).

## Done (2026-10-29)
- `ShopLocation::Train`; `app/src/intro_train.rs`: the train room at `TRAIN_ORIGIN` via `IndoorRoom` (floor measured from the model), the player at
  (120, 340), Rover (`NpcModel` 242) walking from unit (3, 3) to z = 290 at 1 unit/frame, turning (0x400 per frame) and talking.
- Rover's messages run through the ordinary talk engine: `talk_asking`, `begin_special("Rover", (ROVER_NPC, 2))`, answers follow `MsgChoice.next`.
  `MsgContext.town_name` + `PUT_STRING_COUNTRY_NAME` (code 47) added for `{2f}`.
- Driven (Temp capture copy, `BEVY_CROSSING_INTRO=play`): walk, 0x2AC6 -> 0x2AC7 -> 0x2AC8, then the log shows "the script asks for the player's name".
  Screenshots seen: train interior, Rover at the player, the talk box.

## Not done (next parts of slice 2)
The "No" branch of the time check (the time menu: TIMEIN, `lbRTC_SetTime` = a saved `time_delta`), the player-name and town-name editors (LEDIT),
gender and face (`aNGD_sex_select_wait2`, `aNGD_set_pl_face_type`), Rover's expressions and the sit / stand / phone / door animations, the sleeping old
woman (`SP_NPC_SLEEP_OBABA`; NPC model index 294 is a villager, the special-NPC table needs checking), the train window scenery (`ac_train_window`),
the door (`ac_train_door`, `obj_romtrain_door`), the camera of the scene, the intro music, the scene change to the station.
