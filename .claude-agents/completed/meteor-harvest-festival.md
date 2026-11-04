# Meteor shower and harvest festival (`ac_tukimi_npc0/1` with the meteor lines, `ac_harvest_npc0/1`)

Extends `app/src/gatherings.rs`.
- Meteor shower (Aug 12, 18-20h): the harvest moon's villagers (4, `SP_NPC_EV_TUKIMI_0..3`) in the pond acre, own seven layouts
  (`l_ms_pule*`), same corner nudges / angles; lines `base2 + RANDOM(3) + 3 * idx`, base2 = 0x3F46 0x3F55 0x3F37 0x3F64
  0x3F73 0x3F82. Shooting stars (streaks) stand in for the sky effect.
- Harvest festival (4th Thursday of November, 15-19h): five villagers in the shrine acre at units (10,6) (12,8) (5,4) (3,6)
  (4,12), the first four facing 45 270 315 90, lines `base + 3 * idx + RANDOM(3)` with bases 15512 15543 15419 15450
  15481 15574; the fifth (stands, `npc1`) `0x3CA4 0x3CC3 0x3C47 0x3C66 0x3C85 0x3CE2 + RANDOM(3)`; three tables (boxes) at
  (4,5) (11,7) (5,10).
Not: animations, table models, the festival's food, sounds, the visiting towns' names.

- [ ] extend gatherings.rs
- [ ] gates, Temp runs, result, comms, commit

## Result
Both added to `app/src/gatherings.rs` (`Kind::Meteor`, `Kind::Harvest`; the fireworks `Spark` is `pub(crate)`).
Run in game (Temp saves, disc): harvest festival 26 Nov 2026 18h (five villagers at the shrine units, three box tables;
the fifth's line 0x3CA4 "such a mystery"), meteor shower 12 Aug 2027 19h (four villagers on the meteor layout; line
0x3F64 "Checking out meteor showers ..."). The shooting-star streaks are spawned over the pond acre; not seen in the
camera frames I took (they pass high in the sky).
Unverified: the streaks on screen, the harvest festival's first four villagers' lines (15512 ... + 3 * idx), the meteor
layouts other than the one run, the fifth harvest villager's facing (the original's default angle is not read: faces south).
Not built: animations, table models, the festival's food, sounds, the visiting towns' names.
