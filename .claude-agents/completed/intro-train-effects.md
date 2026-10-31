# Train engineer, smoke/steam and sounds

Philip selected priority 1 on 2026-10-31. Identity: codex.

## Scope and design

Extend the existing station-arrival train with its original engineer, chimney
smoke, cylinder steam and positional sounds. Read all art/audio at runtime from
the disc. Reuse NpcModel, the display-list renderer and Nas audio engine.
Original C is authoritative: ac_train0_move.c_inc, ac_npc_engineer.c,
ef_kisha_kemuri.c, ef_steam.c, m_train_control.c and Na_KishaStatus* in game64.c_inc.
Keep current camera, passenger landing, movement speeds and Nook sequence.
Use existing checkout/main workflow, preserve .gitignore and actual save.

The chosen source-driven implementation avoids a second particle/audio framework
and offline exported art. No new dependencies or save schema. Fixed native ticks
keep emission and sound cadence independent of render frame rate.

## Checklist

- [x] Engineer follows the cab at (-40,47,20), faces +X, leaves with train.
- [x] Disc smoke/steam use native lifetimes, positions, motion and blend frames.
- [x] Native stop/departure signals, speed-dependent chuffs/clacks and steam loop.
- [x] Cleanup on departure, scene exit and QuickLoad; safe missing-disc path.
- [x] Sequential check/clippy/fmt, capture/default/release builds.
- [x] Isolated synthetic-save runtime, screenshots and numeric/lifecycle checks.
- [x] Source review, roadmaps/comms and ownership completion; explicit-file publication follows.

No tests added/run without Philip's request. Runtime evidence belongs in Temp.

## Implementation/review notes

`logic::train_control::Presentation` runs at 60 Hz with the movement controller.
Chimney wait is floor(12/speed) or12; native smoke lifetime80 and steam30.
Steam tracks saved integer wheel frames2..5 and13..16. Smoke uses its two native
textures, prim LOD128 and gray30/alpha200->4. Steam uses pairs(0,1),(2,1),
(2,3),(3,3) and all15 native LOD values, interpolated at age/2.
Billboards use the world camera (eEL_AutoMatrixXlu); private per-puff materials
are removed on death. One-time cache preparation preserves missing-asset safety.

Engineer NPC draw index341 (238+103), +X facing, (-40,47,20) from engine;
the app root compensates CharacterRig's capsule-centre offset. No walking gait.
Original intro initialization returns sound state0 (silent initialization),
then stopped0x73/depart0x71; chuff0x3B and clack0x3F use speed-dependent counts.
The stable ambience source id0x10 remains present while stopped, as in native C.

Fresh source review found two issues, both corrected: house state also owns
scene exit, and wheel cues must compare the saved frame before advancing.
Initial QA found Town-to-Town replay self-transition must not trigger cleanup;
cleanup now tracks whether the train has reached outdoor Town and checks both
scene states. A second startup found run conditions execute before sub-state
resources exist: scene cleanup accepts optional state resources.

Evidence folder: `%TEMP%/bevy-train-effects-20261031`. Baseline real save SHA256:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
## Final verification

Final check2.88s, warnings-denied workspace/all-target clippy6.54s, fmt passed;
capture29.29s, default38.49s and release5m30s passed. No tests added/run.
Isolated final arrivals load smoke and all4 steam variants without error/panic.
Engineer root offset measured(-1,2.175,0.5) world units: native offset plus1
capsule-centre compensation; visible engineer/white steam/black chimney smoke
captures actually inspected (normal framing and wider QA-only FOV).
Original shipped camera remains unchanged.

Initial arrival observations peak at16 smoke/6 steam sprites; they decay to0
while waiting stopped. Complete driven arrival/greeting/departure records
stop0x73 and depart0x71, with departure peak33 smoke/6 steam; all train roots,
engineer and particles absent at27.76s of the wide observation run. Audio
diagnostics across these runs report nonfinite0, limited frames0, minimum gain1.
Source cadence/IDs are checked; audible acceptance requires Philip listening.

QuickLoad while the train exists removes every train root/engineer/puff; the
loaded world continues into its saved position and Nook welcome. Direct house
entry setup was intercepted by that existing welcome before entering, so house
exit cleanup is source-reviewed rather than claimed as a driven door result.
Both HouseState and ShopLocation feed the same cleanup path.

Missing-disc completed synthetic save: bells543, loan12345, house ownership,
house mail, pockets/conditions, loose furniture, complete room and player cloth
preserved afterF5; no train actors spawned. Identity remained played with the
same flags/ID/name; only events_day advanced normally from9799 to9800.
Final actual-save hash matches baseline. Preserve unrelated .gitignore/save.

Acceptance/M4 remain open. Camera movement and dialogue reactions are not part
of this selected slice. Back-car spring remains outside it.
