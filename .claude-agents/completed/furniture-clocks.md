# Disc clock hands, motion and antique ticking

Philip authorizes the clock slice after snow-tv-lamp-draw. Audit ten profiles:
kon_blueclock/redclock/grclock/ameclock/mimiclock/waclock/atqclock/snowclock,
nog_tri_clock01 and sum_mezaclock. Drive their source joint-Z hand offsets from
existing local game time; play existing disc animation at source half speed.
Snow and meza draw additional hand models in post-joint callbacks. Antique
clock uses shared120-tick room beat, events15/45, sound12D and reset at0.
No clock power toggle or configurable alarm is exposed by audited callbacks;
do not invent those interactions. Preserve placements, movement, storage, TV,
diary, save schema and no-disc numeric state. Cosmetic time/phase is transient.

Reuse loaded project Rust/Bevy/animation/disc/audio/save/QA/build skills and the
shared workflow read for this task. No tests requested. Temp synthetic previews
only with fresh actual-save hash. Sequential check/clippy/fmt/capture/default/
release gates, inspect runtime/visual/time-change and lifecycle evidence, update
both roadmaps/comms, release and publish authorized finished source/docs.
Unrelated .gitignore/save/settings/handoff remain untouched. Philip acceptance,
M4/migration, clock alarm UI and room lighting remain separate.

Source reconciliation: original m_time.c derives rad_min from seconds+60*minutes
and rad_hour from that plus3600*hour, quantized to signed binary angles. Native
furniture_clock.rs adds safe hand math, cKF1..end half-speed repeat and the shared
120-tick beat with crossings0/15/45. Pinned Rustimal game_time.rs checked; its
native time stack remains unchanged, no unsafe ABI or source tests imported.
Exact joints: blue/gr/ame3,5; red/nog3,4; mimi3,6; wa2 (hour only); atq5,7;
snow6,7; meza3,4. Snow/meza post-draw short/long lists attach to those joints.
Source generic meza rig constructor/move and other custom constructors use0.5
speed. Cached skinned models preserve palette/streams, segment-D matrix slots,
parent hierarchy, authored local motion and joint-aware bounds. Placement checks
rig/animation/joints and required extra models before inventory consumption.

Antique source atq callback emits positional12D on room tick0/tick1 (15/45),
restarting animation at120/0. Bevy virtual time supplies source60Hz elapsed ticks;
initial room phase follows elapsed time and is transient, not serialized.
Scene origin read-only accessor gates current-house sound. Owned birth/pickup
presentation and retiring roots suppress ticking; pause/update/load integration
keeps existing audio ownership. Source beat/render timing is a Bevy adapter,
not a claimed frame-perfect hardware scheduler. No alarm UI/power action found
in the audited meza/other callbacks; none invented. Hardware DSP/listening,
configurable alarms, other furniture and room lights remain separate.

Final check8.98s, all-target clippy -D warnings24.90s, fmt and capture1m01s pass,
normal project settings. Initial check caught private CurrentRoom origin access;
a read-only accessor resolves it. No other source edits after the passing gates,
no tests added/run. Both roadmaps updated79 items46 done6 doing27 planned;
static IDs/local anchors pass, not a game completion percentage. No fresh
browser-layout claim.

Temp disc14692: all ten hour joints move -30.0037deg on F6+1h, with minute-hand
motion bounded to elapsed seconds. Non-hand authored joints visibly animate;
pause holds every joint transform exactly. All ten numeric layers/pockets/
conditions/wallet/music/decor survive F5/F9. Each variant Place/E(no invented
alarm)/pickup roundtrip preserves exact numeric state and item identity.
Antique push34->18 and turn preserve native identity/switches; pickup removes
rig/ticking. Hidden antique in wardrobe retains numeric item without a visible
clock rig. Surface meza extra hands render and pickup returns exact item while
preserving its box and all hidden contents (full-state offline comparison).
Mixed snow/antique/meza plus powered TV retains all state; pause and verified
house exit retire sound. Audio diagnostics settle to0 active voices with
nonfinite0; no audible-retail acceptance inferred. Initial small-clock images
were obscured by player; all ten clear offset-player close-ups subsequently
captured and actually inspected, plus before/after time and mixed-scene views.
No gameplay/render/missing-asset errors in the owned initial disc log.

Fresh disc20008 restores exact mixed-room state, three clock rigs/21 joints,
powered TV and hidden contents. Clock display follows current local game time;
F6 debug offset is not newly persisted. Fresh screenshot inspected. Expected
unmapped gamepad UUID warning appears, with no source/render/asset failures.
No-disc and final default/release builds follow. Actual save pre/post-runtime
SHA25626CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB unchanged.
All automation uses synthetic Temp saves and absolute EXE. Preserve unrelated
.gitignore/settings/handoff; M4/migration and Philip acceptance remain open.
No-disc20428 F5 preserves complete room/TV switches/hidden-item/music/decor and pocket/condition/wallet data without profiles; placeholder inspected. All owned previews closed. Actual save hash matches baseline; unrelated .gitignore/settings/handoff preserved. Default/release gates now running with original project settings.
Final normal workspace build passes1m02s; release build running. No gameplay source edits after passing check/clippy/fmt/capture1m01s gates. No tests added/run.
Final release workspace build passes3m25s with original project settings. All required check8.98s/clippy24.90s/fmt/capture1m01s/default1m02s/release gates complete, no tests added/run. No owned preview or Cargo process remains. Actual save unchanged; ready to release claim and publish authorized source/docs. Philip visual/audio acceptance, alarm UI, M4/migration and room lighting remain open.
