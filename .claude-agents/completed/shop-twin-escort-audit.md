# Twins' escort / echo choreography audit

Philip chose "escort/twin audit" (2026-10-27). Result of the audit and one fix.

## What "escort" and "twin" are in the source
- Escort = `aNSC_ACTION_WALK_PL_OTHER_ZONE` / `_0x41_`: the shopkeeper follows the player between zones
  (`aNSC_decide_next_move_act` in `ac_npc_shop_common.c`; per-shop `get_zone`, `get_next_zone`, `check_safe_zone`,
  `search_player2` in `ac_npc_{shop,conv,super,depart}_master_move.c_inc`).
- Twins = `ac_npc_mamedanuki.c` (+ `_move.c_inc`), the department store's upper floor, compiled with
  `aNSC_MAMEDANUKI`: two actors (`aNMD_actor_p[2]`) sharing the same shop-common code.

## Fixed (shop masters, tiers 0-3), `app/src/shop_nook.rs::follow`
- Non-twin branch: wait within `aNSC_PLAYER_DISTANCE` = 60, walk inside 90, run beyond (the app used the twins'
  80/110 for every tier).
- Tier 0 safe zone is `aNSM_check_safe_zone`: player z > 280 while the keeper is in zone 3 (the app used the twins'
  `z < 120 && zone 2|3`). Tiers 1-3 tables were already from their own move files.
- Zone function and next-zone table of tier 0 were already equal to `aNSM_get_zone` / `aNSM_get_next_zone`.
Temp runtime check (disc, synthetic save): from four player offsets Nook crossed to the player, ran when far
(distance 251 -> 127 -> 68 in successive seconds), and stopped at exactly 60 units; no WARN/ERROR.

## Not implemented (twins, upper floor) - the real remaining work
Today Timmy stays put (`can_walk` is `!layout.upper`) and Tommy is a static sibling model.
Source behaviour to add:
1. Both actors run the shop-common move with their own zone/next_zone. Route tables depend on `sale_type`:
   normal 68x68, furniture 60x60, clothing 14x14, carpet 6x6 (zone grids `aNMD_get_zone` per tile, targets in
   `aNMD_search_player2`), chosen at ct from `tanuki_shop_status == EVENT` + `mSP_WhatSpecialSale()` via
   `special_sale_type[]`. Thresholds 80 (wait) / 110 (walk vs run); wait when player z < 120 in zones 2/3
   (the `#else` of `aNSC_decide_next_move_act`).
2. Alternation: `aNMD_selectIdx` toggles after each forced talk start; only the selected twin takes the A-press talk
   request (`aNSC_message_ctrl_sub`), the other holds `talk_start_tim = -1` and listens (TALK demo).
3. Goodbye at the exit door: selected twin `GOODBYE_WAIT` (full goodbye), the other `GOODBYE_WAIT2` (only turns).
4. Start actions: `start_act_idx[npc_id != MAMEDANUKI0]`; action 61 when entering without a door actor.
Needs: the four zone/next/position tables generated from the C (about 700 lines), a second `Shopkeeper` for Tommy
with clips, and the selection toggle in `shop_nook.rs`. Larger than a slice; keep as its own task.

## Also unaudited
`talk_start_tim` (180-frame forced greeting at < 85 units) and the collision-pipe WAIT of the follow logic.
