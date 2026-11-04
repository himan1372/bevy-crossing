# Lighthouse quest (Tortimer's vacation), the town side

Scope: SONCHO_VACATION_JANUARY / _FEBRUARY rows (Jan 15-24 / Feb 12-21, 10:00-17:59) become live when the lighthouse
data allows (`mSC_LightHouse_Event_Start`); Tortimer (event 101 / 102) stands at the shrine; the first talk is
`MSG_SONCHO_LIGHTHOUSE_1/2` (0x33F4 / 0x340B) and starts the quest (`mSC_LightHouse_Quest_Start`: the date, the
player's started bit; free strings the days a week and a week and a day on); a talk after it (quest running) is the
message + 6 + RANDOM(3). The quest's period (`mSC_LightHouse_get_period`): the start day 0, a week 1, ten more days 2,
then none (data cleared at the next event start). Saved with the identity.
NOT in this slice: the island and the lighthouse switch (the quest cannot be finished without the island, so no
completion, no present, no Kapp'n), the present NPC's ceremony, Tortimer deleting himself when a quest is running.

- [ ] logic: lighthouse_open input and the rows, tests
- [ ] app: lighthouse.rs data + periods, event_today input, soncho.rs events 101/102
- [ ] gates, Temp run Jan 16, result, comms, commit

## Result
Logic: `Inputs.lighthouse_open`, the two vacation rows no longer in the unsupported list (test added; the event_schedule
tests pass, 19). App: `app/src/lighthouse.rs` (the quest's save `LightHouse` kept in the identity, `period`, `event_start`,
`talk_after`, `quest_start`, date helpers; two unit tests pass), `event_today.rs` feeds `lighthouse_open`, `soncho.rs` has
Tortimer for events 101 / 102 (the lighthouse vacations). Run in game (Temp save, disc, 16 Jan 2027 12:xx): Tortimer at
the shrine; the first talk 0x33F4 starts the quest ("The lighthouse quest starts on (2027, 1, 16)"), the explanation
0x33F5 and the questions ("Sure! / No!", "Got it! / I have no idea.") follow through the message's own branches; after a
start a talk is 0x33FA + RANDOM(3) (0x33FC seen).
Unverified: the February vacation, the day strings (free 0 and 1) in the later pages, "I have no idea." repeating, and a
second day (he stays away once the quest runs).
NOT built: the island and the lighthouse switch (so the quest cannot be finished: no completion, no present, no
Kapp'n), the present NPC's ceremony, the effects of the periods on the town (`travel_check`).
