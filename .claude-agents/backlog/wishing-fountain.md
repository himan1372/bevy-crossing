# Wishing fountain restoration

Status: visual restoration implemented; Philip acceptance pending.

Philip reports the wishing fountain missing. WISHING_WELL (5825) was present in the retail layout but absent from the five-entry BuildingSpec renderer.

- [x] Added source actor to existing public-building path: WISHING_WELL 5825, obj_{s,w}_shrine, structure palette index63 (aSTR_PAL_SHRINE), actor shift +20,-19 game units (ac_shrine.c::aSHR_actor_ct).
- [x] Original trunk/figure/base/statue meshes, seasonal leaf geometry and source leaf texture table. Segment9 cedar palette uses mFM_obj_tree_01_pal_dol / TREE_PAL_IDX (m_field_make.c::mFM_SetFGPal); segmentA uses summer/winter/cherry leaf texture as source. These remain runtime disc reads.
- [x] Water/sprash/bubble translucent layers with source first-tile scrolling rates, 60-Hz texture_frame wrapping at8192. Combiner extra.yz now supplies UV offset, default zero for existing materials. GameplaySet pauses animation. Bubble mesh drawn once to avoid duplicate opposing draw submissions.
- [x] Solid footprint measured from base only, excluding tree overhang; normal Town collision/placement logic reused. No-disc placeholder path unchanged.
- [x] Workspace check/clippy all-targets/fmt/capture/default build passed; no automated tests/probes added/run.
- [x] Disposable runtime visual review: seed12345 fountain at (32,0,23.025); original body/statue/water visible in summer and snow variants in winter. Actual HUD Oct16/Dec15 14:44-14:45 (debug+18h; requested negative offset normalizes modulo24). Approaching from z26.525 stopped around z25.40 in both seasons. Summer wider paused view confirmed foliage binding; pause menu obscures central part, so separate ordinary framing used for basin/statue. Preview logs no shader/renderer errors. Images/logs stay in Temp/bevy-crossing-mailbox-review/fountain-*.
- [ ] Philip playtest acceptance.

Source: ac_shrine.c, ac_shrine_draw.c_inc, ac_shrine_move.c_inc, obj_s_shrine.c, ac_structure.h, m_field_make.c. Installed Bevy0.19.1 remains API reference.

## Limits / follow-ups

This restores the visible fountain, not town-rating dialogue, presents/event management or source shadow drawing. Existing shared combiner still approximates TEXEL1 using TEXEL0; fountain first-tile UV scrolls are implemented, exact two-tile water blending/independent second-tile scrolls remain follow-up. Cherry/Christmas dates are source-selected but not separately visually checked. Full retail render-look/M4 remain open.

First preview was launched before linking finished and locked the old executable. Closed only that known Temp preview process, rebuilt capture successfully, then reviewed the correct summer/winter fountain; discarded initial attempt as evidence.

All final previews closed. Default game rebuilt. No save inputs or user settings edits; real save SHA256 unchanged:
0CD03EE33FE4FCFAAFF893271D79A0C6C5885C34B5C9A93D1C375350AA4D49A0.
No disc assets/screenshots committed; unrelated .gitignore/savegame.json preserved.
