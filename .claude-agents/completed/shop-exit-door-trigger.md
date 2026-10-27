# Walk-into-the-door goodbye trigger

Philip: "trigger" (2026-10-27). The original starts the keeper's goodbye when `player->item_in_front == EXIT_DOOR1`
(`aNSC_message_ctrl`), i.e. just by facing the exit door tile; the app needs E at the door (`shop_exit`).

## Result (2026-10-27, claude)
Source: `Player_actor_check_forward` (`m_player_other_func.c_inc`) sets `item_in_front` from the unit 0.00061037 * sins/coss
(about 20 game units, half a cell) ahead of the player; `aNSC_message_ctrl` starts `GOODBYE_WAIT` whenever it is EXIT_DOOR1.
Implemented in `app/src/shop.rs::shop_exit`: when the point half a cell ahead of the player lies in the exit marker's cell
(`room.door_x` / `room.door_z`, half-width 0.5) and a disc keeper can speak (not on the raffle day), the goodbye starts with
no key; E at the door still starts it too. After message 35 the player leaves as before.
Verified (Temp, disc, synthetic tier-0 and tier-3 saves, capture build): entering and standing at the arrival spot started
nothing; walking away north started nothing; walking back south into the door gave message 0x109E and, after 2-3 Space
presses, the town. No WARN/ERROR in the log. Gates: fmt, clippy -D warnings, default build; no release, no tests.
Not done: the upper floor's stairs still use the existing stairs trigger; the no-disc fallback room keeps E only (no keeper text).
