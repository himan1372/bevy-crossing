# game-clock — finish the clock on logic::game_time
- [x] Local time source + F6 debug hour + HUD clock (app/src/clock.rs, done with villager-routines).
- [x] Calendar fields (date, weekday, day-of-year, day count) computed by logic::game_time from the real clock, as the GameCube does (RTC ticks -> calendar).
- [x] HUD shows weekday + date + time; season available for talk/ecology.
- [x] Everything that reads the clock (ecology tables, talk, schedules, quest day limits) uses it.
- [x] Gate passes; live check; Philip playtest (batched at end of M1).
