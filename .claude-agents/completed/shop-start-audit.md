# Non-twin shops' start behaviour audit

Philip (2026-10-27): "audit non-twin shops start behaviour". Source: `aNSC_start_wait` (non-MAMEDANUKI), its wait types and
`aNSC_set_talk_info_start_wait*` in `ac_npc_shop_common.c`, against the arrival block of `shop_nook.rs::interact`.

## Audit (source: `aNSC_start_wait` and `aNSC_set_talk_info_start_wait*`, non-MAMEDANUKI branch; shop master, convenience, super, depart)
Wait type (`aNSC_start_wait`): resident player: HRATALK event flag > pending sale-day balloon > "done rehouse" > default (REHOUSE);
visitor: balloon > type 3. The message and the next action follow the type.
| Source behaviour | App before | Now |
|---|---|---|
| Balloon (`aNSC_MSG_BALLOON` 218 / `_DEPART` 220, then the present) wins over every other talk when the sale is on, the event's first-talk flag is clear and a pocket is free | house talk first; sale welcome (37/16) then the balloon | balloon message first, replacing house talk and welcome (fixed) |
| Done rehouse (loan 0, no renewal, size < upper, size == next size, no basement order): `start_wait1` message by size/basement | matches (`home::nook_greeting`, earlier tasks) | unchanged |
| Default: renewal built messages, statue built, statue order (`rehouse_msg`, loans), then `aNSC_get_start_call_msg_no`: sale greeting (EVENT 37, depart 16) or START_CALL_NORMAL (22) | sale greeting only on sale days; no ordinary greeting | START_CALL_NORMAL (0x1091) greets on every ordinary arrival (fixed) |
| `SAY_HELLO_APPROACH` after the default talk: the keeper walks +z until `aNSC_POS_Z_MAX` (260 / 260 / 420 / 500) or the talk ends | stood still | walks forward during the ordinary and sale greetings (fixed; not during house talks) |
| HRATALK (`mEv_SAVED_HRATALK_PLRn`, set in `m_event.c` ~320; message -1 = 0x082A, then approach) | none | not implemented (needs the event system that sets the flag) |
| Visitor (foreign) player paths | none | not implemented (no foreign towns) |
| Each arrival re-runs `start_wait` (new actor per scene, so also on coming back down the department stairs) | arrival timer resets on each new keeper | same |
Remaining: the walk is skipped for house talks (renewal/statue messages also take `next_action` 1 in the source); nothing was checked on the
department store's lower floor or tiers 1-2 at runtime beyond the shared code; HRATALK and the visitor paths.

## Result (2026-10-27, claude)
Code: `app/src/shop_nook.rs` (arrival block, `follow` approach walk using `Twins.approach`).
Verified (Temp, disc, synthetic saves, capture build; the game clock at 15:00 because the sale event only exists 12:00-19:59): tier 0, loan 1 (no house
talk): message 0x1091 "Welcome! Do come in! ..." and Nook stepped about one cell forward (z -2998.47 to -2997.5, i.e. 220 to 260 units); tier 0, loan 0
(house talk pending) on a sale day with the balloon unclaimed and empty pockets: the first message was 0x2EC0 (balloon, 218) and no house talk or
welcome came first; no WARN/ERROR. Gates: fmt, clippy -D warnings, default build; no release, no tests.
