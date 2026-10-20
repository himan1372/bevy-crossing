# Snowman TV and audited lamp draw callbacks

Philip authorizes the next supported TV/lamp group after television-actions.
Audit remaining C callbacks and lamp lighting, implement kon_snowtv US power,
60-tick screen cycle and level0x2B through the existing TV/native switch paths.
Its face/body and07a..d screen symbols require explicit asset naming and
placement preflight. Correct omitted unique opaque draw lists for bla/whi/gre/
hal lamps, retaining their registered profile lists and original stream order.
Their callbacks have no switch-dependent lighting or drawing; no invented lamp
toggle/PointLight. Other special furniture and whole-room lighting remain open.

Reuse previously read Rust/Bevy/disc/audio/save/build/QA skills and this task's
shared workflow. No tests requested. Claim before source edits, Temp synthetic
runtime only with actual-save pre/post hash, sequential check/clippy/fmt and
capture/default/release builds, both roadmaps/comms, release and publish finished
source/docs. Preserve unrelated .gitignore/save/settings/handoff; M4/migration
and Philip visual/audio acceptance remain open.

Source audit: original US ac_kon_snowtv.c has30 texture-table entries (ten each
07b/c/d), frame>>1,60-tick wrap only while powered, off texture07a, level2B and
switch effects16/17. Both face_model and body_model are drawn every frame.
Native Television::Snow adds the exact source cycle/sound; app asset names are
explicit. Placement preflight now requires both lists and every screen, matching
all four render paths. Existing standard item identities/rotations and save DTO
remain intact. No source tests/unsafe ABI imported or migration completion claimed.

Original ac_my_room_draw runs registered lists, skeleton, then unique callback.
bla adds off_model plus offT; whi/gre/hal add offT. Each callback explicitly uses
POLY_OPA_DISP even for T-named lists; the app appends them to opaque lists in that
order. Existing profile models remain. The audited callbacks never inspect a
switch and insert no light. aMR_FurnitureLightSet reads existing global lights.
Room light control/unaudited furniture are separate, not inferred lamp behavior.

Check11.33s, all-target clippy -D warnings11.51s and fmt pass before runtime;
no code changes since. Initial capture failed with metadata-stub/missing-Bevy
and compiler OOM under low memory (about1.5GB free RAM/1.7GB commit remaining).
A nonincremental attempt also OOMed. Proposed no-debug build was interrupted;
Philip freed RAM and explicitly requested original settings. Normal capture
build then passes23.51s, no persistent toolchain/profile/dependency changes.

Temp disc20708: snowman pocket Place starts off and renders both parts; E turns
on, F5/F9 preserves full room/pockets/conditions/wallet state, E off restores
exact initial state. Powered push moves bit512->2 with owner34->18; turn retains
power/identity; pickup clears bit and returns exact item. All four lamps render;
individual Place/pickup exact-state roundtrips pass. Correctly targeted lamp E
leaves switches and all data unchanged (initial untargeted-position equality
was repeated with positive contact distance). All three older TVs Place/power/
pickup regressions pass through the shared asset/preflight path. Combined four
lamps, snow+TV01 and hidden owner storage preserve full state; independent TV
switching, pause/resume and verified house exit pass. Audio settles to0 active
voices after exit, nonfinite0. No audible-retail acceptance inferred. Screenshot
inspection includes snow off/on sequences, four-lamp scene, individual lamps and
old TV regressions; no automated pixel comparison claimed (Pillow unavailable,
no tool installed). Cosmetic gamepad UUID mapping warning is unrelated; no
missing-model/texture warnings or gameplay errors found in owned disc logs.

Fresh disc32316 restores both powered TVs, four lamps, hidden contents and exact
numeric room/pocket state from startup. Snow off/on changes only its bit, retaining
the other TV and all lamp/storage data. No-disc check and final builds follow.
All runtime helpers/saves/screenshots remain disposable Temp data; actual save
pre-runtime SHA25626CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Philip visual/audio acceptance and M4/migration remain open.
No-disc23280 F5 preserves complete room layers/switches/storage/music/decor and pockets/conditions/wallet exactly; placeholder screenshot inspected. All owned previews closed. Actual save post-runtime hash matches pre-runtime baseline; unrelated .gitignore/settings/handoff preserved. Both roadmaps updated with45 done6 doing27 planned across78 items, not a full-game percentage; static metadata/local anchors pass, no new browser-layout claim. Standard/release gates running with original project settings.
Final normal workspace build passes21.18s; release build running. Default capture23.51s used original settings after Philip freed memory. No gameplay source edits after the passing check/clippy/fmt/capture gates; no tests added or run.
Final release workspace build passes2m39s with original project settings. All required check/clippy/fmt/capture23.51s/default21.18s/release gates complete, no tests added/run. No owned preview or Cargo process remains. Actual save unchanged; source/docs ready to publish and claim ready to release. Philip visual/audio acceptance, M4/migration and room lighting stay open.
