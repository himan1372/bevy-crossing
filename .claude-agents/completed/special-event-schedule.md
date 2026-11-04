# special-event-schedule — Katrina, Saharah, Wendell, Gracie and Crazy Redd on the scheduler

The shop's `shop::bargain::Calendar` already ports `init_special_event` (which of SHOP_SALE, DESIGNER, BROKER_SALE,
ARTIST, CARPET_PEDDLER, GYPSY runs, on which date and hour). `event_schedule` kept those rows off because
`dates[SPECIAL0..3]` and `special_event.type` were not fed.
- [ ] `event_schedule`: `Inputs.special_type`; the six rows are live only when it matches (`update_special_event`).
- [ ] `event_today`: derive `dates[SPECIAL0..3]` and the type from `ShopData.events`; recompute when it changes.
- [ ] check / clippy / fmt; own tests of the kernel (ported-kernel tests are allowed); Temp run: the log lists the
      visitor on the scheduled day.
Handbill rows stay off (leaflets, no actor). The visitors themselves are separate tasks.

## Result
`event_schedule`: `Inputs.special_type`; `special_allows` (the six rows run only as the saved special event);
`row_supported` now excludes only the two handbill rows (and the other old gaps). Own test
`special_visitors_run_only_as_the_saved_special_event` (16 event_schedule tests pass).
`shop::bargain::Calendar::special_event()` gives the event id and `dates[SPECIAL0..3]` (created, start, end = start+1
except for the shop sale, hour); `event_today::refresh` feeds them and recomputes when the calendar changes, not
only on a new day.
Temp run with the disc (synthetic save with an ARTIST calendar for 11-05): 11-03 lists no artist; with
BEVY_CROSSING_DAY_OFFSET=2 the log says "Events 2026-11-05: ARTIST" and "active at 21:00: ARTIST".
The calendar is only rolled with a disc and a player id (`shop::bargain::tick`); no disc: no visitors.
Not built: the visitors themselves; handbill rows; the end-of-event clean-up the original does (`mEv_ClearSpecialEvent`).
check / clippy -D warnings / fmt pass; real save untouched.
