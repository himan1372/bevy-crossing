# Migrate rustimal-crossing modules to bevy-crossing

2026-10-16, muse; verification updated by codex at Philip's request.
Pilot imported in 38c184b1; verification corrections and boundaries below.

## Background

Philip clarified 2026-10-16: bevy-crossing is Animal Crossing in Bevy. The 194
Rust modules in rustimal-crossing were developed with extensive research specifically
to be moved here. The C wiring (Waves A-F) was the interim validation step.

## Scope

Review 194 source module files for migration/merging into `logic/`. Corrected
inventory uses full relative paths recursively, excluding crate-root lib.rs:

- **104 source-only paths** — review dependencies and production suitability;
  these are not automatically independent additions (test_c_stubs.rs is a C
  test harness, not gameplay code to port into the production library).
- **90 overlapping paths** — 88 differ, 2 match exactly in the audited snapshot.
  Compare behavior and callers; a difference does not establish which is newer
  or more faithful. All nine graphics/*.rs files already exist on both sides.
- **27 Bevy-only paths** — preserve, including audio, field/AI and source tables.

The original 113/81/10 counts mixed directory levels. Pinned source is Rustimal
122867f762122eb266f04e319af0f0c31d54f476; Bevy inventory snapshot a2005e7,
before verification corrections. Full list: [inventory](../../docs/rustimal-migration-inventory.md).

## Pilot import (done), retail behavior review (partial)

`furniture.rs`: stripped `#[no_mangle] pub extern "C"` wrappers, exposed native
`pub fn`. Includes the 2026-10-16 `save_angle` bitflag fix in the pc_ compatibility
wrapper. Furniture was already in Bevy before this commit, so this was an update
to an overlapping module. Do not treat wholesale replacement as the merge pattern.

## Per-module steps

1. Claim a bounded task with task-claimer.sh; pin the source commit and path.
2. Compare source, existing Bevy module and original C, including actual callers,
   dependencies, tests and all existing Bevy fixes (not only ECS adaptations).
3. Merge verified changes; preserve Bevy graphics/audio/runtime behavior. Do not
   blindly replace a file or assume the source side wins because it is newer.
4. Remove ABI exports/calling conventions only where no retained ABI needs them;
   keep required layout and caller semantics. Expose only the native API needed.
5. Record porting separately from Bevy integration and Philip playtest acceptance.
6. Run focused tests when Philip authorizes testing/implementation verification,
   plus workspace check, all-target clippy and format/build gates as appropriate.
   This verification request authorizes the pilot's tests; standing instructions
   still apply to future work.
7. Update evidence and remaining gaps, release the claim, commit/push. Do not
   mark the entire migration or M4 complete after one module.

## Verification

Before this review the pilot had only been inspected. Original import compared
with pinned canonical source: only a preamble and ABI removal differed; tests
were copied unchanged. Original C was inspected for rotation macros, footprint
offsets, save-angle flags, switch bitfields, room budgets, actor loading and
storage save-back. The following evidence is bounded, not full C equivalence:

- Baseline workspace check and 10 furniture tests passed. Baseline all-target
  clippy failed on reintroduced nested OnSurface condition; fmt check failed.
  Restored the existing collapsed condition and formatting.
- New native/compatibility save-angle regression failed before correction:
  (sin=0.9,cos=-0.9) returned East from the native helper and West from the fixed
  wrapper/C OR chains. Unified the native helper and wrapper. These are non-unit
  API inputs, not evidence of a normal in-game rotation failure. Regression also
  covers threshold, negative and NaN inputs.
- Final rustimal-logic suite passes: 521 unit + 10 integration tests, including
  all 11 furniture tests. Workspace check, all-target clippy and fmt pass.
  No actual game saves/settings were read for mutation, loaded or rewritten.
- Furniture model rendering still uses ShapeType and housing uses FurnitureSize;
  importing the rest does not wire placement/storage/moving into Bevy gameplay.

### Inherited source gaps found by the pilot verification

- keep_items_to_fg scatters held items into first-free cells in two layers and
  does not clear actor items. C aMR_KeepItem2Fg writes each item at its owner's
  unit in successive layers above the actor and clears that held slot. Existing
  storage_roundtrip_to_fg tests only the abstraction, not retail save/reload.
  Its current signature cannot express C's actor position/layer semantics.
- make_furniture_actors_from_layers treats any nonempty non-FE1F item as furniture,
  unlike C aMR_MakeOneFurniture's ITEM_IS_FTR and bank/actor-budget checks.
  These helpers are not called by the Bevy app; fix before wiring them.

Both gaps existed in canonical Rustimal and pre-pilot Bevy, not just Muse's
update. They were corrected in the subsequent authorized slice below. Copy
fidelity and passing tests are not proof of full retail equivalence.

### Storage and actor-loading correction, 2026-10-16

Philip authorized this follow-up after verification. Original C reviewed:
ac_my_room.c::aMR_KeepItem2Fg, aMR_MakeItemDataInFurniture,
aMR_MakeOneFurniture/aMR_MakeFurnitureActor, aMR_SetFurnitureType and
aMR_FurnitureCt; ac_my_room_move.c_inc::aMR_UnitNum2FtrItemNoFtrID;
m_room_type.c::mRmTp_FtrItemNo2FtrIdx/mRmTp_FtrIdx2FtrItemNo; m_name_table.h.

- keep_items_to_fg now receives a mutable actor and optional destination-layer
  views. It writes held slot n at the actor's primary cell in layer actor.layer
  +1+n, keeps empty/missing-layer holes, and clears only successful writes. A
  layer-0 actor can use three layers; a layer-1 actor only two. Invalid cells or
  unavailable layers preserve held items. As C does, nonempty owner destination
  cells are overwritten; unrelated cells are not touched.
- Added restore_stored_items_from_fg following the C layer/row order and owner
  footprint lookup, with caller-supplied aFTR_IS_STORAGE profile predicate.
  Ownerless/non-storage items, RSV_NO and RSV_WALL_NO remain unchanged. Successful
  transfers clear the foreground cell and restore the corresponding held slot.
- make_furniture_actors_from_layers now builds the persistent actor subset in
  the provided room, rather than returning unfiltered tuples. It scans main then
  secondary, validates ITEM_IS_FTR, respects existing actors/budget/first free
  slots and requires a successful profile/bank callback before using a slot.
  The callback owns GetFurnitureBank2's cached-bank/load decision; no actual
  disc-bank loading was added to logic. Original B0 profile rotations, angles,
  primary cells and saved switches are retained. Other per-type constructors,
  haniwa initialization, rendering and runtime asset ownership remain separate.
- Occupancy now has independent main/secondary grids. Malformed/overlapping
  footprints are rejected without partial occupancy mutation; C's unchecked
  stores are not reproduced. Releasing one actor preserves other layer owners.
- Found/fixed related bank encoding: native saved-item conversion previously
  treated profile indices as raw item bits. It now maps FTR0 (0x1000) and FTR1
  (0x3000/index0x400) correctly and rejects non-furniture IDs when decoding.
  set_furniture_to_fg uses that conversion. Low-level raw rotation macros and
  compatibility wrappers remain distinct and unchanged.

API changes are deliberate for the new source semantics: saved-item decoding
returns Option; the loader accepts layer views, mutable room and bank callback;
storage accepts actor/layer views; occupancy is indexed by layer first. Existing
Bevy code uses only ShapeType/FurnitureSize, so no gameplay callers or user-save
schema were changed. Keep these signatures when merging future Rustimal callers.

Focused regression coverage: both furniture banks/rotation boundaries and invalid
IDs; source scan order/first-free slots with existing actors; failed banks/zero
budget; two-layer occupancy/release; B0 rotated footprints/saved switches; invalid
footprints; primary-cell storage save/restore with a held-slot hole; missing
layers/invalid cells; layer-1 capacity; footprint owner/non-storage/ownerless and
reserved-cell restoration. All 18 furniture tests passed in the focused run.
Final full logic suite: 528 unit + 10 integration tests pass (538 total), with
workspace check/all-target clippy/format clean. Build evidence is recorded in
comms.txt after completion. No real town save was loaded or rewritten.

These are logic-layer corrections, not in-game furniture placement, storage UI
or a Philip playtest acceptance. Integration remains the next separate task;
the migration and M4 remain open.

### Coordination evidence

Muse's pilot log entry uses status=completed rather than action=complete and the
pilot has no task note. The current task-claimer.sh expects action records; that
entry alone cannot establish completion through the script. Historical claim
records were not manually edited in this verification. Future claims/releases
must go through the script; compatibility cleanup remains a separate follow-up.

### First playable furniture slice, Codex, 2026-10-16

Philip authorized connecting the corrected kernels to placement and storage.
A PC crash interrupted research before implementation; the claim was recovered
through task-claimer.sh. This is one bounded gameplay integration, not completion
of the migration or M4.

Implemented in app/src/player_room.rs, furniture.rs, interior.rs, house.rs,
pockets.rs/pockets_disc.rs, input.rs, dialogue.rs/msg_window.rs and save.rs:

- Player-owned small room only: pockets offer Place in room for ordinary
  furniture. Profiles, original IDs, meshes, textures, height/shape and separate
  aMR_layer_set_info values are read from the runtime disc. Missing model support
  refuses the transaction and retains the pocket item. Villager rooms are not
  editable. The original tape deck and orange box are initialized from disc data.
- Placement uses the C aMR_GetPlayerLookAtUnit adjacent-unit table/angular
  offsets, source opposite-facing orientation and special item offsets, native
  footprint/free-slot/budget checks, with room-wall/player-circle guards. The
  inherited native judge's straight five-cell approximation is deliberately
  not used as retail placement. 1x2 fallback searches all rotations at those
  units; this retains the translated BUGFIXES-style fallback, not the retail C
  stale-target loop. Full TypeC square_offset behavior and collisionless
  under-player placement remain follow-ups. Supported small objects can stand
  on non-storage surfaces; the base cannot be picked up while something is on it.
- Backspace picks up aimed furniture, prioritizing layer 1 and using source
  50/56-unit ranges. Furniture plus all held contents must fit before any item
  moves. Empty hands are required. E opens drawers/wardrobes/closets; Put in
  reuses retail pockets, Take out lists up to three stored items plus Quit.
  Layer-1 storage has two held slots. Full/cancel paths keep both sides intact.
- Storage selection follows mSM_check_item_for_furniture: normal-condition
  ITEM1 goods only, excluding fish/insects/turnips, exercise cards and knife/fork.
  Furniture leaves are excluded by the source. Music-player interaction remains
  separate; the general storage predicate still preserves its saved contents.
- make_furniture_actors_from_layers and restore_stored_items_from_fg rebuild
  room ownership. Potential stored leaves above storage are masked only from
  the secondary actor scan, then restored from the intact foreground layers.
  Model roots update only when a piece changes. Player collision refresh keeps
  villager-room obstacles. The no-disc room retains its existing placeholders.
- Additive optional home_furniture saves four numeric 16x16 layers and switch
  masks. Missing field initializes old saves' original room; explicit empty
  layers remain empty. Malformed layer lengths fail deserialization. F5 calls
  keep_items_to_fg on cloned actors/layers so live storage never empties; F9 and
  startup restore layout/storage. Without disc profiles, numeric layout data is
  retained through save/load instead of being replaced by defaults.

Storage currently uses the existing disc message/choice renderer with concise
adapter text, not the complete original room-message scripts. Furniture has no
speaking actor/nameplate. Transactions are immediate; source birth/pickup/door
animations, pushing/pulling/rotation, switches and per-type behavior, loose items
(including the previously omitted starter diary), upgrades, complete TypeC
placement and exact source storage dialogue remain open. Nothing here is Philip
playtest acceptance. Handoff prompt: docs/codex-handoff.md.

Disposable manual evidence so far: original wardrobe placement/removal; ordinary
apple Put in/Take out; repeated F5 keeps live contents; F9 restores three distinct
stored items at the owner's cell in layers 1/2/3; full-pocket pickup/retrieval,
full-storage refusal and Escape cancellation preserve inventory/layers; pickup
with four free slots returns wardrobe, apple, shirt and paper atomically and
clears all four owner cells. A fresh process restored the wardrobe and all three
contents. Explicit empty layers loaded without respawning starter items. Orange
box + tape deck placement saved in main/secondary layers at the same cell,
rendered at the original surface height, and Backspace recovered the deck before
the box. Collision held the player at z=3001.366 in front of the wardrobe; after
pickup, W crossed its former space to z=2999.358. A room filled with 14 boxes
refused placement, retained the pocket wardrobe and kept all 14 actors. Explicit
missing-disc preview entered the placeholder house, opened/closed pockets and
saved/loaded/saved without changing the numeric room layout or item. Early menu
attempts before facing/reveal settled were retried and are not evidence.

Workspace check, all-target clippy with warnings denied, format check, capture
build and final default workspace build pass. No tests added/run in this slice;
historical kernel-test counts above are separate. All preview processes closed.
Real save SHA256 remained
ECD1F9A7D016A0E5862882A704FDB3B88B16AAB51183CA1E7B29084139359BDC.
Settings untouched; no extracted bytes, screenshot assets or Temp saves committed.
Philip acceptance remains pending. Full TypeC/1x2 variety, every model, villager
collision traversal and all source-specific behaviors were not manually certified.

### Philip playtest feedback, 2026-10-16

Philip reports collision works, furniture is remembered, saving works and
"everything checks out so far." Record acceptance of the current playable slice
on that basis; this does not certify every model or remaining retail behavior.
Next bounded proposal: furniture-movement-animation.md (push/pull/rotation and
matching original animations). No new implementation or test evidence here.

## Progress

2026-10-17: Philip reports the bounded movement adapter works. The next slice
adds runtime original leaf icons, source birth/pop/bye scale kernels and pickup1
clip, preserving atomic inventory/storage and save/load. Full evidence, remaining
dust/material/collision limits and Philip's bounded acceptance are in
furniture-birth-pickup-animation.md. No additional module migration batch started.

2026-10-17: Philip reports "wokrs good whats next" after placement/pickup
presentation. Next bounded proposal is furniture-storage-animation.md: source
wardrobe/closet opening and closing around the existing storage interaction.
Philip then authorized implementation ("lets get it"). Nineteen audited common
wardrobe/closet constructors now retain runtime cKF joints and original player
open/wait/close clips around the accepted storage transaction flow. Source timing,
front-contact gating and safe pockets/notice completion/reset are connected.
Check, all-target lint, format and capture/default builds pass; bounded disposable
previews cover transactions/cancel/capacity, rotated doors, moving stored goods,
stack pickup, pause/save/load/restart and no-disc numeric preservation. No tests
added/run. Full evidence/limits in furniture-storage-animation.md; Philip's
acceptance of this new slice is pending. M4, migration and fidelity gaps stay open.

Furniture follow-up: bounded G + direction push/pull/rotation and original player
clips are implemented separately in furniture-movement-animation.md, including
atomic stacked actors/storage/switches and save/load presentation rules. Exact
TypeB contact-dependent pivots and broader fidelity remain open. Philip's first
slice acceptance was followed by separate movement acceptance ("works, next").

- [x] furniture.rs import/pilot update, 2026-10-16
- [x] Bounded pilot verification/corrections and inventory audit, 2026-10-16
- [x] Furniture storage/actor-loading kernel corrections and focused regressions
- [x] First player-room placement/pickup/storage/save integration (bounded scope above)
- [x] Philip playtest acceptance of the current first playable furniture slice
- [ ] Remaining furniture gameplay/source fidelity
- [ ] Remaining 193 source-module reviews (not 193 new imports)

2026-10-17 continuation: Philip authorized a walkable brown home entrance and a
smaller exit area requiring doorway facing, then original drawer animations and
obstruction. Entrance source geometry/bounds and finite facing-required door are
implemented; disposable disc/no-disc checks cover walk/edge stops, carpet and
wrong-facing refusal, correct-facing exit and numeric preservation. Drawer slice
adds 16 audited common constructors, original h1 clips/sounds and aMR_CheckHikidashi
two-front-cell checks (TypeA bypass). F3 provides a blue bureau without replacing
occupied pockets. Per-joint dynamic skin bounds fix the bureau's original offset
rig culling; classic drawer rig alias follows C. Check/all-target lint/fmt,
capture/default builds and bounded disposable placement, transactions, blockers,
rotations, capacity, cancel and pause/load pass. No tests added/run. Final release
build passes in2m32s; detailed limits are in furniture-drawer-animation.md. Philip
acceptance remains pending for these slices; no module batch, M4/migration closure
or codex-handoff update. Existing accepted movement/birth/pickup scope preserved.

Philip subsequently accepts the walkable entrance area and observes only the
blue bureau bottom drawer opening. Original model has one animated drawer joint;
upper fronts belong to the base, so this is expected. Next proposed bounded
source review is loose-item rendering, starting with the omitted starter diary;
not yet claimed/implemented. M4 and migration remain open.

Philip authorized that bounded diary slice, requesting usage efficiency. Runtime
diary00..15 now reuse existing shop goods models for saved floor/surface layers;
source starter diary applies only to default rooms. Existing Backspace handles
visible diary recovery/full-pocket refusal without taking its supporting box.
Storage contents stay hidden. Check/lint/fmt/capture and disposable GUI evidence
are in room-loose-diary.md; no tests added/run, final builds recorded there.
Other loose families, placement/writing UI and exact pickup presentation remain
open; no migration batch or M4 closure, Philip acceptance pending.

2026-10-18: diary-placement.md follows Philip authorization: existing pocket
action places normal diaries on source-eligible front surfaces or empty floor,
using the existing surface kernel and original local search order. Blocked
placement is atomic; disposable surface/floor/F5/F9 and refusal checks pass.
Exact throw/wall-ray/slope fidelity and other loose families remain open. No
tests added/run or new module migration batch; Philip acceptance pending.

Stationery-room.md: Philip authorized the next small family. Shared diary path
now accepts stationery0x2000..0x20FF with encoded design/quantity intact. Original
shared paper model, surface/floor, pickup, F5/F9, full/blocked refusal and hidden
storage checked in disposable preview; no tests added/run. Normal-condition and
mail/selling/storage routing preserved. Evidence/final builds in task note;
Philip acceptance pending. No M4/migration completion or new module batch.

Fruit-shell-room.md: Philip authorized both families together. Shared loose path
adds six fruit and eight shells (coral included), using original indoor goods
models; mushroom/candy excluded. Gallery and bounded surface/floor/pickup/F5/F9,
hidden storage/full-pocket/blocked checks pass. No tests added/run. Final build
evidence in task note; Philip acceptance pending, M4/migration remain open.

Furniture-carry-loose.md follows Philip authorization: supported loose surface
goods accompany push/pull with existing native relocation and animation progress.
Copied-layer destination checks precede publish/player-path validation. Four-cell
and single-cell, mixed top furniture/goods, switches, mid-motion save/load and
blocked/unsupported/turn refusal evidence recorded; no tests added/run. Loaded
rotation/source pivots remain open. Philip acceptance pending; no M4/migration
closure or broader module import batch.

Furniture-turn-loose.md: Philip authorized single-cell/square loaded rotation.
TypeA/TypeC goods now orbit with the base, respecting original goods yaw flags;
fruit keeps model facing. Copied transient angle state and numeric cells commit
atomically. Saved positions persist; loose-facing is runtime-only/reset on F9 or
fresh process, no DTO extension. Long loose-loaded turns still refuse. Bounded
left/right, push/pull-after-turn, mixed stack, placement reset and refusal evidence
in task note; no tests added/run. Philip acceptance/M4/migration remain open.

2026-10-19 Codex diary-writing: partial native integration of source-only
rust/src/diary.rs from pinned122867f into logic/src/diary.rs. Reconciled original
m_diary/m_diary_ovl semantics for space-filled992-byte entries, backward text
length, strict192px wrapping/newline205, six-line and ten-line scrolling and
Yes/Rewrite. Safe slices replace unused unsafe ABI wrappers; no source tests or
card/checksum/repeat/easing follow-ups imported. App wires owner monthly notebook
read/edit/confirmation/persistence through placed surface diaries and existing
disc frame/font/input/save paths. Full source module equivalence, retail calendar/
keyboard graphics and other-player locks are not claimed. See diary-writing.md
for final runtime/gates/limits. Original inventory snapshot counts remain dated
snapshot data; this does not finish migration/M4 or establish Philip acceptance.
