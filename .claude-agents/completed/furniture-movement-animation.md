# Furniture movement, rotation and matching animation

2026-10-16, codex. Next bounded furniture task after Philip's first-slice playtest.

## Scope

Player-owned room: source-backed pushing/pulling by one room unit and quarter-turn
rotation, with matching original player clips and furniture movement timing.
Audit retail contact, input, footprint, wall/player and surface-child rules before
choosing PC controls. Do not assume translated helpers fully express the C flow.

Source starting points (identified, not yet fully audited):
- ac_my_room_move.c_inc: aMR_ManageMoveBottun, aMR_MngPullDirectTimer,
  aMR_MngPushDirectTimer, aMR_SetPullMoveAnime, aMR_SetPushMoveAnime,
  aMR_FtrPull/aMR_FtrPush and rotation/collision helpers.
- m_player_main_push.c_inc, m_player_main_pull.c_inc,
  m_player_main_rotate_furniture.c_inc; trace their callers and original clips.
- Existing logic/src/furniture.rs, app/src/player_room.rs, input/house/character
  systems and runtime skeleton decoder. Canonical migration source remains
  122867f762122eb266f04e319af0f0c31d54f476; compare selectively with git show.

## Requirements

- Preserve stored contents, saved switches, original numeric IDs and rotations.
- Update foreground layers, ownership, rendering and collision together; reject
  blocked operations without partial changes. Preserve villager-room obstacles.
- Audit stacked furniture behavior; move supported children consistently with C
  or refuse unsupported cases explicitly until implemented.
- Define save/load and input-focus behavior during movement before wiring it.
- Runtime disc clips/models only; retain playable no-disc fallback and numeric data.
- Keep placement/pickup/open-close animations, per-type behavior, loose starter
  diary, room upgrades and full original storage scripts as separate follow-ups.

## Validation and acceptance

Check workspace, all-target clippy with warnings denied, format and build, one
Cargo process at a time. No tests added/run without Philip's request. Any GUI
verification uses disposable Temp towns and explicit --disc; preserve actual
saves/settings. Record implementation evidence separately from Philip playtest.

- [x] Audit original control/movement/rotation and player animation semantics for this adapter.
- [x] Implement bounded movement/rotation and matching clips.
- [x] Record checks and remaining source limitations below.
- [x] Philip playtest acceptance of the bounded movement adapter, 2026-10-17 ("works, next").

## Implemented, Codex, 2026-10-16

Hold G with empty hands beside furniture in the player's room. Movement toward
the piece pushes, away pulls, and sideways turns a quarter turn. Release G to
let go; release the direction before another turn. Arrow keys also work. G is
mapped centrally through Actions; exclusive Furniture focus prevents pockets,
doors, storage and normal locomotion from competing. P pauses; F5 snapshots the
committed destination; F9 discards the grip/presentation and restores the room.

New logic/src/furniture_motion.rs proposes room/layer copies using the existing
native ownership and banked-ID kernels. It validates the destination, updates
main/secondary occupants together and carries supported TypeA top actors with
their base. Held storage goods stay on their owning actor. Switch bits are
rebuilt at the new positions. Unsupported loose top items refuse movement.
The app checks the player's swept path and bounds before publishing the copies.
One f32 contact tolerance (0.002 world units) prevents stopped players being
rejected by a fractionally penetrated collision edge at the distant room origin.

Original cKF push1/pull1/Lturn1/Rturn1/hold_wait1 player clips and furniture
push/pull translation curves load from the runtime disc. Source speed0.5 per
60Hz tick, push/pull presentation delays and the ellipse rotation speed5.6 are
retained. Actor destination commits before presentation, following the C flow;
the original curves animate the movement and the existing decoder drives joints.
Player root X/Z is consumed by the checked actor move, rather than applied twice.
Five-frame morph counters blend clip changes and return to locomotion. Existing
room model roots are reused through moves/turns; collision updates preserve
villager-room obstacles. No-disc gameplay/numeric room saves remain available.

Reviewed original ac_my_room_{action,move,data}.c_inc: contact/hold input,
push/pull timers/target tables, fitting/unfitting surface children, TypeB pivot
reset/forbidden cells, TypeA/C neighbour checks, actor state delays, original
animation initialization/root motion and rotation ellipse. Reviewed the three
named player state files and c_keyframe's stop initialization (speed0.5).
Compared the pinned canonical furniture API; no wholesale module replacement.

### Deliberate limits

TypeB rotation uses its saved primary cell as pivot and a conservative swept
rectangle, not C's contact-half-dependent type_target_table/base-position
switching or complete friction tables. TypeA/C use source check_rotation
neighbours, but exact retail collision edges and contact alignment are still
approximated by the existing footprint boxes. Player translation follows the
furniture curve; this is not a full port of AnimationMove root/slip correction.
Movement sounds, switch/per-type behavior and exact retail hold/repeat timing
remain follow-ups. Large/long furniture and every model are not visually certified.
Birth/pickup/open-close, diary, upgrades and full storage scripts stay separate.

### Bounded evidence

No tests added/run. Workspace check, all-target clippy with warnings denied,
format check and capture build pass; final default build recorded in comms.
Ollama qwen2.5-coder:7b confirmed the mechanical positive-Y quarter-turn mapping
(z,-x), checked against the C rotation formula; an earlier larger extraction
request timed out and supplied no evidence.

Disposable preview: wardrobe4096 at cell34 with goods10240/8960/9216 moved to
cell18 and back. F5 during pull retained all three at the destination. Rotation
saved item4099 at cell34; F9/reentry restored it. Original pull/turn poses and
interpolated furniture were inspected in screenshots. Stacked box12536 and
tape7768 moved together34->18; switches512/512 became2/2. A further northward
push at the wall refused and retained both items and switches. Earlier attempts
with insufficient facing settlement or stale preview entity IDs are not evidence.
An initial synthetic loose-item setup correctly refused; its overflowing notice
was shortened. Screenshots/logs/synthetic saves stay in Temp under
bevy-crossing-furniture-motion-review. Actual save baseline in this turn:
04B547074C3F23DE979FCB158A2BD9F674796C07A5EC57D335821ADDF998003B.

Philip accepted the preceding placement/storage slice and subsequently reports
the bounded movement adapter works (2026-10-17). This does not certify every
model or close the explicitly documented retail pivot/contact/root-slip gaps.
Next bounded task: furniture-birth-pickup-animation.md. Migration and M4 remain open.

Final capture-build continuation: fresh process restored the saved box/tape stack
(two actors); left turn saved12537/7769 at cell18 and both switch masks stayed2.
Right turn restored12536/7768. Pull moved both to cell34 with masks512/512.
P during pull held player z=2999.42529296875 unchanged across two observations,
then resumed to the destination. F9 during an active push returned to town and
dropped the presentation. Explicit missing-disc startup remained playable in
town; G + W and F5/F9/F5 preserved all numeric home_furniture layers/switches.
This run did not recheck placeholder-house entry or all long/large-piece turns.
Both own preview processes closed. Actual save hash remained the current-turn
baseline above, settings untouched. No extracted assets entered the repository.
