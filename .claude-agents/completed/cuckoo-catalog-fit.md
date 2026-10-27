# Cuckoo clock catalog preview fit

2026-10-26 claude, Philip: "cuckoo". Follow-up to the audit in nook-tickets-catalog.md
(clock fitting audit): the cabin cuckoo (`sum_log_hatoclk`) renders small and low in the
catalog circle because its fitted Aabb is about three times the visible body (y -3533..19528).

- [x] Dump the rig's per-joint part extents and find the tall invisible geometry.
- [x] Fix the fit (per-part visibility or excluding the hidden parts), disc-independent logic where possible.
- [x] Temp QA with a screenshot of the catalog; other clocks unchanged.
- [x] Gates, docs/comms, release claim, commit/push.

## Implemented (2026-10-26, claude)
Cause found by dumping the body part's vertices: the cabin clock's 27-vertex body list (joint 1) has three triangles sharing a tip at model x 19528 (a 3-sided spire from a base at x 613), beside the 6-sided pine-cone weights that end at x -3533. The spire shows nothing in this renderer (not visible in the catalog or room), but its mesh bounds made the fit y -3533..19528, three times the visible 11447. Why it draws nothing was not established (guess: clamped transparent texels); the original catalog has no bounds fit at all, it uses per-item `furniture_draw_data` scale/pos_y.
Fix: `furniture::FitBounds` (min/max in the piece's root space) inserted by `furniture/cuckoo.rs` with the visible box (x +-1777, y -3533..7914, z +-1615); `catalog_screen::fit_preview` uses it when present. Other pieces unchanged.
Verified (Temp, disc): the cabin clock now fills the circle (was small and low). Other clocks not re-scanned (no code path change for them).
Not done: the exact original `furniture_draw_data` scale/pos_y table (m_catalog_ovl_data.c_inc) would replace bounds fitting for every piece; the spire's rendering cause.
