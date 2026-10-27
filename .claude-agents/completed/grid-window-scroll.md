# Scrolling window texture on the retail grid windows

Claimed by claude 2026-10-27 (claim made retroactively; I had started before claiming). Philip chose it after grid-leftovers.


## Scrolling window texture (2026-10-27, claude)

Source: `mPC_set_frame_dl` / `mPM_set_frame_dl` set `gDPSetTileSize_Dolphin(RENDERTILE, -texture_pos*4,
-texture_pos*4, 32, 32)` before the window background; `mSM_menu_control_move` advances `texture_pos`
by 0.3535 per tick (wrapping at 1024). In the window model tile 0 is the 32x32 cloth (`pwd_win_nuno`
in `pwd_win_mode`) and each `wNT` piece loads only its shape mask into tile 1, so only the cloth
moves; the `ue`/waku/fuki parts load their own tile 0 and stay put.

App: `shop_nook/password/ui.rs` keeps the window's background materials (`PasswordAssets.frame`,
refreshed by `draw` from the cached model) and `scroll` (chained after `draw`) advances a slide at the
pockets' rate and writes it, over the texture size, to `combiner.extra.yz` (the existing tile-0 UV
offset) of the materials that also carry a tile-1 mask. Both windows (Say code `pwd`, receiver `pas`).

Verified (Temp, disc, Say code window): two screenshots 5 s apart show the silhouette cloth shifted while
the cloud outline and the controls are unchanged. The receiver window uses the same path but was not
driven. No WARN/ERROR. Not done: cursor pulse (`cursol_opacity_step`), pad rumble.
