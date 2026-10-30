# Station arrival visual recheck

Philip selected priority 1 on 2026-10-30: recheck train alignment, stepping off,
Porter and the walk out in an isolated Temp capture. No new intro features.

Read the repository skills, shared workflow, BRP protocol and latest handoff.
Use a synthetic save and the arrival-only development trigger; preserve the real
save and unrelated .gitignore changes. Record observed evidence and limits.

## Reproduction and cause

Current source revision: `54d9aead`. Capture build passed. First isolated replay:
`%TEMP%/bevy-arrival-recheck-20261030`. Train runs along the rails with its door
facing the platform; the original outtrain1 animation plays; Porter greets the
player and the scene completes. Philip reported that no walking animation plays.

`intro_arrival::scene` moved the player Transform in Stage::Walk without updating
Locomotion. `character::animate_characters` selects its motion from that state,
which stayed Wait because cutscene input is locked. Original
`ac_intro_demo_move.c_inc::aID_walk_one_unit_init` requests demo walking to
(2220, 840) at speed 2.5, followed by wait in `aID_go_out_of_station_init`.

## Fix

The arrival resets locomotion while it owns the player, selects Walk with the
scripted world speed normalized to the existing locomotion units while moving,
and returns to Wait at the destination. Its scene system now runs after ordinary
movement and before character animation. No new animation assets or framework.

## Follow-up feedback

Philip then reported a jump after hopping off and that the walk was much too fast.
The first walking fix was captured at `%TEMP%/bevy-arrival-walk-fix-20261030`:
13 descendant joint rotations changed between the two moving samples; the final
position was (-0.5, 5, -42.6). The initial manual south movement hit the platform
boundary, so it did not establish returned movement control.

Further source tracing found that disc_town::follow_ground overwrote player Y
during the train/root-motion phases. It now leaves the player to the arrival in
those phases, and the get-off base height settles onto the platform using the
actual clip's normalized progress. The clip-stop path previously skipped
play_clip when name became None, discarding the requested exit blend; it now
passes through the existing cache reset and locomotion blending.

The walk now advances at 30 movement ticks per second instead of 60, reducing
world speed from 3.75 to 1.875 units/s; gait playback uses the same speed.
Exact disc inspection: outtrain1 has 29 frames, root (0,1000,0) initially and
(-300,1000,5200) finally. Its first root matches the normal base, so no extra
horizontal placement or root-offset correction was invented.

Status: final feedback changes verified. No tests added or run, following
the standing project rule. Remaining retail camera easing and Porter idle are
outside this repair, and Philip's visual acceptance remains open.

The failed manual-movement check also exposed a real input lock: Porter's
begin_special completion leaves DialogueState.external_result pending. The
arrival now consumes its result as it advances from Talk; this releases the
existing modal input gate after the cutscene.

## Final capture evidence

`%TEMP%/bevy-arrival-walk-smooth-20261030`: inspected getoff-contact.png and
walk-contact.png, plus exact parent transforms and descendant joint rotations.
The get-off samples settle continuously to (-1.6553, 5, -43.3), with the animated
vertical arc intact. Six moving walk samples reach (-0.5, 5, -42.6), with 12-13
joint rotations changing between samples. A subsequent KeyA moves the player to
(-1.92224, 5, -42.00008); F5 produces a synthetic save with intro_done=true and
first_job=true. No ERROR or arrival warnings. One unrelated missing-model warning
for iam_sugi_kpool (0x3258) appears at startup. Audio diagnostics remain finite.
The real save SHA256 stays C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

Final code passes check, clippy, fmt, capture, default and release builds.
One scoped review found no blocking issues. No retail comparison
or Philip acceptance is implied; camera mode 9 easing and Porter idle stay open.
