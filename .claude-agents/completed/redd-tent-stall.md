# redd-tent-stall — Crazy Redd, slice 1: the event, Redd outside and the stall as a menu

Source: `ac_ev_broker_move.c_inc` (outside), `ac_ev_broker2_move.c_inc` (inside the tent), `ac_event_manager.c`
(`init_sp_broker`, `broker_start`), schedule row BROKER_SALE (18:00 on his day until 17:59 the next).
Event area (`mEv_broker_c`): three furniture pieces chosen when the event is rolled (`init_sp_broker`: with the shop's
goods power at 0 the list is COMMON; each piece is from the EVENT list with 50%, else the COMMON list; all different),
the players who bought and the pieces sold (`used`; at three the event is sold out).
Outside talk: 0x788 (not entered the tent yet), 0x789 (has been in), 0x78A (bought something, its name in item string 2).
Inside: hello 0x78B; pointing at a piece: its explanation (0x793 + RANDOM(5) the first time for that piece, 0x7A1 after) with
the name and the price (4 * Nook's price) and a question; Buy -> 0x798 + RANDOM(5) then: too poor 0x7A0, no free pocket 0x79F,
else the piece goes into the first free pocket, the price is paid, 0x79E; No -> 0x79D. Leaving: goodbye 0x792 after a
purchase, else 0x791; a sold-out tent chatters 0x7A2.
Port slice: the tent is not built (no exterior, no interior room): after the outside talk the stall is a menu of the pieces
left (name and price) plus Leave, the rest through the original messages. Redd stays outside (he does not walk in and hide).
The menu's own text ("Which one?") is the port's.
Not built: the tent and its interior with the pieces on display, Redd walking into the tent, his walk around the interior,
money bags counting as payment, the handbills.

- [ ] app/src/redd.rs, save field, main.rs.
- [ ] check / clippy / fmt; Temp run: the three pieces and prices, a purchase (pocket, Bells, 0x79E), too poor, full pockets,
      No, the sold-out state, restart with the event saved.

## Result
`app/src/redd.rs` (`ReddPlugin`): spawn (the previous day's visitor until 18:00), face, `reset_for_new_event` (rolls the
three pieces as `init_sp_broker` does: 50% event list, else the shop's common list, all different), `talk` (`Step`: Said,
Menu, Explain), `BrokerEvent` (`SaveData.broker_event`: roll date, the three slots (0 = sold), what was bought, entered;
restored at start-up and on F9). The save system's inner tuple grew to five event areas.
Temp runs with the disc (synthetic save: calendar BROKER today, clock +15 h to 20:00, 9000 Bells; BRP keys, screenshots,
F5 save):
- Outside: 0x788 ("Don't you dare rush yourself!..." is 0x78B's page), then the stall menu.
- A piece: "That folk guitar? You're...interested?" (0x796), Buy 0x798, "Redd sells 0x10D8 for 6400 Bells" (1600 * 4), 0x79E;
  the second piece 0x793 then Buy 0x79C and, with 2600 Bells, 0x7A0 ("I only deal with paying customers!");
  asking again: 0x7A1 "That sewing machine is only 5400 Bells."; Never mind 0x79D; Leave 0x792 (after a purchase).
- F5 save: pocket 0 = 0x10D8, Bells 2600, broker items [0, 0x1284, 0x324C] (slot 0 sold), sold_items [0x10D8], entered.
- A second outside talk: 0x78A (a returning buyer).
Not driven: the sold-out state (0x7A2), no free pocket (0x79F), 0x789, a restart with the event area saved, the
"Which one?" menu itself was not screenshotted (the first E chose its first entry).
Not built: the tent (exterior and interior), Redd walking in, the pieces on display, money bags as payment, the handbills.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
