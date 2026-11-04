# gulliver-weekly-save — the weekly visitor survives a restart

`TodayEvents.weekly` (`event_schedule::WeeklyState`: dates[WEEKLY], weekly_event.type/flags, dozaemon_completed)
was rebuilt at start-up, so Gulliver's weekday could change after a restart or F9. Save it in the JSON save as
an optional `weekly_event` field (legacy saves: absent = never set, as a new game).

- [ ] `event_today.rs`: `WeeklySave` (serde) <-> `WeeklyState`, `TodayEvents::weekly_save/restore_weekly`.
- [ ] `save.rs`: field `weekly_event`, written by the save system, restored at start-up and by F9.
- [ ] check / clippy / fmt; Temp-copy run: save, restart, same weekly visitor and Gulliver day.

## Result
`event_today::WeeklySave` (serde) is `SaveData::weekly_event`; the save system writes it, `apply_loaded_weekly`
(Startup) and F9 restore it and force today's events to be recomputed. The log now prints the weekly visitor.
Temp run (synthetic saves, no disc): saved {date 11-03, kind DOZAEMON} -> "Weekly visitor DOZAEMON on 11-03",
Gulliver active at 20:00; no weekly field -> a fresh pick (DOZAEMON on 11-06), as before. The picker's day depends
on the launch hour, which is why it has to be saved.
Not driven: the F5 write and F9 reload path (compiles, derives are symmetric); the real save was not touched.
Still open for Gulliver: spawn on the beach, talk and item-giving flow, `gulliver_completed` set by the talk.
