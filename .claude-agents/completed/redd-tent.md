# redd-tent — Redd's tent (exterior, interior room, Redd inside)

Source: `ac_br_shop*.c` (exterior structure `obj_{s,w}_yamishop`, palette `aSTR_PAL_BR_SHOP` = 64, 3 x 3 footprint, door trigger
at (x, z + 60) game units facing north, animated doors: closed frame 1, open frame 17), `data/scene/broker_shop.c` (the
room: `grd_yamishop_model`; the player enters at (100, 200) units, Redd2 stands at unit (4, 4), the three pieces on display at
units (2, 2), (4, 2), (2, 4)), `ac_ev_broker_move.c_inc` (outside Redd stands at tent + (0, 2) cells, talks once, then walks
into the tent and hides until the scene reloads), `ac_ev_broker2_move.c_inc` (inside: hello 0x78B on entering; A facing a
piece explains / sells it; A on Redd chatters (0x7A2 sold out, 0x78C bought, 0x78D + RANDOM(3)); leaving: Redd says goodbye
0x791 / 0x792 first).
Port: `broker_tent.rs` builds the exterior while the event runs (on a clear 3 x 3 lot found next to the day's pick), the room
at (0, -3000), `ShopLocation::BrokerTent`; `redd.rs` loses the stall menu: outside Redd talks once then goes in; inside the
flow is the original's (hello, pieces, chatter, goodbye).
Not built: the tent's ground offsets / collision outside, window glass and light, the shadow, Redd2 following the player,
the sold-out sign on a sold slot, tent music (the town's plays).

- [ ] disc_town: `broker_tent_model`; shop.rs: `ShopLocation::BrokerTent`.
- [ ] broker_tent.rs; redd.rs (outside talk, inside flow).
- [ ] check / clippy / fmt; Temp run: tent exterior seen, walk in, hello, pieces on display, explain / buy, chatter, leave
      with the goodbye, Redd outside again.

## Result
Built and run in game (Temp save, disc): tent exterior (`t1` body + doors posed from the skeleton animation at the open
frame, teal canvas with the Market sign), walk in at the door, interior room, Redd inside with hello 0x78B, the three
pieces on display via `furniture::spawn_furniture`, explain/buy (0x797, 0x79B, 0x79E, money taken), goodbye 0x792, exit,
Redd outside again.
Seen not working / unverified: talking to Redd outside after a visit did not start a talk in one run (a tree stands in
front of him; trees are not lot obstacles); the poor/full/sold-out branches, chatter and the outside intro talk 0x788+
after the first fix were not re-run.
Not built: ground offsets / collision outside, window glass and light, shadow, Redd2 following the player, sold sign,
tent music, money bags, handbills.
