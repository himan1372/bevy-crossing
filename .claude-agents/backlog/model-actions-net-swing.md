# model-actions-net-swing — first held-tool action slice

2026-10-15, codex. First action slice implemented; Philip playtest pending.

Read Model-research/report.md and index.html before choosing this slice.
Next task: a normal net held on the player, one complete empty swing, stop/recovery,
and return to locomotion. Catch presentation and full tool families follow later.

- [x] Trace held-net setup, player/tool clips and procedural joint adjustment.
- [x] Load the normal net rig/materials at runtime from the user's disc.
- [x] Bind the net to the source hand transform and share the action clock.
- [x] Drive one non-looping player/tool swing and source-backed recovery.
- [x] Gate movement/repeated actions and cancel safely on load/scene/tool changes.
- [x] Retain usable no-disc action behavior.
- [x] Compile/lint/format/build; no tests unless Philip requests them.
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

Before this slice, character.rs PLAYER_ANIMATIONS had wait1/walk1/run1 only;
animate_characters selected only locomotion. ecology.rs tool_interact resolved
the net on E immediately with a distance/facing/nearest candidate test and no
held net model.
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
## Implementation and evidence

app/src/player_tools.rs loads cKF_bs_r_tol_net_1 and its normal held, swing
and swing-wait clips from the disc. The root attaches to player hand joint20;
existing display-list decoding and segmentD joint skinning draw the net.
The runtime BOY_part_data NET mask combines the net1 held-arm pose with
locomotion. No disc-derived asset files or mask bytes are embedded/committed.

The player swing is frames1-10 at 0.5 frames per 60-Hz tick. Recovery freezes
the player at the final swing frame while the tool's swing_wait1 frames1-21
finish. Procedural short-angle chase follows m_player_item_net and m_lib;
an independent transform after dummy joint3 bends descendants4/5, outside
clip morphing. Movement and facing lock during the action; repeated E does
not restart it. Unequip, focus/room/scene changes and successful F9 cancel
pending action/contact. Load precedes tool processing. Pause uses the existing
virtual clock. Missing-disc action timing uses the source lengths, with the
existing placeholder player and catches.

Interim adaptations: movement stops immediately instead of retail braking;
five-frame pose morphs use quaternion interpolation instead of short-Euler
interpolation. Existing distance/facing catch selection is preserved, delayed
until after source player frame6; this is NOT the full retail endpoint/capsule
capture chain. The source swing effect near frame9, obstacle/pull reactions,
capture presentation, ready/creep states and golden net remain follow-ups.

Disposable Temp town review with Philip's disc: inspected held idle, windup,
downstroke, recovery and return-to-ready screenshots; textures and original
mesh are present and the net stays attached. A tree partly obscures the
downstroke/recovery views, so these are not retail motion comparison sign-off.
Empty swing leaves pockets at0/15. Movement position remains fixed during
action and changes after recovery. Logs show one swing/recovery/ready cycle
for repeated E, pause extends the timeline, unequip hides the net and cancels,
and F9 cancels and restores the disposable saved position. Room/scene cleanup
and successful bug catches were reviewed in code, not exercised in this pass.
Screenshots/logs remain in Temp, outside the repository.

Explicit missing-disc preview starts the placeholder town, equips the net,
completes swing/recovery/ready, then permits movement; screenshot inspected.
Only the existing placeholder player is drawn, as intended without disc art.

Workspace check, all-target clippy with warnings denied, formatting, capture
build and final default build pass. No tests added or run. Real save hash
unchanged; settings untouched. Philip's visual/action acceptance remains open.
Claim released at handoff; task stays in backlog for acceptance. M4 stays open;
title/rename remain deferred.
