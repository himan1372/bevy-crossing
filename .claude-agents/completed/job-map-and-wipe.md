# The town map and the end-of-job wipe

Philip: "map and wipe" (2026-10-28), the two pieces left after `completed/first-job-full-run.md`.

## Implemented (2026-10-28, claude)
Map (`Common(map_flag)`):
- `PlayerIdentity.map_flag` (saved, default on): the guide turns it off when it starts the job for a new player (the new-game intro's `Common_Set(map_flag, FALSE)`), and hands the map over with the third job's end
  (`aNRG2_3rd_job_end_talk_proc`: set on); `Action::Map` (X) opens `town_map.rs`'s panel only while the flag is on (`mSM_CHECK_OPEN_MAP`; new `InputFocus::Map`; the focus system's two newest modal flags share one tuple parameter
  because it is at Bevy's sixteen-parameter limit).
- The panel is a plain acre grid (rows A-F, columns 1-5, the names the guide's messages use) with the station, shop, post office, police, museum, tailor, well, dock, the player's house and a `*` where the player stands:
  the retail map overlay (`m_map_ovl.c`) is not built.
- Copper (`aPOL_check_player` / `aPOL_check_select2`): without the map his greeting is the one that offers it (0x0772-0x0775 + 0x1554 = 0x18C6-0x18C9); the first answer gives the map (0x18CA, "Press the X Button to view your map"),
  the others are the ordinary answers shifted by one.
The wipe (`aNRG2_all_job_end` / `aNRG2_send_away_init` / `aID_first_job` / `m_player_main_complete_payment.c_inc`):
- After the last job's end talk (0x081A) the guide waits a second (`GuideState.ending`, `guide::send_away`), then the player is sent out of the shop through the goodbye's exit path; in town `job_end.rs` waits 0.8 s,
  plays the player's YATTA1 clip (`cKF_ba_r_ply_1_yatta1`) with the fanfare BGM_INTRO_CHORES_COMPLETE (73, `sound::Fanfare` takes the place of the scene music for 14 s, ending when the message is read), shows 0x17B6
  ("YESSSSS!!! I'm finally done working! I'm free!") after 42 frames with no name plate, and releases the player when it is read.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build. No kernel change.
Verified (Temp, disc, synthetic tier-0 save): job ended at the notice step: 0x081A, one second later `Left the shop`, then `complete payment: the cheer starts` and `message 0x17b6`; the screenshot shows the player outside with the
blue "YESSSSS!!!" window and no name plate; the saved state afterwards had the job dropped, HRAWAIT set and `map_flag` true. The third job's end (0x0801) set `map_flag` from false to true (read from a save).
X with the flag off did nothing and with it on opened the grid (screenshot: "Town map", the buildings, `*` at the player's acre). Copper without the map greeted with 0x18C7, the first answer gave the map (0x18CA) and the panel then opened.
No ERROR/panic in the logs.
Not done: the retail map overlay and the map hand-over animation (the guide gives it with no item model), the fanfare's own length (14 s is a guess), the player's pose was not inspected frame by frame (the clip
name is `yatta1`; the screenshot only shows the message), the green window colour of 0x17B6 (`mDemo_Set_talk_window_color` 185/245/80), the camera moves, the real screen wipe on leaving the shop (the app has no scene wipe),
`mTM_off_renew_time(1)`, the golden-item and house payment variants, and the lighthouse/island pieces.
