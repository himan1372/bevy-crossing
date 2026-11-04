# wendell-artist — Wendell the hungry artist

Source: `ac_ev_artist_move.c_inc`, kernel `logic/src/event_visitors.rs` (`wendell_*`), schedule row ARTIST
(06:00 on his day to 05:59 the next, via `special-event-schedule`).
Talk: this player already got a wallpaper (`used > 0` with one player) -> 0x2FB, ends. Else 0x2F5 with a question:
Give (0) opens the pockets in the give mode, No (1) -> 0x2F6.
Item picked: it leaves the pocket whatever it is. Nothing -> 0x2F6. Fish (ITEM1 cat 3) -> a wallpaper from the event list
(`mSP_LIST_EVENT` = 3) not in the event's `walls` replaces it in the same slot, recorded; 0x2F9, Wendell hands it over,
0x2FA. Fruit (cat 8) -> 0x2F8, eaten. Tool (cat 2) -> 0x2FC, handed back into the first free pocket. Anything else ->
0x2F7, gone.
Event save (`mEv_artist_c`: used, pids, walls) is per special event instance: the port keeps `ArtistEvent`
(roll date + walls) in the save and starts it empty when the calendar rolls a new event.
Also: the ported kernel's `wendell_msg(class)` indexes the message table by class although the table is indexed by
class + 1 (0 = nothing): fixed with `wendell_msg_for`.
Not built: his tent-less spot is plain free ground; the event-manager walk-away after the talk (`mEv_STATUS_TALK`).

- [ ] logic: `wendell_msg_for`, test. pockets give-mode flag renamed `give_any` (shared with Halloween).
- [ ] app/src/wendell.rs, save field, main.rs.
- [ ] check / clippy / fmt; Temp run: fish (wallpaper), fruit, tool (returned), other, nothing, a second talk (0x2FB).

## Result
- `logic/src/event_visitors.rs`: `wendell_msg_for(class)` (the table is indexed class + 1) and a test; `wendell_msg(class)`
  alone would have given the wrong lines (0x2F6 for a fish).
- `app/src/wendell.rs` (`WendellPlugin`): spawn, face, talk (`Step`: Hungry, Picking, Said, Hand), `ArtistEvent`
  (roll date + walls, saved as `SaveData.artist_event`, restored at start-up and on F9, started empty when the shop
  calendar rolls a new ARTIST event). The pockets give-mode flag is now `give_any` (shared with Halloween).
- Temp runs with the disc (synthetic saves: calendar ARTIST today, hour offset +5, BRP keys, screenshots, F5 saves):
  - Fish 0x2300: class FISH, wallpaper 0x2731 from the event list replaced it in slot 0, 0x2F9, hand-over completes
    (modes 2, 3, 7), 0x2FA "Ahhh, so full! So happy!"; asking again: 0x2FB; F5 save: pockets [0x2731, ...], artist
    walls [0x2731], rolled (2820, 2820).
  - Tool 0x2200: class TOOL, 0x2FC, handed back (2, 3, 7), the tool is in pocket 0 again, walls empty.
Not driven: fruit (0x2F8), another item (0x2F7), nothing / No (0x2F6), a restart with the event area saved.
Not built: the event manager's walk-away after the talk. The model (NPC 238) is Wendell the walrus, standing on
free ground like Katrina.
check / clippy -D warnings / fmt pass; kernel test event_visitors (6) passes; real save untouched.
