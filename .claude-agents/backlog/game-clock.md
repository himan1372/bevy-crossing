# game-clock — finish the clock on logic::game_time
- [x] Local time source + F6 debug hour + HUD clock (app/src/clock.rs, done with villager-routines).
- [ ] Calendar fields (date, weekday, day-of-year, day count) computed by logic::game_time from the real clock, as the GameCube does (RTC ticks -> calendar).
- [ ] HUD shows weekday + date + time; season available for talk/ecology.
- [ ] Everything that reads the clock (ecology tables, talk, schedules, quest day limits) uses it.
- [ ] Gate passes; live check; Philip playtest (batched at end of M1).
