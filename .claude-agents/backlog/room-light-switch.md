# Player-room light switch and switch-driven room lighting

Philip authorizes the room-light audit proposed in docs/codex-handoff.md ("le go"),
supplying room-light research and asking that ac-decomp be the source. Agent:
claude. Skills read: shared workflow, task-coordination, rust-workflow, bevy-019,
module-port, gc-texture-pipeline, brp-visual-qa (+protocol), save-persistence,
gc-audio-pipeline, bevy-debug, bevy-ui-screens, windows-build, and both top-level
skills/ (gc-iso-reader, gamedb-cli). gamedb not used; direct rg/source reads.

## Source audit

Upstream ac-decomp master `src/game/m_kankyo.c`, `m_lights.c`, `m_room_type.c`,
`include/m_kankyo.h`, `src/actor/ac_my_room_draw.c_inc`, `src/game/m_actor.c`
were downloaded to the session scratchpad and diffed against ACGC-PC-Port with
CRLF ignored: lighting logic is identical; the port only adds dt scaling
(flicker loop, add_calc) and a PC fog tweak. Retail frame = one 60 Hz tick.

The pasted research is broadly right; verified details and corrections:

- Switch bits are RAM (`l_mRmTp_switch_data`), not saved. Game start/load
  (`mRmTp_SetDefaultLightSwitchData(1/2)`) sets an owned house's main-floor bit
  from 18:00 to 05:00 (`mRmTp_CheckMyRoomLightON`). Basement bits start off.
- `mEnv_ManagePointLight`: on -> 20-frame table flicker from point_light_min
  0.14, then add_calc(1, .01, .01, .01); off -> add_calc(0.14, .2, .2, .02).
  Z (`chkTrigger(BUTTON_Z) && mRmTp_PleaseDrawLightSwitch()`) is read after the
  level update; ignored while the switch-on flicker runs. SE NA_SE_LIGHT_ON 0x16
  / NA_SE_LIGHT_OFF 0x17 via sAdo_SysTrgStart. Z allowed in the four player
  houses (main/basement) and island cottage; NPC/lighthouse/tent are not Z rooms.
- Entering (`mEnv_RoomTypePointLightSet`) sets the level directly: 1 or 0.14.
- SCENE_MY_ROOM_S point light (120,180,180), colour 220,220,200, radius 1000.
  Its colour is scaled by the level every frame.
- `mEnv_ChangeDiffuseLight`: in switch rooms sun/moon colours x
  `(0.4 - 1) * level + 1`. Ambient is the normal time-of-day table ambient.
- `mEnv_GetRoomPrimColor`: room*(1-0.3 level) + point_base*0.6 level, clamped;
  `Global_kankyo_set_room_prim` sets it as prim right before ac_my_indoor draws
  floor/wall/steps. Disc floor (rel 0x594AA0) and wall (rel 0x595DB0) lists are
  G_LIGHTING with TEX0*SHADE -> PRIM*COMBINED and contain no SETPRIMCOLOR, so
  they inherit it. Furniture lists draw without a fresh room prim
  (aMR_DrawOneFurniture), so their inherited prim is order-dependent: not applied.
- Lights per object (`LightsN_list_check` with a position): point light becomes
  a directional light toward the light, colour x (1 - (d/r)^2). Furniture uses
  `aMR_GetFtrShapeCenter` (footprint centre, actor y); actors use world position
  (no actor sets ACTOR_STATE_LIGHTING); ac_my_indoor is a control actor at (0,0,0).
- Window light alpha (`mEnv_MakeWindowLightAlpha`) feeds only the
  ef_room_sunshine effects (player room scenes spawn ROOM_SUNSHINE). This port
  draws no room sunshine yet: follow-up.
- Furniture lamps are not light sources (their callbacks are static draws).

## Implementation

- `logic/src/lights.rs`: pinned Rustimal 122867f `lights.rs` subset
  (LightPoint, LightOut, LightsN::add_point_relative). Pool/absolute point/glow
  left in the pinned module (no caller); `Xyz` replaced by `[f32; 3]`; no
  source tests imported. Migration decision: partial import, source behaviour.
- `logic/src/room_light.rs`: native kernel (retail add_calc, default rule,
  RoomLight enter/tick/point_light/celestial_adjust/room_prim, SE ids).
  Pinned `kankyo_env.rs` covers periods/rainbow/window target but not the
  switch; its window target is not needed until sunshine exists.
- EnvUniform/shader gain one directional `point_dir/point_color` slot (black
  elsewhere). `BaseLight::switch_room_uniform` gives table ambient and
  byte-truncated dimmed sun/moon.
- `app/src/player_room/lighting.rs`: session `Lighting` in PlayerRoom, reset in
  `restore` (startup and F9) from the clock; entry detection per visit; Z
  (`Action::LightSwitch`, KeyZ) only with World focus; 60 Hz ticks (max 4/frame)
  with a pending trigger; info logs on entry/toggle. Render runs after
  update_environment and writes env per material (diff-checked, first root wins
  for shared handles, menu-lit materials skipped) for the shell (anchor origin,
  plus room prim), RoomPiece/loose/retiring pieces (root translation = shape
  centre) and the player hierarchy. No save schema change.
- Not covered: villager-room NPC schedule lights, shops/museum tables,
  basements/cottage, window sunshine, exact u8 table rounding of the blended
  period (Bevy blends floats; values truncated before use).

## Verification (Temp only, synthetic save, absolute EXE, explicit disc)

Real save SHA256 before and after runtime:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB (unchanged).
Preview %TEMP%/bevy-crossing-room-lighting (PID 39092, stopped): seed-12345 base
with blue/antique/snow/meza clocks. Logged sequence: `off on entry` at 15:38
(default off in daytime); Z -> `switched on`; F6 x7 to 22:38; Z -> off;
Z then Z after 0.12 s -> one `switched on` (second ignored during flicker);
Z with pockets open -> no toggle; Z after closing -> off; exit and re-entry ->
`off on entry` (session persistence); Z on; Z off, F9 -> `on on entry` (load
resets to the 22:xx default). F5 after toggles: home_furniture/pockets/
pocket_conds/bells equal to base, no light field in the save.
Centre-crop mean RGB of inspected screenshots: day off 85/81/87, day on 94/81/87
(warmer: sun x0.4 plus warm room light), night on 78/75/77, night off 50/54/57
(dark bluish moonlight). Screenshots inspected; no WARN/ERROR in logs (gamepad
mapping notice excluded). SFX ids are source-checked; audible acceptance not
claimed (audio diagnostics not run).
No-disc %TEMP%/bevy-crossing-room-lighting-nodisc (PID 21540, stopped): placeholder
town runs; Z outdoors does nothing; F5 preserves complete numeric state. The
placeholder player has no reflected name, so no-disc room entry was not driven;
render returns early without the disc light table.
Initial exit attempt walked south without facing the door, then the entry helper
teleported while still inside; excluded, recovered by a proper door exit.
No tests added or run.

Final gates: workspace check, all-target clippy -D warnings, fmt check, capture
build (18s after log edit), default build 36s, release build 2m32s; normal
project settings. Philip visual/audio playtest pending.
Note: the claim was made after the source audit and implementation (the claimer
needs an existing backlog file); recorded here and in comms.
