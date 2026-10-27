# Forced greeting (SPEAK demo) for display sales

Philip: "forced greeting" (2026-10-27). Source: `aNSC_message_ctrl_sub` (twins, 85 units) / `aNSC_message_ctrl` (80),
`aNSC_calc_talk_start_tim` (180 frames). After A at a display the keeper requests SPEAK; the player is held;
the keeper keeps following, turns to the player, and the talk starts at < 85 (twins) / 80 units or after 180 frames.

## Result (2026-10-27, claude)
Implemented in `app/src/shop_nook.rs` (`Step::Greeting`, `begin_display_sale`, `hold_player`, `Twins.waiting/expired`) and
`app/src/input.rs` (`update_focus` holds the player while `Twins.waiting`): A at a display no longer opens the talk
at once. The keeper (Nook or the selected twin) keeps following the player (`follow` / `follow_twins` treat the hold as
in-world); the talk starts when he is within 85 units (twins) / 80 (others) and has turned to the player (< 12 deg), or
after `talk_start_tim` = 180 frames, which also makes him stop and turn (`expired`). The twin toggle (`aNMD_selectIdx`)
now happens when the talk starts, as in `aNSC_message_ctrl_sub`.
Verified (Temp, disc, synthetic saves, capture build): upper floor with twins ~260 units away: either twin ran over and the
talk began at ~2.0-2.3 s, twins alternating 1,0,1,0; near twins start within a frame; a slower approach started at ~100
frames (dist < 85). Tier 0: Nook walked from 120 to ~80 units and the talk began at 1.8 s. No WARN/ERROR except the gamepad
mapping line. Gates: fmt, clippy -D warnings, default build; no release build, no tests.
Not verified: the 180-frame expiry itself (every run ended earlier by distance), that movement keys are ignored during the
wait (focus derivation only), cancelling by leaving the shop mid-wait (step reset exists), clothing/carpet route sets.
Harness note: choose "No" (ArrowDown, Space) at the purchase question or the bed is bought and the display becomes sold.
