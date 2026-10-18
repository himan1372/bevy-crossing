# Player home entrance movement and exit

Philip requests walking on the brown wooden entrance area inside the player
home, a much smaller exit interaction limited to that area, and facing the door
to exit. Implement this first, then continue the separately authorized drawer
animations/obstruction slice. One task claim at a time.

- [x] Audit rendered entrance geometry, movement bounds and exit callers.
- [x] Add bounded walkable entrance and narrow facing-required exit.
- [x] Appropriate check/lint/format/build and disposable GUI checks. No tests
  added/run without Philip request; never load the actual save for automation.
- [x] Record evidence, release claim, continue drawer task. No codex-handoff edit.

Preserve saves/settings/.gitignore, villager collision and playable no-disc room.
M4/migration remain open; previous "works" follows the build resolution and does
not certify every remaining furniture/source fidelity detail.

Implementation, 2026-10-17: inspected original rom_myhome1_wall_model: first six
vertices use rom_myhome_enter2_tex. Runtime disc numeric vertices transformed
with existing ROOM_SCALE/ROOM_CENTRE give x[-1,1], y0, z[2,4], relative to room
origin. The old square movement bound ended at1.6. RoomBounds now permits the
main square plus the landing corridor, inset by player radius0.4; floor/side
edges stay blocked. Player-room furniture push/pull shares the same floor helper
with its existing0.398 contact tolerance. Furniture placement cells remain on
the original carpet. Exit requires feet on the inset wooden landing, normalized
south-facing cosine>=0.9 and a ray hitting the finite door panel at landing end.
Player-home outside entry and villager-room bounds/doors/collision are unchanged.
No-disc home uses a small central door area on its existing wooden floor.

Evidence: workspace check, all-target clippy -D warnings -j1, format and capture
build pass. No tests added/run. Disposable GUI in Temp/home-entrance-review:
walked from carpet onto wood; stopped at x2999.40015/z3003.59985 after walking
south and trying left, without exiting. E while facing side/away on wood refused;
E facing south on carpet z3001.5 refused; E facing south at z3002.6 exited once.
Confirmed-entry repeat: outside corridor (x~3000.85425) south movement stopped
at z3001.59937; northward return from wood z3003 reached carpet z2999.38013.
An earlier repeat attempted before confirmed re-entry was excluded/repeated.
Explicit missing-disc GUI entered placeholder home, E facing inward refused,
then south-facing E in its small door area exited. Structural Python JSON
comparison confirmed original home_furniture and pockets unchanged after save
(an initial string comparison differed only in object key order).
Own previews13296/11260/11120 closed. Synthetic saves/logs/screenshots stay Temp.
Actual save hash remains DD89745CCFEDDEAEA619A1C931FEE5233EC533C1860E3B5A910A00B8121E5721.
Settings/.gitignore preserved; no handoff edit. Final default workspace and
release builds pass after the drawer slice; Philip acceptance remains separate.
