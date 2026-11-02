# Retail HUD Implementation Plan

**Goal:** Finish the existing native HUD's visibility and coordinate adapters.
**Architecture:** Retain `logic/src/banti.rs` and disc clock/pocket art; update app integration only.
**Tech Stack:** Existing Bevy 0.19/Rust workspace.
**Spec:** `docs/superpowers/specs/2026-11-01-retail-hud-design.md`.

## Constraints and review focus

No tests without Philip's request; no dependencies or extracted assets. Preserve real save/settings and unrelated handoff/.gitignore edits. Use root task claims and synthetic Temp previews. Review intro suppression, all modal focuses, paused presentation, widescreen coordinates and unchanged pocket Bells.

- [x] Audit native `m_banti` and existing completed HUD/idle-clock notes; pocket Bells already matches `mIV_set_money`.
- [x] Update `app/src/idle_clock.rs`: real-time HUD ticks, intro/demo inputs, complete submenu handling and centered fixed-height screen coordinates. Correct `app/src/dev_overlay.rs` documentation.
- [x] Added scope: restore native collection pages, palettes, scrolling, catch bits and R/page controls; integrate atomic wallet transfers with native42-tick counter animation. Verify full/cap/quest refusals and save/reload independently from visuals.
- [x] Run check, warnings-denied clippy, fmt, capture build sequentially. Inspect isolated runtime screenshots/state for the scenarios above, including date/minute rollover and F9. Both-side crossing remains source-reviewed only.
- [x] Source review; build default/release. Update task/comms/roadmaps and complete ownership. Publish explicit files after final status inspection.
