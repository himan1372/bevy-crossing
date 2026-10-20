# Retail idle clock (m_banti)

Philip 2026-10-20: "the clock with the date never came up when i stood still".
Correction to retail-hud-ui: retail has this window (src/game/m_banti.c).

Source: banti_move/banti_calc_disp_alpha_rate/banti_chk_disp_left/
banti_time_check/banti_draw; Player_actor_Set_address_able_display table
(m_player_common.c_inc). Outdoors (FG field) only; fades in (add_calc 1-sqrt0.8,
0.0425, 0.0005) when the player's addressable type is TRUE (WAIT and some
tool actions) and no submenu; type changes commit after 30 frames (to TRUE),
50 (from TRUE, or to READY_NET), talking at once; fast fade-out on demo or
disabled; slides -184 when the player's screen pos is x>=180,y>=150.
Draws clk_win_mode/youbiT/maru2T (odd seconds)/ampmT/maruT and cKF digit
flips (clk_hiniti date, clk_jikan time; STOP 1..15 at 0.5, seg 8/9 textures,
joint 2 hidden when stopped) in font ortho at scale 16.

## Result (claude, 2026-10-20)

- logic/src/banti.rs: Banti (banti_move, chk_disp_left, calc_alpha,
  time_check, chk_anime_start, cKF stop flips), Addressable, BantiTime.
- app/src/idle_clock.rs: assets from the disc (clk_win_mode/youbiT/maruT/
  maru2T/ampmT, cKF clk_hiniti/clk_jikan + textures by segment), overlay
  layer 1 at x16 units; addressable from InputFocus (dialogue/shop/quest/
  mail = talking; pockets/pause = submenu), NetAction busy (swing = TRUE),
  locomotion Wait (net held = READY_NET, rod = movement), else movement;
  player screen position from the world camera; town only.
- Same change set: shop raffle room by date (logic shop::tanuki_shop_status,
  Now.year added), wall clock skipped on raffle day, font UV half-texel inset.
Verified (Temp preview, real save unchanged): window shows 10.20 Tue 4:56 am
after standing, still up at walk start (50-frame hold), gone mid-walk; shop
walls without bunting on 10/20 with the clock; pockets name marks gone.
Not verified live: digit flip at a minute change, left slide, raffle day.
Gates: clippy -D warnings, fmt, capture, default, release 2m48s; no tests.
