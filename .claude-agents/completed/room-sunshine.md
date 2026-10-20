# Player-room window sunshine (ef_room_sunshine)

Follow-up to room-light-switch (741b675a). Agent: claude. Draw the player room's
two window sunshine patches from the disc, positioned per the scene actor data,
with alpha following time of day and the light switch. No save schema change;
no-disc fallback stays playable. No tests added or run.

Skills read: shared workflow, rust-workflow, bevy-019, module-port,
gc-texture-pipeline, brp-visual-qa (+protocol), save-persistence, top-level
gc-iso-reader. gamedb not used; direct source reads and gc_iso_reader lookups.

## Source audit

ACGC-PC-Port `src/effect/ef_room_sunshine.c`, `src/data/model/room_lightR.c`,
`src/data/scene/player_room_s.c`, `src/game/m_kankyo.c`, `src/game/m_play.c`
(identical to ac-decomp apart from dt scaling, per the light-switch audit).

- `SCENE_MY_ROOM_S` places `ROOM_SUNSHINE` with ct arg 2 at (40, 0, 120) and
  ct arg 3 at (200, 0, 120): the left and right side walls of the 4x4 room
  (floor x 40..200 at game y 40). ct: arg != 0 -> x -= 1; arg 2 -> L procs,
  x += 5, y = 1 + BgY, x -= 6 -> (38, 41, 120); arg 3 -> R procs, x -= 5,
  y = 1 + BgY, x += 6 -> (200, 41, 120). Draw translates (x, y + 0.1, z),
  scales (scale.x, 1, 1). The port reads BgY from the disc floor list surface
  (`TriangleSink::height_at`, 40.0 fallback); logged positions match.
- Move procs: L scale.x = 1.5 sin_s((sec<<14)/14400) before 04:00 and
  1.5 sin_s(((sec-12h)<<14)/28800) 12:00-20:00; R = 1.5 sin_s(((12h-sec)<<14)
  /28800) 04:00-12:00 and 1.5 sin_s(((24h-sec)<<14)/14400) from 20:00; else 0.
- calc_alpha: 120 -> 0 over 00:00-04:00, 255 * (28800 - |sec - 12h|) / 28800
  04:00-20:00, 0 -> 120 over 20:00-24:00, x0.6 in rain/snow, (u8). `now_sec`
  is int, so the mixed signed/unsigned |sec-12h| branch has no quirk.
  `GETREG(TAKREG, 30)` is a debug register (0).
- Prim colour: `base_light.sun_color_window` 04:00-20:00, else
  `moon_color_window` (BaseLight bytes 28/31, blended per period like the
  other colours); prim alpha `(int)((u8)calc_alpha * mKK_windowlight_alpha_get())`.
  Draw skipped when window alpha < 1e-4 or scale.x == 0.
- `mEnv_MakeWindowLightAlpha` (indoors): target per pinned kankyo_env
  (0 within 120 s of midnight/noon with the s16 cast quirk, light-switch rooms
  `(1 - point_light_percent) * 0.78 + 0.22`), `add_calc(alpha, target,
  1 - sqrtf(0.8), 0.005, 0.005)` each frame from `Global_kankyo_set`, which
  m_play runs after `mEnv_ManagePointLight`. `Global_kankyo_ct` (scene
  construct: room entry, load) sets alpha 0 then eases once with 0.5 steps.
- Lists (foresta.rel): `light_floor01_mode` (G_ZBUFFER|G_SHADE|G_CULL_BACK,
  ZB_CLD_SURF translucent, combiner colour PRIM / alpha TEXEL0*PRIM,
  `light_window` I 4-bit 16x16 via plain LoadBlock, mirror both axes),
  `light_floor{L,R}01_vtx` (floor quad at local y 1), `light_shine01_mode`
  (texture off, colour PRIM / alpha SHADE*PRIM) and `light_shine{L,R}01_vtx`
  (slanted shaft, vertex alpha 0..255). `light_window` bytes are row-major
  N64 nibbles (zero border, 0x9 inside), not GX tiles. No G_SHADING_SMOOTH,
  but emu64 only names that flag for debug output; GX interpolates anyway.

## Implementation

- `logic/src/kankyo_env.rs`: pinned 122867f `kankyo_env.rs` subset
  (window_light_target + constants, source tests not imported) plus
  `window_light_step` (the source easing/reset). Migration decision: partial
  import; periods/weather fade/rainbow left in the pinned module (the port's
  environment.rs already owns its period table).
- `logic/src/room_sunshine.rs`: native kernel (actor ct, scale ramps,
  calc_alpha, colour choice, prim alpha, draw gate).
- `environment.rs` BaseLight gains sun/moon window colours.
- `player_room/lighting.rs`: Lighting owns `window_alpha`; reset on entry
  (after `RoomLight::enter`) and stepped after each 60 Hz light tick using
  the live `RoomLight.percent` (no duplicated level).
- `player_room/sunshine.rs`: spawned once at startup with the disc (top-level
  entities so the shell's room-prim/env pass does not touch them), meshes from
  the disc lists, I4 decoded row-major, blended N64 materials with back-face
  culling; per frame hidden outside the player room or when not drawn,
  otherwise scale.x and prim colour/alpha updated (diff-checked). The shared
  interpreter does not record plain G_SETTILE format/size, so the tile key is
  built from the source tile commands (as music_grid does for its legacy
  texture). Info log when the window alpha settles or the drawn state changes.
- Not covered: weather (`Common_Get(weather)` rain/snow x0.6; the port has
  no weather simulation, passed as clear like sound.rs), other room scenes
  (villager/shop/museum sunshine variants, other player-room sizes), exact
  `sin_s` table rounding (float sine used).

## Verification (Temp only, synthetic save, absolute EXE, explicit disc)

Real save SHA256 before runtime:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Preview %TEMP%/bevy-crossing-room-sunshine (synthetic seed-12345 base from the
light-switch task), `--disc rom/Animal Crossing (USA).iso`. First run showed a
uniform patch: the plain-tile key had no size, fixed as above; rerun PID 4120.
Logged sequence (prim alpha / window alpha / scale), all matching hand
calculation: 15:58 light off on entry -> L drawn 113 / 0.891 / 1.062, R hidden,
colour (219,239,239); Z on -> 27 / 0.220; F6 x5 to 21:01 -> L hidden, R drawn
6 / 0.220 / 1.382 moon (230,230,120); Z off -> 26 / 0.891; F6 x3 to 00:01 ->
window 0, both hidden; F5 -> home_furniture/pockets/pocket_conds/bells/
furniture/identity/town_seed equal to base, no new keys; F9 -> switch default
on, saved outdoor position, patches hidden outside; F6 to 01:02 and re-entry
-> L 19 / 0.220 / 0.595. Screenshots day-off, day-on, night-on, night-off
inspected (zoomed crops for the pane texture): afternoon pale patch with the
mirrored pane frame and shaft from the left wall, nearly gone with the light
on; night moonlight patch from the right wall, nearly invisible with the light
on. No WARN/ERROR in logs.
No-disc %TEMP%/bevy-crossing-room-sunshine-nodisc (PID 13076): placeholder
art, no sunshine spawned, Z + F5 keep the compared numeric state, no warnings.
No tests added or run.
Real save SHA256 after runtime: unchanged (26CD8CBC...ADEB); never loaded.

Final gates (normal settings): workspace check, all-target clippy -D warnings,
fmt check, capture build 47s (+17s after the texture-key fix), default 41.8s,
release 3m05s. Philip visual playtest pending.
