# Arrival and Nook camera implementation plan

**Goal:** Match native outdoor intro camera holds, releases and talk framing.
**Design:** .claude-agents/completed/intro-camera-framing.md.
**Architecture:** A native math kernel supplies View targets and60-Hz chase.
Existing player camera becomes the single writer for outdoor intro camera
targets; the train-interior and shop-preview camera paths stay separate.
**Constraints:** Existing Bevy0.19.1, no tests/dependencies/save schema; runtime
disc eye-height data; preserve actual save/.gitignore; current main workflow.

- [x] Add logic/src/intro_camera.rs and export it; use native add_calc and
  short-angle stepping. Include normal, talk and cliff-safe talk targets.
- [x] Expose arrival held centre and Nook's originating camera selection;
  integrate app/src/player.rs with a focused intro camera adapter and remove
  arrival's competing writer. Seed return chase from the held camera.
- [x] Implement terrain centre-height/cliff checks against GroundMap.
- [x] Sequential check/clippy/fmt, capture/default/release; no tests.
- [x] Drive isolated arrival/release, welcome, tour, home offers/refusal,
  payment/cancel, reload and missing-disc fallback. Inspect images and numeric
  camera progression; verify lens and previous preview behavior remain intact.
- [x] Review source, resolve findings and update records; publish verified slice.

## Added from source and runtime findings

Native15-tick foreground culling clears the noticeboard/trees/buildings from
the close view and restores prior visibility on mode/room/reload transitions.
Typed profile widths and coordinated cached-shop visibility avoid stale
restoration on tier changes. Camera poses interpolate between native ticks.

Philip's smoke-regression report took priority during execution. Astra traced
per-puff material-count changes to full-town lighting/material GPU rebuilds.
Added/LoadedWithDependencies events now update only new materials; full refresh
occurs on minute/scene changes. Bounded smoke peak79.01ms became11.50ms after
startup; no samples>20ms in the corrected comparison. Details and acceptance
limits are in the completed task. Final release4m07s passed; no tests.

Review focus: no camera discontinuity on release; preserve message camera mode
through branches/payment; handle missing NPC/assets; house/reload/warp reset;
short-angle wrapping and frame-rate-independent minimum steps.
