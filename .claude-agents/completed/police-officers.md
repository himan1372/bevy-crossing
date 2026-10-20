# Copper and Booker

Philip 2026-10-20: "start with copper and booker then finish the museum".
Copper (SP_NPC_POLICE) outside the police box: placement, schedule
(exercise, dozing, greetings), talk and menu (event hint, lost-and-found
count, directions/other). Booker's remaining behaviour beyond the desk talk
and lost and found. Other feeds into the lost and found where the port
has the systems.

## Result (claude, 2026-10-20)

Source map via Haiku skim: fd_npc_land.c, m_field_make.c:431-441,
ac_npc_police*.c, ac_npc_police2_move.c_inc, m_police_box.c.
- app/src/copper.rs: placement (station unit + 2 x), CopperUnit blocks the
  player (player.rs), keeps the item on his unit, think (taisou 06-07,
  nemu 02-04), talk by time, exit call-out 0x771 without turning, menu.
- character.rs: NpcClip (named disc clip, loop count / once) and NpcGait.
- police_box.rs: Booker follows by aPOL2 zones and is solid;
  LostAndFound::keep (mPB_keep_item); compaction on leaving;
  grow_lost_and_found (mPB_force_set_keep_item) with SaveData.police_grow_day.
Verified (Temp preview, real save unchanged): Copper's spot, greeting
0x773, menu labels, hint 0x2C37, count 0x782, exit call-out 0x771,
exercise and doze poses, Booker following / stopping at ~50 units /
solid / talk after moving.
Left: event hints once special events exist; foreign-player map
(0x18C6-0x18CA); April Fools override; umbrella pick is a plain range
(mSP_RandomUmbSelect not ported).
