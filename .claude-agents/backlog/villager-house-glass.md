# villager-house-glass — every villager home style

2026-10-15, codex. Implemented and visually reviewed; Philip acceptance pending.
Philip's scope: ALL regular villager homes, summer and winter, not just the three
pictured homes. Roof colors are explicitly not part of this correction.

- [x] Read Screenshot 2026-10-15 154957, 155151 and 155706 (left port, right retail).
- [x] Source-check all ten obj_[sw]_house1..5_light_model lists.
- [x] Keep opaque glass despite zero combiner alpha; retain existing true cutouts.
- [x] Inspect all five styles in both seasonal variants, day and night.
- [x] Workspace check/clippy/fmt and capture build; no automated tests added/run.
- [x] Final normal build: cargo build --workspace passed.
- [ ] Philip: visual acceptance in his normal town.

## Cause and correction

All ten normal house light lists use G_RM_AA_ZB_OPA_SURF2 with constant-zero
combiner alpha. emu64::alpha_compare uses GX_ALWAYS for that render mode, and
the opaque blend mode retains the panel. Our shader discarded every non-blended
pixel with alpha below 0.5, deleting these backing panels. The cutout wall and
window-frame geometry then exposed terrain/other scenery through the house.

build_windows now marks its opaque glass materials with FLAG_OPAQUE.
N64Material routes those panels to AlphaMode::Opaque; n64_combiner.wgsl keeps
their pixels regardless of combiner alpha. Window frame/roof/body cutouts and
the separate blended ground-glow pass retain their existing policies.
This applies through the shared builder to every normal home shape and both
seasonal models, with existing geometry, pose, palette and light color.

An initial broader render-mode change also made inherited tree cutouts opaque.
Live screenshots caught it; that change was removed. The final patch explicitly
marks the shared window-panel pass and does not change general actor alpha
classification or the logic interpreter. Broader inherited actor render-mode
setup and exact per-actor alpha thresholds remain separate rendering work.

Files: app/src/disc_town.rs, n64_material.rs, n64_combiner.wgsl.
No new assets, roof recoloring, save/settings changes or gameplay changes.

## Source

Under C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port:
- src/data/model/obj_s_house1.c contains all five summer and five winter glass
  lists; each uses zero alpha and opaque SURF2.
- src/actor/ac_house_draw.c_inc: glass primitive tint and separate window glow.
- src/static/libforest/emu64/emu64.c: alpha_compare and blend_mode.
- src/include/PR/gbi.h: opaque vs texture-edge render-mode flags.

## Visual evidence

Used a disposable synthetic save in Windows Temp/bevy-crossing-house-review
with resident IDs [0,8,5,2,1,3], selected from runtime npc_house_list metadata
to include all five home styles. Original save was never edited. The game
loads the user's disc read-only via an absolute --disc path. No save inputs sent.

Inspected all five summer styles at actual HUD 16:07–16:08 and 19:08.
Inspected all five winter styles at Dec 14, 16:08–16:09 and 19:09, using the
existing +60-day debug offset. Each has black, opaque unlit backing and solid
yellow lit backing; roof/window-frame and tree cutouts remain visible correctly.
Changed viewpoints between houses; stale framing captures were replaced before
review. The first temporary JSON had a BOM and was not loaded; rewrote only that
temporary file as ASCII and confirmed all five house names before coverage.

Twenty final screenshots remain outside the repo:
Temp/bevy-crossing-house-review/obj_{s,w}_house{1..5}-{day,night}.png.
These are agent visual evidence, not Philip's playtest acceptance or a complete
retail lighting/seasonal-scene comparison. No screenshots/disc assets committed.

All previews closed cleanly. Real save SHA256 before review:
0CD03EE33FE4FCFAAFF893271D79A0C6C5885C34B5C9A93D1C375350AA4D49A0.
Final post-shutdown hash matches exactly; default build, fmt and diff whitespace pass. Preserve unrelated
.gitignore and savegame.json; do not stage either.

## Philip acceptance

Restart the rebuilt game. The gable/dormer/upper windows on every regular house
style should be solid dark panes in daylight instead of showing ground/trees,
and should still illuminate at night. Check side windows while walking around.
Tree outlines and existing window-frame gaps should retain their proper cutouts.
Roof color matching and full render-look/M4 completion remain outside this task.
