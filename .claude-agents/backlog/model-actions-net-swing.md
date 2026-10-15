# model-actions-net-swing — first held-tool action slice

2026-10-15, codex. Planning only; implementation and Philip playtest pending.

Read Model-research/report.md and index.html before choosing this slice.
Next task: a normal net held on the player, one complete empty swing, stop/recovery,
and return to locomotion. Catch presentation and full tool families follow later.

- [ ] Trace held-net setup, player/tool clips and procedural joint adjustment.
- [ ] Load the normal net rig/materials at runtime from the user's disc.
- [ ] Bind the net to the source hand transform and share the action clock.
- [ ] Drive one non-looping player/tool swing and source-backed recovery.
- [ ] Gate movement/repeated actions and cancel safely on load/scene/tool changes.
- [ ] Retain usable no-disc action behavior.
- [ ] Compile/lint/format/build; no tests unless Philip requests them.
- [ ] Philip: visual/action acceptance below.

## Research compared with current code

The report describes linked Vtx/Gfx lists, Dolphin opcodes, cKF part matrices,
instance texture segments, DOL/REL storage and no LOD found. Those foundations
already exist in disc/src/rom.rs, assets.rs, app/src/disc_gfx.rs, skeleton.rs,
character.rs and n64_material.rs. The report does not demonstrate a broken rig,
mesh, material or transform in this port. render-look comparison is still open.
Do not restart M0–M2 or the main M3 model-loading work.

Our readers decode big-endian fields directly; the C ports' swap classes are
not a requirement to add byte-swapping here. ARC files still supply textures:
game_disc.rs reads face_boy.bin, cloth textures/palettes and NPC metadata.
The research's unenumerated ARC contents are already partly understood here.

Read both research files. The four notes/source-*.md files named at the end
of report.md are absent from Model-research. Several external documentation
links could not be fetched. Read accessible upstream cat_1.c,
gbi_extensions.h and arc_tool.py, plus the Nookipedia page; corroborated the
relevant model and action details against Philip's local C source. Do not
claim every external source or missing note was read. Unused-content counts,
all-region coverage and a universal absence of LOD remain research claims.

## Why this slice comes first

character.rs PLAYER_ANIMATIONS has wait1/walk1/run1 only; animate_characters
selects only locomotion. ecology.rs tool_interact resolves the net on E
immediately with a distance/facing/nearest candidate test and no held net model.
One empty swing/recovery exercises attachment, independent tool animation,
one-shot completion and input ownership without the rod's bobber/line/bite chain.

## Source and reuse

Original root: C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port.

- src/game/m_player_main_swing_net.c_inc: NET_SWING1 setup at speed 0.5,
  capture only after frame 6, effect at (8.5, 9], stop/pull outcome routing.
- src/game/m_player_main_stop_net.c_inc: stop waits for item readiness before
  returning to wait. Do not invent a fixed recovery delay from clip length.
- src/game/m_player_draw.c_inc: right_hand_mtx from the hand callback.
- src/game/m_player_item_net.c_inc: procedural dummy-net joint adjustment,
  including different behavior before/after swing frame 5. This is needed
  alongside sampled clips; a rigid prop plus player clip is incomplete.
- src/data/model/player_anim.c and disc/data/foresta.sym identify
  cKF_ba_r_ply_1_net_swing1 and cKF_ba_r_tol_net_1_net_swing1.
  Trace tool skeleton/actor selection before selecting model symbols.
- logic/src/player_tools.rs: net_swing_outcome and priorities.
- logic/src/ecology.rs: frame gate/capture helpers. net_capture_test explicitly
  approximates the C endpoint/radius checks: audit/fix that translation before
  using it for a later retail capture integration. No new nearest-bug shortcut.

## Relevant files and boundaries

- app/src/character.rs: player action selection and access to the hand joint;
  reuse the existing cKF rig rather than introducing a GLB pipeline.
- app/src/skeleton.rs: reuse decoder/pose code; change only if a demonstrated
  missing playback primitive is needed.
- app/src/ecology.rs: action request/input ownership and completion handoff.
- app/src/locomotion.rs and input.rs: action movement/focus gating.
- A small app/src/player_tools.rs module may own tool rig/attachment/timeline.
- logic/src/player_tools.rs: source-backed pure action decisions as needed.
- Existing disc_gfx/N64Material and sound queue supply rendering/audio.

First acceptance is an empty swing plus recovery; full capture presentation,
ready/creep states, obstacle reactions, golden tools and rod chains remain named
follow-ups. Preserve current catches while integrating, and document any interim
timing adaptation instead of claiming full retail capture acceptance.

## Acceptance

Philip can equip the normal net, see it stay in the correct hand while turning,
then trigger one complete swing and recovery back to a usable held idle/walk.
Player and net deformation/orientation/scale agree through the motion; no detached
net, looping swing, missing texture or stuck pose. Repeat input cannot start
overlapping actions. Pause resumes consistently; unequip, F9 and scene changes
leave no stale rig/action. Empty swing emits its existing source cue once and
does not award an item. Existing catches remain usable until their separate
source-backed integration. Missing disc keeps the placeholder loop usable.

Record compile/lint/format/build separately from Philip's visual acceptance.
No tests/probes were added or run for this planning pass. No code changed.
Claim through task-claimer.sh before implementation; planning claim released
at handoff. M4 stays open; title/rename remain deferred.
