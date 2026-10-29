# New-game intro, slice 3c: Porter, stepping off, the walk one unit

Philip: "yes" (2026-10-29). Source: `ac_npc_station_master*.c_inc` (get-off wait, force talk 0x07DD), `ac_intro_demo_move.c_inc` (`aID_walk_one_unit`, `aID_go_out_of_station`), `m_player_main_demo_getoff_train.c_inc`.

## Done (`app/src/intro_arrival.rs`)
- Porter (`SP_NPC_STATION_MASTER`, model 238+14, voice 2) spawns by the station and turns to the player (0x400 per frame), then speaks 0x07DD as a special-NPC talk.
- After the door has been open a second the player steps down to (2180, 820) over 45 frames; when the talk ends the train may leave (`depart`) and the player walks to (2220, 840) at 2.5 units a frame; the train goes when gone.
- The arrival music fades once the player passes z 970 (`aSTM_talk_wait`).
- Driven live in the Temp capture: stop, greeting, Space through the pages, the train pulling out, the walk, control returned (`finish_intro` still runs there until Nook's welcome is built).

## Not done
The get-off animation (`outtrain1`; the player keeps the default pose), Porter's idle clip, his real home position (guessed 2 cells west of the exit), the engineer, smoke, train sounds, camera mode 9's easing.
