# Twins: finish the remaining behaviours

Philip: "finish twin" (2026-10-27), after `completed/shop-twins-upper-floor.md`. Remaining list there: goodbye2,
forced greeting (talk_start_tim), listening twin waits, player collision, start actions, special-sale route QA,
Tommy visibility/nameplate check.

## Result (2026-10-27, claude)
Done: the exit goodbye at the stairs (`aNSC_ACTION_GOODBYE_WAIT`/`_WAIT2`): `department_stairs` holds the player when
leaving the upper floor (`Twins.goodbye` Wanted), the selected twin says message 35 ("Thanks much, and please come
again"), the other twin only turns (both twins face the player while a conversation runs), then the floor changes
without another keypress. The listening twin waits while the other talks (twins wait when the world is not in focus).
The player is blocked by both twins (`staff_blocks` already covered Tommy).
Verified (Temp, disc, synthetic tier-3 save with an active furniture-sale offer, capture build): both twins follow along
the furniture route set (settling at 50-80 units with separation); Tommy's spawn warning (B0004) is gone; the goodbye
dialogue (Timmy plate, message text) showed and one Space later the lower floor loaded. No WARN/ERROR apart from the gamepad
mapping line. Gates: fmt, clippy -D warnings, default build; no release build, no tests.
Not done: the forced greeting (`talk_start_tim`, SPEAK demo: after A at a display the selected twin walks over and
the player is held until the twin is within 85 units or 180 frames; needs the player-lock demo state), the start actions
(`start_act_idx`, action 61), weight 0 for the listening twin, clothing/carpet route sets were not driven (only furniture),
Tommy's name plate not screenshotted, and the lower-floor Nook has no goodbye either (south-door exit is still the E key).
