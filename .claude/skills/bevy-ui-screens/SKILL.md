---
name: bevy-ui-screens
description: "Implement or fix game menus, dialogue and HUD screens while preserving their existing renderer, modal input ownership and source behavior."
---

# Game UI screens

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

## Match the existing screen

Retail pockets, dialogue and music menus use disc meshes/materials/font metrics on UI cameras/render layers. Native Bevy `Node`/`Text` UI also exists. Extend the renderer the target screen already uses; do not convert disc menus to flexbox, BSN or Feathers as a prerequisite.

Start with the relevant paths: `app/src/pockets.rs`, `app/src/pockets_disc.rs`, `app/src/msg_window.rs`, `app/src/dialogue.rs`, or `app/src/player_room/music_grid.rs`. Check original C for cursor movement, wrapping, accept/cancel timing, empty states and refusal messages. Resolve Bevy API details against the manifest and installed matching source.

## Preserve ownership and transactions

Follow `app/src/input.rs` and `app/src/states.rs`: modal focus, action consumption and interaction ordering prevent gameplay keys from leaking through menus. Verify accept/cancel, release and reopening; scene changes and QuickLoad must clear stale focus and transient state. Do not prohibit keyboard navigation or introduce a new input/event framework to fix one screen.

Keep selection tied to stable item/slot IDs rather than labels. Reuse existing requests and transactional gameplay helpers. Displaying a choice must not consume an item; refusal/cancel must preserve numeric state.

## Check presentation and cleanup

Use native font advances, source geometry, masks and cursor anchors for disc UI. Account for asset readiness before building caches. For native UI, inspect computed bounds and target camera when diagnosing layout. Preserve render layers, mesh/material lifecycle and no-disc fallback.

Use [visual QA](../brp-visual-qa/SKILL.md) for the affected screen: compare appearance plus numeric result and input ownership, including close/reopen and relevant reload behavior. Report layout checks separately from source fidelity and user acceptance.
