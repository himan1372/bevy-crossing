# Arrival and Nook-tour camera framing

Philip selected priority2 on2026-10-31. Agent codex; existing checkout/main.

## Design and source

Follow ac_intro_demo_move.c_inc, ac_npc_rcn_guide_talk.c_inc, m_demo.c and
m_camera2.c. The arrival request uses priority9 and zero transition duration:
hold (2180,ground,820), distance620, 45-degree downward view. Porter's0x07DD
force-talk requests NORMAL at demo priority6 after the initial request priority
has reset to0 (ac_npc_station_master_talk.c_inc, m_demo.c). Release then, not
after the subsequent walk. On release, chase
the player from the actual held centre rather than the hidden normal-camera
state. Nook's0x07DE call,0x07E2 acre warning and0x07E3/4 door speech use NORMAL;
introductions0x07DF, explanation0x07E1, returned-home offers0x07E5/6 and0x0820
use TALK. Carry the originating camera mode through message branches/payment.

Implement the native centre/distance/angle chasing and two-person talk target
in a disc-independent logic kernel. Read native NPC eye height at draw-data+0x5E.
Use existing terrain collision data for the source's high-cliff alternative.
Keep one camera writer for these outdoor intro modes, integrating with existing
player camera rather than stacking camera systems. Preserve shop previews,
train-interior camera, house views, widescreen support, gameplay and saves.
No invented arrival pan, dependency, tests or save schema.

Visual QA also exposed the native foreground-actor culling needed by TALK:
Camera2_Talk_SetTalking_Cull waits15ticks, then hides eligible actors south of
the view centre in the actors/eye horizontal corridor. Typed occluder widths
65/90/110/130/142 preserve profile rules, excluding station/train/ground.
Boards, trees, homes/mailboxes/gyroids and public buildings use this path;
restore their prior visibility when leaving the close view or entering a room.
bg_item_move.c_inc / bg_item_common.c_inc apply the same65-unit corridor to
grown trees and NOTICE/MAPBOARD/MELODY; ordinary ground goods/rocks remain.
Shrine's own draw callback uses105. Cached shop visibility changes coordinate
with the culling restore value so reloading another tier cannot revive an old
variant. The reviewer identified that ownership edge; it is corrected.

## Plan/checklist

- [x] Source audit including native angle wrapping and terrain branch.
- [x] Kernel and focused app camera integration, seeded release and cleanup.
- [x] Nook camera selection survives branches and modal payment.
- [x] Sequential check/clippy/fmt, capture/default/release builds.
- [x] Synthetic Temp arrival/tour/home/choices/payment/reload checks and captures.
- [x] Source reviews and roadmaps/comms; completing ownership and explicit publication.

Real save and unrelated.gitignore remain untouched; runtime evidence in Temp.

## Reported smoke regression (current priority)

During camera QA Philip reported whole-game choppiness starting with smoke.
Camera work is retained; fix this regression before further camera verification.
Focused Astra source review found smoke material birth/death invalidates
environment's materials.len cache, whose iter_mut flags every town material as
Modified. Bevy then extracts/clones/rebuilds all their GPU bindings. No particle
leak found. No builds/input/screenshot automation during bounded comparison.

Temp pre-smoke vs smoke arrival after startup: median15.07/15.03ms but p95
17.99/74.42ms, peak20.11/82.03ms. With smoke: p9572.42ms, peak79.01ms,
34/84 sampled frames over20ms. Ordinary walking with train absent: median
7.01/7.02ms, p957.81/10.52ms, peak8.47/11.61ms. This supports material churn
as the whole-scene hitch during arrival/departure, rather than a general leak.

Correction: environment uses Added/LoadedWithDependencies events for new
materials, ignores Modified/Removed events, and only refreshes the whole set
when minute/indoor state changes. Immutable equality checks precede mutable
asset access. Lighting uses the current minute consistently. Re-profile before
claiming the regression fixed. No tests authorized.

## Verification and evidence

Evidence: `%TEMP%/bevy-intro-camera-20261031`, synthetic saves only. All owned
previews closed before final default/release builds. Actual save SHA256 still
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

Final code check2.80s, warnings-denied clippy6.72s, fmt passed; capture9.21s.
Final default47.78s and release4m07s passed. No tests added/run. Earlier successful
capture checks exposed a source-backed visual issue (noticeboard occlusion),
which the final native culling path corrects.

Source review agrees with talk math, original mode selection, terrain branch
and schedule. Porter's0x07DD NORMAL request releases the arrival hold; priority9
is not a numbered camera mode. Native eye-height byte35 confirmed on disc at
npc_draw_data_tbl+258*0x6C+0x5E. Close views measured distances9.8072/10.1495/
9.805 world units (matching290+1.46*actor-distance) and pitch23.7963867deg.
Hold eye observed(-1.5,14.9601545,-32.139843); release has intermediate positions
before settling to player. Camera FOV20deg/near5/far40 are preserved.

Driven without camera component edits: station arrival, Porter release, stairs
to welcome, tour, wide/close mode changes, home1 inspection/refusal, home2
acceptance, repeated explanation, Escape payment cancellation and Give/handover.
Final save: home_slot2, loan17400, quest bag removed, played/first_job true.
F9 leaves the normal camera transform unchanged. A QA door tap needed700ms
to face the panel from the sampled initial yaw; no game movement fix was made.

Final cull run: board0007 Hidden during explanation, prior Visibilities of
board/four houses/four mailboxes all Inherited again when choosing, entering a
room and F9. Images inspected, including unobstructed explanation. Cached shop
tier visibility changes update the underlying cull restore state. Native BG
talk limit applies to trees/boards, not rocks/ordinary ground goods.

Completed synthetic missing-disc save retained identity, bells, loan, ownership,
mail, pockets/conditions, furniture/full room and cloth after F5 (no differences
across compared fields); normal camera remained finite and correct.

Material fix confirmed with no builds/input/screenshot automation during timing:
corrected smoke post-warmup median8.70ms/p959.59ms/peak11.50ms, no samples>20ms.
The earlier material rebuild path's p9572.42ms/peak79.01ms is removed. Native
AssetEvent timing initializes newly constructed materials next frame from their
constructor/copied lighting; room lighting reapplies after base environment and
menu lighting remains exempt. Astra reviewed this path and culling ownership.

Limits: bounded development checks, not Philip acceptance. High-cliff branch
is source-reviewed; sampled tour used the normal low-angle talk branch. Global
camera border/swing behavior and dialogue reactions remain outside this slice.
