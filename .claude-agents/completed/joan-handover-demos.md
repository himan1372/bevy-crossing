# Joan's hand-over demos and walk-away

2026-10-26 claude, Philip: "animations". Follow-up to completed/joan-turnip-peddler.md.
Source: `ac_ev_kabuPeddler_move.c_inc` (MONEY_GET_DEMO, KABU_TRANS_DEMO), `ac_event_manager.c`
(`turnipbuyer_behind`, `walk_actor_at_wade`), `ac_handOverItem*`, `mPlib_request_main_give_type1`.

- [x] Audit: what the player and Joan do during the money and turnip demos; her departure.
- [x] Generalise `shop_nook/handover.rs` (kernel `logic/hand_over_item.rs`) to other talkers/items.
- [x] Money bag from the player to Joan, turnip bundle from Joan to the player.
- [x] Joan walks away after the talk.
- [x] Gates, Temp QA, docs/comms, release claim, commit/push.

## Implemented (2026-10-26, claude)
`shop_nook/handover.rs` now takes a `Request{item,giver,taker}` (`Party::{Player,Nook,Npc}`, `Item::{Present,Money,Turnips}` from `present/bag/kabu_DL_*`). NPC takers follow aNPC_act_get (get1 -> GET_PULL with get_pull1 -> PUTAWAY with get_putaway1); player takers play ply get1/get_putaway1; givers play transfer1/trans_wait1 (NPC or `cKF_ba_r_ply_1_*`). The player carries an `NpcClip`, which `animate_characters` plays ahead of locomotion. Joan: sale -> money demo -> turnip demo -> 0x0711 question; after a talk she walks (WALK_SPEED 1.6, NpcGait) to a clear unit with a clear line. Phase log lines `hand-over <item> -> <mode>`.
Verified (Temp, disc, Sun 1 Nov): log shows Money 2,4,7 then Turnips 2,3,7 each sale; Bells/pockets as before; no WARN/ERROR; after "no thanks" she moved ~7.6 units. Visuals only spot-checked (one frame showed the bundle at the player's hand): screenshots are too slow to catch a 1 s demo.
Not done: dialogue window is hidden during the demos (original keeps it locked); walk target is a simple clear-line pick, not `walk_actor_at_wade`'s block search; Nook's present with the new player clips not re-QA'd.
