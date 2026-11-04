# saharah-carpets — Saharah the carpet and wallpaper peddler

Source: `ac_ev_carpetPeddler.c_inc`, kernel `logic/src/event_visitors.rs` (`saharah_*`), schedule row CARPET_PEDDLER.
Talk: 0x48A+0 introduces the offer with two questions (the price is 3000 * 2^used Bells, `used` counted per event and
capped at 4; the new carpet is a random one of the event carpet list `mSP_LIST_EVENT` = 3, shown by name).
First Yes -> second question; No -> end. Second Yes: no carpet in the pockets (ITEM1 category 6) +4, not enough Bells +3,
else +5 and the pockets open: nothing picked +6 (end); a carpet picked -> `used` + 1, the pocket takes the new carpet,
the price is paid, hand-over, +7, she hands the new carpet over; anything else picked -> hand-over, +9, handed back
into the first free pocket. The picked item leaves the pockets either way.
Event area `arabian.used` per special event: `CarpetEvent` (roll date + used) in the save, started empty with a new roll.
Not built: money bags in the pockets count as payment in `mSP_money_check` / `mSP_get_sell_price` (the wallet only here),
the money-bag hand-over visual, the walk-away after the talk.

- [ ] app/src/saharah.rs, save field, main.rs.
- [ ] check / clippy / fmt; Temp run: trade a carpet (price, new carpet, used saved), a second trade (doubled price),
      no carpet, too poor, a non-carpet (refused and returned).

## Result
`app/src/saharah.rs` (`SaharahPlugin`): spawn (the previous day's visitor after midnight), face, talk (`Step`: First,
Second, Picking, Said, Hand), `CarpetEvent` (roll date + `used`; `SaveData.carpet_event`, restored at start-up and on
F9, started at 0 when the shop calendar rolls a new CARPET_PEDDLER event). The first question's Yes follows the
message's own continuation (`next[0]`, here 0x48C) for the second question. Pockets give mode `give_any` (shared).
Temp runs with the disc (synthetic saves: calendar CARPET today, hour offset +5, 7000 Bells; BRP keys, screenshots,
F5 saves):
- Trade-in: "Saharah offers 0x2635 for 3000 Bells (trade-ins 0)", message 0x48A then 0x48C ("You give me any carpet and
  3000 Bells. Can you..." with the price filled in), pockets open, carpet 0x2600 given, hand-over (2, 4, 7), 0x491 (+7),
  she hands over the new carpet (2, 3, 7); F5 save: pocket 0 = 0x2635, Bells 7000 -> 4000, carpet_event used 1.
- A second talk offered 0x2630 for 6000 Bells (the price doubled).
- Refusal: a fish (0x2300) given, hand-over, 0x493 (+9), handed back (second hand-over), pocket 0 = fish again, Bells
  7000, used 0.
Not driven: no carpet in the pockets (+4), too few Bells (+3), No at either question (+1 / end), nothing picked (+6), the
fourth and later trade-ins (cap 4). Not built: bags of money in the pockets as payment, the money-bag hand-over visual.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
