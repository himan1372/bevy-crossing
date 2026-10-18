# Contact-dependent long furniture turns

Philip authorized this bounded follow-up on 2026-10-18 ("ok le go").
Audit original contact-half pivot selection, TypeB transforms and rotation
clearance, then connect long turns with supported loose/mixed surface tops.
Preserve TypeA/C, storage ownership, switches, atomic refusal, save/load and
no-disc numeric data. No tests without request; use workspace check, clippy,
format/builds and disposable preview evidence. Preserve actual save/settings.

- [x] Source audit and bounded implementation.
- [x] Validation and evidence, distinct from Philip acceptance.
- [x] Update roadmap/comms and release claim.

M4 and migration remain open.

## Implementation, 2026-10-18

Reviewed original aMR_DecideKurukuruDirect, PlaceKurukuruFurniture,
ResetFurnitureType, RotatePermission, TypeChangeByRotate and SearchNextSituation,
their contact/collision size tables, MoveShapeCenter/base-position patterns,
SetFurniture2FG and Fit/GetItemPosOnMovingFurniture. Pinned migration furniture
has no contact-pivot implementation to reuse; existing native relocation stays
the transaction owner. Local Ollama executable unavailable; direct source review.

Native LongTurn maps all four TypeB shapes, outward sides and contact halves
from type_target_table to an endpoint pivot. C's temporary shape/base-position
switch is expressed as an orbit of the canonical saved primary cell; angle and
numeric footprint stay in the existing format. Complete-obstacle checks use
the source rotated-distal/diagonal swept cells. Conditional friction checks use
the two source neighbours and only refuse another check_rotation profile.
Player clearance retains the adapter's conservative radius/rectangle check.

App computes the directed contact half from player position and the footprint
midpoint (the source collision shapes force unit scale), passes the turn plan
to native relocation, and animates base/children/loose goods about that pivot.
Supported long loose-loaded turns are enabled; unsupported tops still refuse.
Copied actor/layers/loose-angle state publish only after ownership, destination,
source clearance and player-path validation. Push/pull and TypeA/C share their
existing paths. No save fields, extracted assets or gameplay item grants added.

## Bounded evidence

Workspace check, all-target clippy with -D warnings, format and capture build
pass. No tests added/run. Default/release builds also pass (release 3m24s).

Disposable Temp/bevy-crossing-long-turn-review, own preview PID14604:

- Long country table4512 carries diary11008/paper8387. South/right contact turns
  about cell35: primary34->19 (4515), paper stays35, diary moves19, switch512->4.
  Reverse left turn restores all room layers/switches/pockets/conditions exactly.
- Paused mid-reverse screenshot/transforms show the primary/diary orbiting cell35,
  paper fixed on the pivot, and matching yaw. Pause Save snapshots the committed
  destination; resumed movement finishes aligned. Enter on pause Save resumes;
  an initial attempt accidentally paused again and was excluded/repeated.
- Primary-end left turn keeps primary34 (4513), paper35->18. All four initial
  orientations exercised with primary-end left and opposite-end right turns;
  each saved the expected distinct canonical footprint and relocated switch.
  Initial insufficient-facing/house-entry attempts were excluded and repeated
  after longer direction settlement with confirmed interior entry.
- Mixed tape7768/paper8387 becomes tape7769 at34/paper18, preserving both switch
  masks512. Fruit10247 orbits but its model yaw remains identity; diary yaw turns.
- Diagonal swept MAIN cell19 loose diary, occupied secondary destination18 coral,
  and unsupported mushroom top each refuse with exact room/pocket/condition
  preservation. TypeA box and TypeC square loaded turns still commit correctly.
- Long blue bureau4196 with three stored items10240/11008/9499 at owner34 turns
  about the far end to4199 at19, moves all storage layers to owner19, relocates
  switch512->4, and preserves the complete room across F9/reentry/F5.
- Screenshots inspected; synthetic saves, logs and helpers remain Temp-only.
  Preview log has no WARN/ERROR. Actual save SHA256 remains
  26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

Source friction branch reviewed/compiled, not separately certified with a
check_rotation furniture cohort. No fresh no-disc GUI cohort; unchanged numeric
save preservation and no-disc fallback stay intact. Exact contact-edge range,
player root/slip, conservative player sweep and special furniture behavior remain
separate fidelity limits. Philip playtest acceptance is pending.

Final default workspace build passed. The first attempt hit the still-running
own preview executable lock after sandbox Stop-Process was denied. Closed only
verified own PID14604 with escalation; repeated default build passed. No unrelated
process was stopped. Handoff/settings/.gitignore preserved.

Final release build passed in 3m24s; actual-save hash still matches baseline.
Claim released; Philip acceptance pending, M4/migration open.
