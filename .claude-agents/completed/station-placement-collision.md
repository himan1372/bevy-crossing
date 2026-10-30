# Station placement, collision and platform exit

Philip reported on 2026-10-30 that the station is not fully on its platform,
has no collision, and an invisible boundary traps the player on the platform.
Scope: trace original station placement and walkable/collision geometry; repair
the station and platform exit, then verify with an isolated synthetic arrival.
Protect the real save; no tests without request. Update roadmaps/comms and
complete the claim after verification, commit and push.

Status: source investigation. The original ac_station.c constructor shifts x
by -20 game units, while all three current BuildingSpecs claim no shift.

## Source findings and implementation

- `ac_station.c::aSTA_actor_ct`: apply x -20 for all three station models.
- The seven-joint station skeletons place the clock hands; they were absent
  from POSED_STRUCTURES. Station variants/seasons also use different original
  part names (`t1_1`, `t2_2`, `shourt`, `ststion`). Read their actual skeleton
  display-list pointers and apply their original idle pose.
- The combined station mesh includes the platform. A single building box makes
  arrivals start inside an obstacle (and escape logic lets them cross its body),
  while also blocking entry from the stairs. Use the native station acre's
  collision heights and attribute walls instead of that whole-mesh box.
- Disc station backgrounds 0xEF/0xF0/0xF1 all place stone stairs at units
  (9,7) and (10,7): corners [8,4,4,8], attributes 34/33. The generic maximum
  walkable rise of 3 rejects them. Permit rise 4 for native stone bridge
  attributes 32..35, retaining edge and attribute wall checks. Platform rails
  and the tower's height cells remain solid.

Check/clippy/fmt and capture/default/release builds passed; no tests added or run.

## Runtime evidence

Temp folder: `%TEMP%/bevy-station-collision-20261030`. Final capture and default
builds pass. Inspected station-arrival.png and stairs-up-aligned.png: station
is aligned to its platform/stairway and clock hands sit on the clock face.
Ordinary input from the arrival end reaches (1.9863,5,-42.2210), then down the
stairs into town at (2.7176,4,-35.6979). An aligned return reaches
(2.2341,5,-41.2090). One unaligned return drifted to x3.449 and correctly stopped
at the outer wall; it was not treated as a staircase failure.

Blocked checks: the tower/front barrier stops south movement at z-42.0007,
the east wall stops at x2.9979, and the west front railing stops at z-42.0043.
`checks.json` records actual positions. F5 creates only the synthetic save.
One existing startup missing-model warning (iam_sugi_kpool); no ERROR.
Real save SHA256 remains C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
One scoped source review found no blocking issues. Release build passed (5m23s).
The owned preview is closed and task ownership is released at completion.
Read-only disc inspection confirmed seven joints and five display lists in each
of all six station skeletons (three models, summer/winter), including the original
variant spellings. No extracted assets are committed.
Live capture covers the generated town's selected autumn station; all three
model/season lists are source-driven, not all visually accepted by Philip.
