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

## Progress

- [x] furniture.rs import/pilot update, 2026-10-16
- [x] Bounded pilot verification/corrections and inventory audit, 2026-10-16
- [x] Furniture storage/actor-loading kernel corrections and focused regressions
- [ ] Wire corrected kernels into Bevy furniture gameplay and obtain playtest acceptance
- [ ] Remaining 193 source-module reviews (not 193 new imports)
