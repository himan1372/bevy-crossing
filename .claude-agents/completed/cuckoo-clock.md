# Cuckoo clock (sum_log_hatoclk)

Source: `src/furniture/ac_sum_log_hatoclk.c` (`aSumLogHatoclk_*`), audited in
[docs/furniture-callback-audit.md](../../docs/furniture-callback-audit.md) §6.
Profile index 0xC1, item 0x1304. A three-state mechanism starts on minute 0
(hour-count table, noon and midnight = 12) or minute 30 (one call),
alternates stop/play and rearms only after leaving both minute windows;
entering during a trigger minute can start it. Sound 0x133 at frame
GETREG(MYKREG,33)+2 (resolve the register default). Reuse the native clock
kernels (logic::furniture_clock) and the rigs/clocks skinned builders.
Planned, not started.

## Implementation (claude, 2026-10-19)

Source re-read: speed is 1.0 (not the 0.5 system rate); ct plays once so the
idle pose is frame 2; state 0 arms on minute 0 (pappo_count_table[hour]) or 30
(one call); state 1 checks `frame == MYKREG[33] + 2` (register never set, so
2.0) before play; cKF stop_proc clamps to the end and reports STOPPED, then a
speed-0 frame, then restart with frame 1 and calls--; state 2 rearms once the
minute leaves 0/30. DwBefore hides joints 7/8; DwAfter draws short/long hands
at those joint origins with yaw, Rz(0x4000 - rad_hour/rad_min), +1 unit z.
logic::furniture_clock::Cuckoo (+ CUCKOO_CALLS, CUCKOO_SOUND_FRAME);
app::furniture::cuckoo (skinned body without 7/8, hand anchors under the root
updated from the composed pose, 60 Hz ticks while the room is shown; state
re-created on entry as the actor is constructed there; 0x133 via se_at with
the clock birth/removal gate). No save change.

## Verification (Temp only; no tests)

Real save C15154C0... unchanged. Preview with the clock and a blue clock:
body renders, hands match the blue clock at 20:37 and after F6 +3h at 23:37.
At 21:00:00 real time (game 00:00 with +3h) a temporary log recorded 12 calls
over 4.7 s, counting 12..1, then idle; Philip reported "it worked right at
21:00". Temporary log removed. Gates: check, all-target clippy -D warnings,
fmt, capture, default 45s, release 3m09s.
Limits: half-hour single call not observed live (same code path, count 1);
pickup/F9 not re-driven (ordinary furniture placement path).
