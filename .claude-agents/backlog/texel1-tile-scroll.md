# TEXEL1 second tile and two-tile scroll

Queued by Philip (2026-10-19) to follow retail-hud-ui. The renderer does not
model TEXEL1 (second tile; uses TEXEL0, see app/src/n64_material.rs) or
segment-8 two_tex_scroll_dolphin tile offsets, so two-cycle combiners that
read the second tile (on the RDP, cycle 2's TEXEL0 reads tile 1) lose their
moving layer. Known affected: the sprinkler water (completed/sprinkler.md)
and lawnmower grass (completed/lawnmower.md); audit other two-tile lists.
Port emu64's tile 1 handling and per-tile scroll, then wire the sprinkler
(-counter*10 on tile 1) and lawnmower scrolls. Planned, not started.
