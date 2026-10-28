# Eyeball Nook's present hand-over frames

Philip: "do the hand-over frames" (2026-10-28): capture and inspect the frames of Nook's present hand-over (sale-day balloon, same generic path as the password present) which earlier tasks (shop-password-arm, seasonal-balloon-handover) never eyeballed. Evidence first; fix what it shows.

## Result (2026-10-28, claude)
Driven in Temp (capture build, disc, synthetic tier-0 save, sale day with the balloon unclaimed and free pockets, game clock 15:00, fresh process). The
sale-day balloon uses the same generic `Request::nook_present()` hand-over as the password present. Screenshots every ~0.25 s starting at the
`hand-over Present` log line (24 frames, about 5.5 s), cropped and enlarged around Nook and the player, and looked at:
- Frame 0-1: the wrapped present (white box, red ribbon) is out beside Nook at the player's front, Nook's arm toward it.
- Frames 2-4: the present shrinks and moves down as the player puts it away; gone by frame 5.
- Frames 5-23: both stand idle, Nook blinks; nothing stuck, no leftover present, no flicker or T-pose.
Not established: the top-down shop camera hides Nook's arms, so the TRANSFER1 / TRANS_WAIT1 poses themselves and frame-exact sync with the 78-frame
kernel are not judged; only the present's placement and put-away are. Earlier runs where Space was pressed faster than about once per second did
not give the balloon (the talk restarted as an ordinary greeting); the slow cadence of the earlier QA is needed. No WARN/ERROR. No code change.
