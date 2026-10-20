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
