# Shop wall clock (ac_house_clock)

Split from retail-hud-ui (2026-10-19, claude). Retail shows the time indoors
through ac_house_clock, a wall clock keyed by interior scene
(aHC_position_data: SCENE_SHOP0 (200,40,40) obj_clock_shop1 joints 5/6,
CONVENI/SUPER/DEPART shop2..4, post office, police box, museum, tailor).
Cycle: cKF repeat at 0.5, restarted on the shared 120-frame beat (phase =
game_frame % 120 at ct), NA_SE_12D at frames 15/45 (same as the antique
clock kernel); hands drawn at the joint origins with Rz(0x4000 - rad), no
yaw or offset; hidden during the FUKUBIKI raffle for shops 0..4.

Blocker found: obj_clock_shop1 lists use legacy N64 texture commands (plain
G_SETTILE/G_SETTILESIZE with G_LOADBLOCK/G_LOADTLUT) whose tile format and
size the decoder only fills for Dolphin tiles (keys decode as I4 0x0, which
crashed wgpu with a zero-sized texture), and their TLUT is in segment 7
(0x0703C000), which the shop scene does not map. Needs: plain settile
fmt/siz/shift + settilesize width/height in logic graphics, the legacy
row-major decode path, and the segment-7 palette source. A complete module
draft (spawn at SHOP_ORIGIN - SHOP_CENTRE + pos/40, beat/frame/hands update)
was kept at %TEMP%/bevy-crossing-shop-clock-wip/house_clock.rs.
Planned, not started.

## Result (claude, 2026-10-19)

The "segment-7 palette" was a misread: 0x0703C000 is the w1 of an N64
gsDPLoadTLUTCmd (tile 7, 16 entries), whose palette is the current image
(obj_shop1_clock_pal). Ported from ac-decomp emu64 (dl_G_SETTILE,
dl_G_LOADTLUT N64 branch, tmem_map / setup_texture_tile wd==0 branch,
get_dol_wd_ht, wrap rules):
- logic interpreter: plain G_SETTILE reads its tile from w1 (it used w0,
  filing every plain settile under tile 0) and keeps fmt/siz/line/tmem/
  palette/cm/mask; G_LOADBLOCK records tmem_map; settilesize keeps sizes;
  setup_n64_tile derives address, size, format and wraps; Dolphin loadtlut
  is type 2, else the N64 form (RGBA5551, name = tmem/16, 16 per name).
- disc_gfx: TexKey.n64 / tlut_rgba5551; decode_n64 reads row-major texels
  (odd-row word swap when dxt is 0); zero-sized tiles are untextured.
- furniture/house_clock.rs: obj_clock_shop1 at SHOP_ORIGIN - centre +
  (200,40,40)/40, beat/frame/tick and hands; spawned with the real shop.
- ShopHudText joins the dev overlay.
Verified in Temp previews (real save unchanged): clock decodes 64x64 CI4
(3598 opaque texels), shows 10:18 PM correctly; town grid, house and shop
otherwise identical. Tick sound not audibly checked (same kernel as the
antique clock). Gates: clippy, fmt, capture, default, release 3m03s.
