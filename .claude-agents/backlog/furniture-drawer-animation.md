# Furniture drawer animation and obstruction

Philip authorizes the next bounded drawer slice after the player-home entrance
change: original drawer opening/closing, original player clips and retail
obstruction checks. Reuse the wardrobe/closet lifecycle and item transactions.

- [x] Claim, audit source aMR_CheckHikidashi/callers, common drawer constructors,
  player h1 clips, sounds and any existing logic translations/pinned snapshot.
- [x] Implement audited drawer families and source obstruction semantics, with
  a small in-game way to obtain a drawer for playtest (prior F4 cabinet issue).
- [x] Check, warnings-denied lint, format and appropriate capture/default/release
  builds sequentially; disposable synthetic-save GUI checks. No tests added/run.
- [x] Evidence separate from Philip acceptance. No codex-handoff.md edits.

Preserve actual save/settings/.gitignore, stacking/storage ownership/conditions,
atomic full-pocket/cancel behavior, wardrobe/closet behavior and no-disc numeric
preservation. No disc bytes/derived assets committed. M4/migration stay open.

## Implementation, 2026-10-17

All 16 original DRAWERS profiles use the audited common open/close constructor,
10-frame furniture animation and null draw callbacks. Added these to the runtime
joint renderer, alongside the existing 19 wardrobe/closet profiles. Original
player kagu_{open,wait,close}_h1 clips share the existing lifecycle; source drawer
sounds are 0x42E at opening, 0x42F at closing start and 0x3A when shut. The classic
drawers profile uses sum_clchest01 skeleton/animation names, matching its C
constructor/draw rather than assuming the profile filename is the rig name.

Source audit: ac_my_room_action.c_inc aMR_CheckHikidashi and aMR_JudgeDemoStart;
ac_my_room.c aMR_GetTypeBPlaceInfo/aMR_Direct2PlussUnit; common furniture move
and player hold/clip mappings. Existing logic and pinned Rustimal furniture.rs
had no reusable obstruction helper. New native drawer_front_clear checks both
rotated MAIN foreground cells in front of wide drawers. Furniture banks1/3 and
RSV_FE1F block; loose goods do not. Single-cell TypeA bypasses this check. Front
contact/facing and atomic storage transactions retain the existing adapter.

F3 in the player's house adds a blue bureau (wide drawers) to the first empty
pocket, resolved from runtime profile metadata. Full pockets refuse; condition
bits and other slots stay intact. F4 still supplies the blue cabinet. Use normal
I -> leaf -> Place in room, then face the front and E. No extracted assets added.

Blue bureau initially disappeared because original large joint translation and
opposite vertex offsets put its static bind-space AABB offscreen. Diagnosed with
a disposable culling override, then fixed using installed Bevy0.19.1 per-joint
DynamicSkinnedMeshBounds. Only a bounds-generation error falls back to disabling
culling for that part, with a warning. Final fresh-process previews use the real
dynamic-bounds implementation, without remote culling overrides.

## Bounded implementation evidence (not Philip acceptance)

Workspace check, all-target clippy -D warnings -j1, format and capture build pass.
Final default workspace build and release build (-j1, 2m32s) pass. No tests added
or run. First default-build
attempt hit an executable lock after sandbox denied closing our preview; closed
only own PID30144 with escalation and repeated the build successfully.

Disposable Temp/bevy-crossing-furniture-drawer-review GUI, synthetic saves:

- F3 inserts native item4196 into an empty slot; other goods and condition bits
  preserved. Pockets displays blue bureau. Normal placement saves owner34=4196
  and footprint35=RSV_FE1F, consuming the leaf once.
- Final closed/open models visible; original player h1 reach and drawer travel.
  Put apple stores it at owner34/layer1; take returns it once and clears that
  content. Main owner/footprint cells and pocket condition bits remain intact.
- Either wide front cell blocked by furniture refuses opening. Reservation
  marker also refuses; loose apple permits opening. Refused interactions leave
  room/pockets unchanged. Correct rotated front blockers refuse at all four
  orientations; a clear sideways drawer opens with the matching player reach.
- Single-cell classic drawers render and open despite a marker in front,
  exercising the source TypeA bypass and skeleton alias.
- Pause during opening, pause-save and F9 restore the saved content without a
  stale demo/modal. Full storage, full pockets and F3 with full pockets refuse
  without changing room, inventory or condition bits. Put-pockets cancellation
  and take-list cancellation preserve room/pockets/conditions and close drawers.
- Initial unconfirmed re-entry attempts were excluded: an incorrect exterior
  facing left the app in Town. Repeated with confirmed house entry and the
  source house's diagonal door facing. Confirmed screenshots/logs and numeric
  snapshots stay in Temp, including final dynamic-closed/open and capacity shots.

Earlier entrance slice separately verifies explicit no-disc placeholder entry,
exit, save/load and numeric preservation. New grant/storage animation requires
a disc and does not replace the no-disc numeric room path; a fresh no-disc GUI
cohort was not repeated for drawers. All drawer previews closed before builds.
Actual save SHA256 remains
DD89745CCFEDDEAEA619A1C931FEE5233EC533C1860E3B5A910A00B8121E5721.
Settings/.gitignore and codex-handoff.md preserved.

Local Ollama qwen2.5-coder:7b reviewed the bounded obstruction helper. Its claimed
offset mismatch lacked a valid counterexample; direct C footprint/direction and
predicate review rejected it. Do not count that response as validation evidence.

## Remaining scope / acceptance

Philip has not playtested this new drawer or entrance slice. Original BUBU
blocked-action effect is represented by a brief blocking notice; exact source
feedback, contact/root snapping, per-type constructors outside the audited common
family, full scripts, long-piece pivots, birth dust and loose-item rendering
(including starter diary) remain open. This does not complete M4 or migration.

## Philip feedback and source follow-up

Philip reports that only the blue bureau's bottom drawer opens and is content
with that if original behavior. Checked original int_sum_blue_lowchest01.c:
one base display-list joint plus one door_1 drawer joint; the base includes the
remaining drawer fronts, and the sole drawer joint is animated. Its constructor
uses exactly this animation/common1..10 flow, with no drawer-selection callbacks.
The observed single bottom drawer is expected; do not add independent upper
drawer controls. This feedback does not certify every storage transaction or
drawer family. Proposed next bounded slice: source-backed loose-item rendering,
starting with the omitted starter diary, preserving furniture/storage layering.
