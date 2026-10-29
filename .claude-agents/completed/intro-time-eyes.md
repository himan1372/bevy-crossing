# New-game intro, slice 2e: the clock window, the old woman's eyes, the void at the side

Philip chose "eyes and the time menu" (2026-10-29), then reported "when the scene starts you can see the void on the left side of the screen".

## Done
- Eyes (`aNPC_Animation_c`: `eye_seq_p` / `eye_seq_type` / `eye_seq_stop_frame`, read from the animation's 0x40-byte header): `ClipEyes` in `character.rs`, `CharacterRig::eye_pattern`,
  `face.rs` lets the clip override the random blinks. The old woman's `kokkuri_d1` holds eye pattern 2 (closed), `kokkuri_d2` has a per-frame table. She now alternates two to four nods
  with one twitch like `aNSO_sleep` / `aNSO_cramp`. Seen: closed eyes in the train.
- The clock window (`m_timeIn_ovl.c`): `logic/src/time_menu.rs` (own tests): the five values and the rules that keep a real date (`mTI_move_Play` up / down, wraps, month length, leap years, year 2001-2099),
  the twelve-hour dial minutes and the hands' sweep (`mTI_set_time`, twelve minutes a tick). `app/src/time_window.rs`: the window from the disc (`tim_win_model`, month and weekday text from
  `tim_win_*_tex_rgb_i4` in segment 8 with the red / blue prim colour, `tim_hari_b1T/b2T/b3T` hands turned about the dial's centre), slides in from the top, "Adjust the clock.",
  keys: Enter / Space / Right = A, Backspace / Left = B, Up / Down change (held keys repeat), F4 = Start. `InputFocus::TimeWindow`.
- `lbRTC_SetTime`: `clock::set_game_time` keeps the difference to the PC's clock (`Save.time_delta` -> saved `clock_delta`, restored at load); `GameClock` already took a delta.
- Rover: "That's wrong!" on 0x2AC6 -> 0x2AE1 (order 9 = 1) -> the window -> the clock is set -> 0x2AE5 ("Now, let me get this straight. You say it's 6:32 PM on ...?") -> yes 0x2AC7 / no 0x2AE6 (window again).
  Driven live: hour +2, minute +7, the hands swept, "clock set to 2026-10-29 18:32", 0x2AE5 shows the new time.
- The void: the camera sits at z = 400, past the car's open end, and a 16:9 window shows beyond the side walls. The scene is drawn in a centred 4:3 viewport (the retail picture's shape); the camera's
  viewport is put back when the scene ends. (The z order bug that hid the month and the hands behind the frame's parts: the frame's 25 parts reach z 202.4.)

## Not done
The clock's looping hand sound (`sAdo_SysLevStart(0xA)`), the window's cloth scroll (`texture_pos`), the clock window's own hints are the port's text, "No" on 0x2AE5 was not driven,
the cheat flag path of `mTI_OVL_OPEN_NORMAL` (not used by the intro).
