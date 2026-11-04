# joan-event-scheduler — Joan follows the original event scheduler

Joan (`app/src/joan.rs`) used a hard-coded "Sunday 06-11" test. K.K. Slider already asks `TodayEvents`.
Make Joan ask `TodayEvents::is_active(ev::KABU_PEDDLER, hour)` too, so both follow `m_event_schedule.c_inc`
and the weekly visitor pick.

- [ ] joan.rs `visit` reads `TodayEvents`; drop the local weekday/hour test.
- [ ] check / clippy / fmt.
- [ ] Temp-copy run: Sunday 07:00 Joan out, Sunday 12:00 gone, Monday none.

## Result
joan.rs `visit` now reads `TodayEvents` (`ev::KABU_PEDDLER`), ordered after `event_today::refresh`. The local
Sunday 06-11 constants are gone; the kernel's own test (`event_schedule`, Joan active 6..=11 on Sunday) covers the
hours. check / clippy -D warnings / fmt pass. Not run in the game (no capture build); same hours as before, so no
visible change expected.
