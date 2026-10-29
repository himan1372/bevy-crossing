# New-game intro, slice 2f: finishing the train scene

Philip: "ok finish slice 2" (2026-10-29).

## Done
- The train's music: `Scene::Train` in `logic/src/scene_music.rs` -> `BGM_INTRO_TRAIN` (id 42, the enum two before `BGM_SHOP0` / `BGM_POLICE_BOX` 45); `sound.rs` picks it in `ShopLocation::Train`.
  Seen in the log: "music 42 (sequence 60) on group 3" once the train scene starts.
- The "No" branch of the time check driven live: 0x2AC6 "That's wrong" -> 0x2AE1 -> the clock window -> 0x2AE5 -> "No" -> 0x2AE6 -> the window again -> 0x2AE5 -> "Yes" -> 0x2AC7.
- The cloth pattern of the name and clock windows creeps (`mSM_menu_control_move`'s 0.3535 per tick, `-texture_pos * 2` / `* 4` quarter texels): two screenshots 2.5 s apart show the swirls moved.
- With the earlier parts of slice 2 (a through e) the whole train scene now runs: Rover walks in, time check and clock window, sits, name window, gender answers, town name window, the moving questions, stands,
  walks to the door, phones Nook, comes back, the last talk, gender / face / shirt set, the scene ends (`Step::Finished`).

## Left for later slices (not the train)
- Rover's emotion orders (`{09:..}` NPC0 index 0: 0x15, 0x18, 0x19, 0x1B...): no talk in the port plays them yet (villagers' either), so it is a shared talk feature.
- The clock window's looping hand sound (`sAdo_SysLevStart(0xA)`), the window scenery's tunnel scroll and time-of-day shading, the sit-down message lock, Rover's eyes in his own clips (the same `ClipEyes` now applies to
  them, not looked at), the camera pass over the phone call (Rover is behind the near bench), echo sound while in the tunnel (`sAdo_TrgSeEcho`).
- The scene's end hands over to slice 3: the player gets off the train in town.

## Follow-up (2026-10-29, Philip: "the windows blue to white, it should be blue to yellow" and "rovers mouth doesn't stop moving")
- The sky: the retail sky texture is blue, lilac, yellow, cream. What washed it to white was the room's own translucent window glass (`rom_train_in_modelT`, an aqua sheet): the room now
  draws only the ceiling lamps' glow from that model (`spawn_ceiling_lights`), the scenery's materials have no distance fog (the town's fog would wash the backdrop out), the scenery lists get the
  prim colour / alpha / LOD of `Train_Window_Actor_draw` (ambient + sun colour, `aTW_GetNowAlpha`, `lod_factor`), and the sky's texture coordinates stop at the yellow so the cream and the dark horizon rows
  do not fill the low camera's windows. Seen: deep blue at the top, lilac, yellow, trees (autumn palette) below, as in the retail pictures.
- Mouths: `Face` moves the mouth only while the page is typing out and for 8 ticks after (`aNPC_tex_anm_ctrl` / `kutipaku_timer`); a page waiting for a button or an answer leaves it closed. Villagers' too.
