# Spooky the ghost (Wisp)

Scope: scheduler ghost day (`init_weekly_event` ghost setup, GHOST row live when the day is in [today-7, today]),
Spooky's night (00:00-04:00): approach states, found bit, normal talk by spirit count, favours at 5 spirits (weeds,
roof colour, item), leaves after. Save: `SaveData.spooky_event`, ghost_day in the weekly save. Slice 2 (separate):
the spirits (hitodama) in the five spirit acres, caught with the net.

- [ ] logic: ghost_day + row (kernel tests)
- [ ] app/src/spooky.rs, save, main
- [ ] gates, Temp run, result, comms, commit

## Result (slice 1)
Kernel: `WeeklyState.ghost_day`, `WeeklyContext.ghost_roll`, GHOST row live when the ghost day is in [today-7, today]
(hours 0-3), two new kernel tests. App: `app/src/spooky.rs`; ghost_day saved in the weekly save, `spooky_event` in the
save. Run in game (Temp save, disc, 01:00 on the ghost day, weeds check bypassed with a temporary env hook, since a
new town has no weeds; hook removed): Spooky hidden at an acre's middle; walking within 5 cells: 0x2ED7; within 2
cells: 0x2EE5 (found, shown); with a five-spirit item in the pockets a talk gives 0x2EF0, the spirits are handed over,
the first favour (weeds) 0x2EF1, weeds cleared, Spooky gone.
Unverified: the roof-colour and item favours (choice 2/3), the 0x2ED8 call-back, 1-4 spirits lines (0x2EEC..), the
04:00 leave, saves of the found bit.
Not built (slice 2): the spirits in the world (hitodama in five acres, caught with the net) -> spirits are not
obtainable yet; translucency and vanish animation.
