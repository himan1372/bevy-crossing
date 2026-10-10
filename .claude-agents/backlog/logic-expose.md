# logic-expose — open the translated slice modules to the app
- [x] Make `pub` in logic/src/lib.rs: player_move, inventory, item_prefs, talk_request, talk_topics, request_selector, npc_ai, behavior, collision, game_time.
- [x] No behaviour changes; additive API only.
- [x] A small test per module showing the app-facing call works.
- [x] Gate passes (check, clippy -D warnings, fmt, test).
