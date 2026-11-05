# The lighthouse, its switch and the quest's present (finishes Tortimer's lighthouse quest)

`app/src/toudai.rs`, `app/src/lighthouse.rs` (switch days, `in_check`, `switch_check`, `switch_on`, `event_check/clear`, the present
item: 0x3070 = FTR_TAK_TOUDAI in January, 0x1FA4 = FTR_IKE_JNY_MAKADA01 in February; test added), `app/src/event_room.rs`
(`RoomKind::Lighthouse`, doors that face south or north and may carry the player in at dusk, per-room entry facing).
The lighthouse (TOUDAI) is on the town's southern coast, block (3,6) unit (5,7): world (-3, 39) (found by scanning the field's FG
records: the default island has FLAG, BOAT, COTTAGE_MY and COTTAGE_NPC, no lighthouse).
- Outside: a placeholder tower; its lamp is lit at night (18:00-05:00) when the day's switch was thrown or no quest runs; the door is
  on the north side (walking south into it within 0.9 cells of (x, z - 0.8)); in the quest's week at 18:00-21:59 before the day's
  switch it takes the player in by itself.
- Inside: the real `rom_toudai_model` room (hatch, life buoy, door alcove); the switch at (180, 100) game units: E beside it between
  18:00 and 05:00 in the quest's week throws it (2 s pause, `switch_on`: the day's bit and the player's); the lever glows; leaving
  by walking north at the entry.
- The present demo: after the week (period 2), when the player is in town, Tortimer comes up and says 0x33FD / 0x3414 (all seven days
  switched; 0x33FF / 0x3416 with full pockets: nothing given or cleared), or 0x3400 / 0x3417 (a started quest that fell short);
  the item goes to the pockets and `event_clear` is called.
Run in game (Temp saves, disc): 4 Jan 2027 19:xx with the quest started on the 2nd: walking to the door took me into the lighthouse
room by itself (dusk), E beside the switch threw it ("The lighthouse switch is thrown (day 1)", the lever glowed), walking north
left to the coast; 15 Jan with seven switched days: Tortimer came up, the line "Ho there, phi! ... hot spring resort" and
"Tortimer gives the lighthouse present 0x3070". clippy / fmt / the lighthouse tests clean.
Unverified: Tortimer leaving after the last page (the despawn), the "started only" and February lines, full pockets, the lamp's
glow at night (set, not seen), `in_check` after midnight, walking in and out repeatedly, saving inside the room.
Not built: the tower's disc model (`obj_s_toudai_*` draws black through the building path: it is drawn in the NPC render path in the
original; the model parts and palette 90 are known), the beam and lens, the switch / pole skeletons and animation, the player's switch
animation, the other presents of the present demo (birthday, golden rod / net).
