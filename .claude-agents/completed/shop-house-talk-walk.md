# Keeper steps forward during house talks

Philip: "house talk walk" (2026-10-27). Source: `aNSC_start_wait` actions: renewal-built, statue-built, statue-order messages
(`aNSC_set_talk_info_start_wait` sets `next_action = 1`) and the large house without a basement (`REHOUSE_BASEMENT`, action
`SAY_HELLO_APPROACH`) take the approach walk; the other done-rehouse offers go to `CHECK_*` actions and do not.

## Result (2026-10-27, claude)
`home::nook_msg::walks(message)`: MEDIUM_BUILT, LARGE_BUILT, UPPER_BUILT, STATUE_BUILT, BASEMENT_BUILT, REHOUSE_STATUE and REHOUSE_BASEMENT
(source: `next_action = 1` in `aNSC_set_talk_info_start_wait`, and the large house without a basement choosing `SAY_HELLO_APPROACH`);
REHOUSE_MEDIUM, REHOUSE_2_OFFER, BASEMENT_PAID, REHOUSE_UPPER, REHOUSE_LARGE do not. `Twins.house_walk` is set when the opening house talk
is one of those and cleared when the conversation ends; `hold_player` ORs it into `approach`, which `follow` turns into the forward walk.
Verified (Temp, disc, synthetic tier-0 saves, game clock 15:00): renewal pending (message 0x03, "built"): Nook walked from z -2998.47 to
-2997.5 (the 260-unit limit); small house (message 0x00, the expansion offer): he stayed at -2998.46. No WARN/ERROR. Gates: fmt, clippy
-D warnings, default build; no release, no tests.
Not driven: the statue and basement messages (same flag path).
