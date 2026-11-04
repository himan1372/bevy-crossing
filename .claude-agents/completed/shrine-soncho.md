# Shrine events, slice 3: Tortimer's commemorative speeches (`ac_ev_soncho2`)

Scope: during the SONCHO_* special events (not the sports fairs' field events, the lighthouse vacations or the morning
aerobics) Tortimer stands at the shrine; a player without that event's trophy gets the offer msg(0)/msg(1), then
msg(3) and the commemorative item (`mSC_trophy_item`), or msg(2) with full pockets; with the trophy msg(5) / msg(6+r3).
msg(n) = 0x3280 + 10 * event + n (harvest festival 0x3391 + n). Trophies and the calendar marks are saved per player.

- [ ] app/src/soncho.rs, identity fields, main.rs
- [ ] gates, Temp run on an event day, result, comms, commit

## Result
`app/src/soncho.rs`; `PlayerIdentity.soncho_trophies` / `soncho_calendar` (saved with the identity). Run in game (Temp
save, disc, Jan 1 08:12): Tortimer (event model) at shrine unit (11,6); first talk 0x3280 (msg 0), then 0x3283 (msg 3)
and the New Year's gift (a diary, "red polka pad"); second talk the same day 0x3286 (has the trophy, calendar marked).
Unverified: the full-pockets line (msg 2), the other events' items (table from the decomp's furniture enum, ids
checked only for the diary), the harvest festival's own message base, the Town Day station toy (the port picks from
the train goods list, the exact `mSP_GetRandomStationToyItemNo` list is not confirmed).
Not built: sports fairs' field events, the lighthouse quest, morning aerobics, wandering, the event-name string.
