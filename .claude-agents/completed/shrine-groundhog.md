# Shrine events, slice 2: Groundhog Day

Scope: Feb 2, 07:00-08:59 (GROUNDHOG_DAY): Tortimer (speech actor) waits at the shrine; before 08:00 his talk is
0x3DB6+r5; at 08:00 for a player in the shrine acre the groundhog comes up, speaks by weather (0x3DAF+r3 clear /
0x3DB2+r3 snow), goes down, Tortimer gives the speech 0x3DB5; afterwards his talk is 0x3DBB+r5 (clear) / 0x3DC0+r5 (snow).
Not: audience villagers, camera, music, the later Tortimer (soncho2).

- [ ] app/src/groundhog.rs, main.rs
- [ ] gates, Temp run Feb 2, result, comms, commit

## Result
`app/src/groundhog.rs`. Run in game (Temp save, disc, Feb 2 08:xx): Tortimer (speech model) waits at the shrine unit
(8,9); with the player in the shrine acre at 08:00 the groundhog (the mole model) appears at unit (5,8) and speaks
(0x3DAF, clear), is removed, Tortimer gives the speech 0x3DB5, and a later talk with him gives 0x3DBB (clear weather).
Weather: the port has none, so snow only on the scheduler's snow days, otherwise clear (no rain).
Unverified: the pre-08:00 talk (0x3DB6+), the snow lines, a show with the player arriving later after the speech.
Not built: audience villagers, camera, music, rise/sink animation and hole effect, the later Tortimer (soncho2,
SONCHO_GROUNDHOG_DAY 09:00-16:59 with the commemorative item).
