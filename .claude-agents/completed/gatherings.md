# Cherry blossom viewing, harvest moon, New Year's countdown (`ac_hanami_npc0/1`, `ac_tukimi_npc0/1`, `ac_countdown_npc0/1`)

Scope: standing villagers with their talk lines at the event layouts, in `app/src/gatherings.rs`.
- Cherry viewing (CHERRY_BLOSSOM_FESTIVAL, Apr 5-7): five villagers at the shrine acre units (10,6) (12,8) (6,10) (4,12)
  (4,6) sitting, facing by the table (45 -90 -45 90, the fifth wanders); lines `base + RANDOM(3) + 3 * idx` (bases
  6445 6460 6430 6475 6490 6505; the fifth 6457 6472 6442 6487 6502 6517 + RANDOM(3)); two sakura tables (boxes).
- Harvest moon (HARVEST_MOON_FESTIVAL, 18-20h on the moon day): four villagers in the pond acre, seven layouts, nudged to
  the corners of their unit, facing by `def_angle`; lines 0x1EB0 0x1EBF 0x1EA1 0x1ECE 0x1EDD 0x1EEC + RANDOM(3) + 3 * idx
  (idx 0 none), the others' world names (other towns) are left empty.
- New Year's countdown (NEW_YEARS_EVE_COUNTDOWN, 23:00-00:59): five villagers + two signs at the pond acre (seven
  layouts of eight); the term by time (1 h, 30, 10, 5, 1 min, new year, +10 s), lines `base + RANDOM(3) + 4 * term`
  (new year and after: +17), bases 7528 7549 7507 7570 7591 7612; the host (first) speaks by himself at the 30, 10, 5
  and 1 min marks when the player is in the pond acre (`base1 + (term - 1) * 4`, bases 7531 7552 7510 7573 7594 7615)
  and a salvo of fireworks goes off at midnight. Year and minutes left in the lines' free strings.
Not: animations (sitting, toasts), the sakura tables' and signs' models (boxes), the moon-viewing table, the host's
freeze of the player, the other towns' names, the countdown sounds and the hanabi effect proper.

- [ ] gatherings.rs, fireworks burst shared, main
- [ ] gates, Temp runs, result, comms, commit

## Result
`app/src/gatherings.rs` (the fireworks module's `burst`, `water_acre`, `unit_spot` and `context` made `pub(crate)`).
Run in game (Temp saves, disc): cherry viewing 5 Apr 2027 (five villagers at the shrine units, the fifth wandering, two
box tables; Rolf's line 0x195F "look at the weather"), harvest moon 15 Sep 2027 18-21h (four villagers at the layout's
corners; line 0x1EA8), New Year's countdown at 00:xx on 1 Jan 2027 (five villagers; line 0x1DBA "Enough already! Happy
New Year"). A first version despawned every entity (an unfiltered query): fixed before the commit.
Unverified: the countdown's term lines before midnight (1 h .. 1 min), the host's forced talks at the 30 / 10 / 5 / 1
minute marks, the midnight salvo (shares the fireworks `burst`), the minute / year blanks in the lines, the cherry fifth's
line (6457 + ...), the moon layouts other than the one run, the signs.
Not built: the sitting / toasting animations, models of tables and signs (boxes), the moon-viewing table, the host's
freeze of the player, the visiting towns' names in the moon lines (blank), the sounds.
