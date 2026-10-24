# Raffle ball size, drum height and clearance

Found during catalog-raffle-verification on2026-10-24 at372e29bf.

RaffleBall changes Hidden->Inherited and its translation follows the roll,
but the ball is not discernible in inspected screenshots. load_ball_frames
in app/src/raffle.rs builds via disc_town::build_model_full, which already
applies OBJECT_SCALE(0.00025) to mesh vertices. present applies GOODS_SCALE
(0.00025) to its parent again. Live child Aabb half width0.0625 yields only
0.00003125 world-unit diameter after the extra parent scale (4000x too small).
Original ac_garagara_ball.c_inc aGRBL_draw applies0.01 once in game units.

Authorized fix: use one model-to-world scale, preserving per-frame ball
texture/position/billboarding; inspect the visible ball leaving the drum and
rolling to rest in a synthetic Temp preview. Check ticket transactions remain
unchanged. Read texture/QA/Rust/Bevy skills first; no tests unless requested.

Evidence is in the completed/catalog-raffle-verification.md task. Parent
nook-tickets-catalog remains open and claude-owned. This note does not grant
permission to take over that claim or mark parent acceptance complete.

Philip authorized the ball fix, then reported the raffle object floating.
Additional confirmed cause: game_to_world subtracted centre(4,0,4), while
interior::spawn_shop_room uses(4,1,4). Live table-top Aabb is a horizontal
plane at model y960, rendered at960*(.0625/40)-1 = world y0.5. Drum was
y60/40 =1.5, one unit above it. A Temp-only -1Y diagnostic placed the drum
on the table. Fix the same origin conversion for drum and ball together;
Nook's XZ stage positions are independent and remain unchanged.

Philip also reported the player's hat clipping into the drum. Live posed drum
bounds extend to world z-2998.78125, beyond the table edge z-2999; the existing
table-only blocker permits the player up to z-2998.6. Include the original
frame1 drum geometry in the existing shop collision path on raffle day,
keeping its0.4 player radius. Transform every posed vertex with the original
draw scale0.01 and actor position(160,60,190), then game_to_world; collect its
exact XZ footprint. No guessed padding, hat edits, Nook placement changes or
ordinary-day drum obstacle. Height-grid sampling was insufficient for thin
projecting parts; direct bounds cover them without changing the room grid.

## Implementation and validation

- Ball parent starts at identity scale and no longer reapplies GOODS_SCALE.
  Mesh vertices retain the existing build_model_full conversion, preserving
  segment8 textures, billboarding, animation/ticket/result logic.
- game_to_world now matches shop shell centre(4,1,4); drum base worldy0.5,
  ball settles at0.55 on its tray. Original source actor position unchanged.
- raffle::obstacle_bounds poses original cKF frame1 and collects finite world
  XZ bounds. Shop adds that footprint only on raffle day, using its existing
  obstacle checks and0.4 player radius. Empty/missing assets returnNone.
- Independent read-only reviews of scale, origin and final exact-footprint
  collision found no actionable issues.
- Final synthetic previewPID30648, explicit ISO and dayoffset6, noon:
  normal walk toward drum stops atz-2998.38110, versus-2998.59985 previously;
  another1000ms forward leaves exact position unchanged. Original geometry
  footprint(2999.7,-2999.4375)..(3000.6,-2998.7813). Front-stop screenshot
  inspected for hat clearance; drum sits on tabletop. Talk/raffle still works
  from the side. Inspection framing used setup teleports outside the table.
- Inspected final captures at0.05/0.15/0.3/0.5/1.0s after observed draw start:
  white ball appears, moves onto tray and stops. Numeric Hidden->Inherited,
  parent scale[1,1,1] throughout, three distinct visible positions including
  airborne/rolling/settled state. No exact-frame timing claim.
- Actual final loss consumed first stack of5 from15; pockets now[0,2C4C,
  2C4C,0..], conditions0/wallet80000/catalog/prizes/stock/sales/room/mail
  unchanged. Full relevant fields compared in Temp helper. Initial helper
  required more than3 unique samples but captured exactly3; corrected to
  require those three observed motion states, with all transaction checks pass.
- F9 clears raffle hierarchy in town; reentry creates a fresh hidden ball at
  unit scale and drum at0.5. Final log has no ERROR/panic; expected unmapped
  gamepad warning only. Earlier height-only win preview also reproduced the
  already-open slab allocator error during prize-display refresh; appended
  that evidence to mail-render-slab-errors rather than claiming it fixed.
- Evidence/helpers/synthetic saves only in %TEMP%/bevy-crossing-room-critters/
  raffle-ball-fix-fe8bf84ca33c4e5c9798ec43a0a61068. Final evidence:
  stderr-164337.log, checked-before/after.json, verified-ball-motion.json,
  checked-front-stop.png, verified-raffle-ball-*.png and check_final.py.
  Earlier diagnostic/capture attempts are separate files/logs in same folder.
- Owned previews13268/4552/26404/30648 closed after verifying EXE paths.
  Actual save before/after SHA256 unchanged:
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
- Final check3.59s, all-target clippy -D warnings6.79s, fmt, capture28.94s,
  default30.71s and release3m37s pass. No test suites added/run.

Parent nook-tickets-catalog stays claude-owned/open; user acceptance remains
separate. Non-furniture catalog previews, exact clock fitting, remaining UI and
renderer allocation errors are follow-ups, not implemented here.
