# TEXEL1 second tile and two-tile scroll

Queued by Philip (2026-10-19) to follow retail-hud-ui. The renderer does not
model TEXEL1 (second tile; uses TEXEL0, see app/src/n64_material.rs) or
segment-8 two_tex_scroll_dolphin tile offsets, so two-cycle combiners that
read the second tile (on the RDP, cycle 2's TEXEL0 reads tile 1) lose their
moving layer. Known affected: the sprinkler water (completed/sprinkler.md)
and lawnmower grass (completed/lawnmower.md); audit other two-tile lists.
Port emu64's tile 1 handling and per-tile scroll, then wire the sprinkler
(-counter*10 on tile 1) and lawnmower scrolls. Planned, not started.

## Result (claude, 2026-10-19)

Source: ac-decomp emu64 combine_auto (cycle-1 stages GX_TEXMAP0, cycle-2
stages GX_TEXMAP1), dl_G_SETTILESIZE (Dolphin sl/tl in 1/16 texel),
m_rcp two_tex_scroll_dolphin (x << 1), fFTR_GetTwoTileGfx, fNS_dw, fNLM_dw.

- disc_gfx PartKey.tex1: tile 1 key for two-cycle parts with a resolved,
  non-empty tile 1.
- N64Material.texture1 (bindings 4/5), CombinerUniform.tile1 (UV scale to
  tile 1, origin shift), FLAG_TEX1; shader TEXEL1 / TEXEL1_ALPHA read tile 1,
  and cycle 2 swaps TEXEL0 to tile 1 when bound. attach_texture1 at the shared
  spawn paths (interior sinks, rigs, ground, town models);
  set_tile1_scroll applies gDPSetTileSize_Dolphin origins.
- Sprinkler: tile 1 -counter*10 (t). Lawnmower: tile 1 -10*frame (t).
- Town fountain overlays keep their tile-0 scroll; tile 1 bound unscrolled.

Verified in a Temp preview (real save unchanged): sprinkler and mower
lists bind their 16x16 I4 tile pairs; spray renders as moving streaks
(frames differ); river water still shows both layers. Gates: clippy
-D warnings, fmt, capture, default, release 2m52s; no tests.
