# gracie-car-wash — Gracie the fashion designer and her car-wash game

Source: `ac_ev_designer_talk.c_inc`, `_schedule.c_inc`, `_move.c_inc`, kernel `logic/src/event_visitors.rs` (`gracie_*`,
tables checked against the C), schedule row DESIGNER.
Normal talk: she judges what the player wears (`wear_flag`: the worn shirt is in the event cloth list, picks the message
table); after the game this visit `21 + RANDOM(3)`; a player who got a gift already `17` (the gift is an event piece) or
`18`; otherwise `8` when someone completed the event, else `RANDOM(4)` (+4 for a girl): the offer, whose Yes continues,
then with a free pocket message 19 and the car-wash game, else the talk ends. Game (`aEDSN_game_end_wait`): press A
(here E) 100 times within 18 s; done with more than 6 s left -> result 0 (a piece of the event cloth list), done in the
last 6 s -> result 1 (the common cloth list), out of time -> result 2 (nothing). Then the forced talk 25, then 12 / 14 /
16; a piece is handed over and put in the first free pocket; the player and piece are recorded. `mEv_SPECL_DESIGNER_COMPLETE`
is switched on when the game is over (any result).
Event area (`mEv_designer_c`): gifted pieces and the complete flag, per special event: `DesignerEvent` in the save.
Not built: the car (her `s_car` structure), the player's wash-car animation, camera and the wash speed; a plain counter
window stands in for the game's view. Money / carpets are not involved.

- [ ] app/src/gracie.rs, save field, main.rs; `goods::select_common` (the common list of a kind).
- [ ] check / clippy / fmt; Temp run: win fast (event piece), win slow (common piece), time out (nothing), a second
      talk (17 / 18), no free pocket.

## Result
`app/src/gracie.rs` (`GraciePlugin`): spawn, face, `wash` (the game: counter text, 100 presses of E in 18 s, result 0 /
1 / 2 by time left), `talk` (`Step`: Offer, Said, Washing, Washed, Hand), `DesignerEvent` (`SaveData.designer_event`:
pieces given + complete flag, per special event, restored at start-up and on F9). `goods::select_common` (the common list
of a kind). Save-system parameter tuple regrouped (Bevy's 16-element limit).
Correction found while driving it: the offer messages (0x721..) carry no question; when they end the original checks the
pockets (`aEDSN_inventory_check_talk_proc`) and shows message 19 ("Stand in front of my car. You'd better work!"),
then the game starts.
Temp runs with the disc (synthetic saves: calendar DESIGNER today, hour offset +5; BRP keys; a script mashing E; F5 saves):
- Fast win: "result 0 (100 presses, 12.2 s left)", forced talk 0x72C (idx 25), prize 0x2423 from the event cloth list,
  0x72D, hand-over completes (2, 3, 7); second talk this visit 0x749 (idx 21); F5 save: pocket 0 = 0x2423, designer event
  gifted [0x2423], complete true.
- Timeout: "result 2 (47 presses)", 0x72C, prize None, 0x731 (idx 16), then 0x74B on the next talk; gifted [], complete true.
- The counter window ("Press E to wash the car! 0/100 17 s") was seen.
Not driven: result 1 (the common list, `select_common`), a restart with the event area saved (17 / 18 for a player with a
gift), no free pocket, the girl's table (idx 4..7), the worn-event-shirt tables (wear_flag 1), message 8.
Not built: the car, the wash animation, camera and wash speed.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
