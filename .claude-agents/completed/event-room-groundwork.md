# Interior scene groundwork (generic event room: door, room from the disc, walking, exit) + the kamakura as first user

`app/src/event_room.rs`: a reusable "walk through a door into a small disc room" path, extracted from the pattern
`broker_tent.rs` uses (it is left as it is): `ShopLocation::EventRoom`, `EventRooms` (doors registered per `RoomKind` by
the structure's owner, rooms built lazily from the disc with `IndoorRoom`, current room, visit timer), systems for
entering (near the door point, facing it, walking or E), walking about, and leaving (the entrance, facing out, down or E)
with the return position outside. `app/src/kamakura.rs` is the first user: the snow hut (a box stands for
`kamakura_DL_model`) at unit (4,7) of the shrine acre while KAMAKURA runs (Jan 2 - Feb 23), door check at (x, z+40) within
40 units facing south, exit (x, z+86), the room `rom_kamakura_model(T)` entered at (160, 260), a villager inside (resident 0's
model) with the hello line base (6358 6367 6376 6385 6394 6403 by looks, no gift variants).
Not: the hut's real model and lights, the room's candle rings / mochi, the quest manager's full talk (errands, gifts, hello
variants by Bells / items), the tent migrating onto the generic path.

- [ ] event_room.rs, kamakura.rs, shop.rs match, main
- [ ] gates, Temp run Jan 10, result, comms, commit

## Result
`app/src/event_room.rs` (generic door / room / walk / exit, `ShopLocation::EventRoom`, `RoomKind`, `Door`, `EventRooms`) and
`app/src/kamakura.rs` (first user). Run in game (Temp save, disc, 9 Jan 2027): the hut box stands at the shrine acre's unit
(4,7); walking north into its door point at (x, z+40) entered the real `rom_kamakura_model` room from the disc (rug,
hearth with the pot, candles, lit window), the villager inside answered E with hello 6385 ("Whoa there, phi! That
chowder's mine! ..."), and walking south at the entrance left to (x, z+86) outside.
Unverified: the villager's place in the room (a guess), the door check's yaw band at the original's exact limits, E-press
entering, hello variants (Bells / items), the room's behaviour on save / load while inside (not handled: the player would be
saved in the room).
Not built: the hut's model and window light, the room's candle rings and mochi, the quest manager's full talk, wipes,
`broker_tent.rs` moved onto this path.
