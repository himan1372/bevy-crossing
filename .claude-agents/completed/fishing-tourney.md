# Fishing tourney host (Chip, `ac_ev_angler.c`)

Scope: FISHING_TOURNEY_1/2 (June / November Sundays 06:00-17:59): Chip stands at a river shore; first talk 0x10E8 (entry),
later 0x10EF (Yes/No "caught something?"), after hours 0x10EE. Yes: the pockets open; bass (FISH05-07) is measured (size
by rank) against the record (strictly larger beats: reward cloth/furniture, record saved); other fish / items get their
fish message and a keep-or-give choice, tools come straight back. The record starts as a villager's size by hour
(`aTRC_clip_random_topsize`). Saved as `SaveData.tourney_event`. Not: the five participating villagers, the stands,
the trophy.

- [ ] app/src/tourney.rs, save, main
- [ ] gates, Temp run on a tourney day, result, comms, commit

## Result
`app/src/tourney.rs`; saved as `SaveData.tourney_event`. Run in game (Temp save, disc, Sunday 6 June 08:18, large bass in
the pockets): Chip (bear model) at a river shore (-28.5, 8.5); first talk 0x10E8; second 0x10EF, Yes opens the pockets,
the large bass is taken (hand-over), his measuring line 0x111A and the winner line 0x111C, a new record (19-23 inches
beats the villager's 9), a random cloth (denim shirt 0x24BE) is given and handed over.
Unverified: No (record line), nothing picked, losing a measurement (0x111F.. / msgno+1), small and medium bass, other
fish and items (keep-or-leave choice, labels not checked), tools (0x17E5), the after-18:00 line, the November tourney.
Not built: the five villagers who fish, fishing stands and trophy, the camera.
