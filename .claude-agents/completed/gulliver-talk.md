# gulliver-talk — talking to Gulliver

Source: `ac_ev_dozaemon_move.c_inc`, `ac_ev_dozaemon_schedule.c_inc`.
- Collapsed (WAKEUP clear): normal talk. RANDOM_F < 0.35 -> 0x23F8, he gets up, then a forced talk 0x23FE+RANDOM(6)
  (sets WAKEUP), then `to_ageru`: no free pocket -> 0x2408; else GAVE_FIRST set, 0x2404, then `ageru`: random
  furniture from the Jonason list (mSP_LIST_JONASON = index 11) into the pockets, GIVE set, dozaemon_completed,
  0x2406 (GAVE_FIRST) or 0x2407. Otherwise 0x23F9+RANDOM(5), still collapsed.
- Awake (WAKEUP set): talk -> GIVE clear: `to_ageru2` (0x2409 with no free pocket; else GAVE_FIRST cleared, 0x2405,
  then the same gift, 0x2407); GIVE set: 0x240A+RANDOM(6).
- The three flags are saved with the weekly state (`WeeklySave.dozaemon_flags`) and cleared when a new Gulliver
  day is rolled.
Port simplifications: no lying / getting-up animation, the forced talk follows 0x23F8 directly, the hand-over plays
after the gift message (as K.K. Slider's), free pocket means an empty slot (the condition is not checked).

- [ ] event_today: flags + completed setters, saved.
- [ ] gulliver.rs talk + hand-over; spawn survives `completed` (only a new spawn is blocked).
- [ ] check / clippy / fmt; Temp run with the disc: talk through the wake-up and the gift.

## Result
`app/src/gulliver.rs`: `talk` / `face` / `Step` (Notice, Forced, Offer, Giving, Close, Handing), K.K. Slider's dialogue
and hand-over plumbing. `event_today.rs`: `dozaemon_flags` (saved in `WeeklySave.dozaemon_flags`, cleared when a
new Gulliver day is rolled), `complete_gulliver`. An already-spawned Gulliver stays after the gift; only a new
spawn is blocked (`aEDZ_actor_ct`).
Temp runs with the disc (synthetic saves, saved Gulliver day = today, cap build, BRP keys + screenshots):
- Awake, not given (flags 1): 0x2405 (screenshot, he faces the player), gift 0x301C, 0x2407, the piece crossing in his
  hands (screenshot); F5 saved flags 3, gulliver_completed, pocket 0x301C; a later talk gave 0x240D (0x240A+3).
- Collapsed (flags 0): 0x23FD, 0x23FB (ordinary talks, he stays down), then 0x23F8, forced 0x2402, offer 0x2404, gift
  0x1FC8, 0x2406 (GAVE_FIRST), F5 saved flags 7, completed, pocket 0x1FC8.
check / clippy -D warnings / fmt pass; no tests; the real save untouched.
Not built: lying / getting-up animation and talk camera, the dock exclusion, the normal-condition check on free
pockets, the message item-name slots (all five hold the gift's name; which index each message uses is unchecked).
Observed: the player can walk through him and into the waves (not changed here).
