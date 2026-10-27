# Twins' initial actions and Nook's goodbye message

Philip (2026-10-27): "do the twins initial actions and nooks goodbye message". Source: `aNMD_actor_ct`
(start_act_idx, action 61), `aNSC_goodbye_wait`/`aNSC_say_goodbye`/`aNSC_exit_wait` for the non-twin shops.

## Result (2026-10-27, claude)
Source: `aNMD_actor_ct` (`start_act_idx = {0, 1}`: Timmy starts in `aNSC_start_wait`, Tommy in `SAY_HELLO_APPROACH`, both facing
180 degrees; action 61 = WAIT when there is no door actor), the twins' `aNSC_start_wait` (SPEAK request with 0x1722 sale /
0x10E1 raffle / 0x1713 normal, then `SAY_HELLO_APPROACH`: walk until z < 160 or the demo ends), and the common
`aNSC_goodbye_wait` / `say_goodbye` / `exit_wait`.
Implemented (`app/src/shop_nook.rs`, `app/src/shop.rs`):
- Both twins spawn facing 180 degrees. On every arrival upstairs Timmy greets (37 on a sale day, else message 22 = 0x1713,
  new `Step::Welcome`) and steps forward at walk speed until z < 160 while the greeting runs (`Twins.approach`, in
  `follow_twins`); Tommy waits.
- Nook's goodbye: E at the south door no longer leaves at once; the keeper turns and says message 35 (0x109E), then the player
  leaves (`shop_exit` sets `Twins.goodbye` Wanted, `interact` starts it with the existing goodbye mechanism, `Done` lets
  `shop_exit` proceed; skipped on the raffle day and without disc text). `department_stairs` only consumes `Done` upstairs.
Verified (Temp, disc, synthetic tier-3 / tier-0 saves, capture build): upstairs arrival showed Timmy's "Welcome!" (0x1713),
Timmy moved ~0.4 units north to z = 160 while Tommy stayed; tier 0 and the department store's lower floor: E at the door
-> message 0x109E with Nook facing the player -> two Space presses -> out of the shop. An earlier run caught the bug where the
stairs also consumed the lower floor's Done flag (fixed). Gates: fmt, clippy -D warnings, default build; no release, no tests.
Not done: action 61 (no greeting when entering without a door actor, e.g. loading a save inside), the raffle-day twin greeting
0x10E1 (the upper floor is closed then), the original triggers the goodbye by walking into the door (`item_in_front ==
EXIT_DOOR1`) rather than E, Nook's own start actions (non-twin `start_wait` wait types), Timmy's walk was not seen in motion
(z samples only), and the stairs-down goodbye still ends in the stairs rather than a scene change.
