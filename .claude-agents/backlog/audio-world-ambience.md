# audio-world-ambience — M4 continuous town sounds

Requested by Philip after action audio (2026-10-14 local).

- [x] Trace original water/insect sources, IDs, spatial rules and level-slot handshake.
- [x] Integrate safe continuous playback with stable emitter ownership; prevent duplicated or stuck loops.
- [x] Wire current town terrain and applicable live insects; silence outdoors when inside and clean up removed sources.
- [x] Compile, lint, format and build. No tests unless requested.
- [ ] Philip: river -> waterfall -> shore, move away/inside/outside, nearby audible insects, pause/resume/restart.

Wind/rain depend on absent weather state; do not invent weather. Existing effects/music/voices remain accepted or retain their recorded listening status. Document unsupported insect states explicitly.

## Implementation and resume notes

- logic/src/ambience.rs: four nearest sources, stable u64 identities/slots,
  540-game-unit range, source distance2vol and angle2pan; original marine block
  indices extracted from m_random_field.c, not inferred from nearby sound data.
- app/src/disc_town.rs reads six {kind:i16,ut_x:u8,ut_z:u8} records after each
  collision grid (data_bgd +0x41C). GroundMap stores runtime metadata only.
  River ID 0B / shore 1C; original +175-game-unit shore Z offset and source
  height +40 applied. field_draw's nearest-two adjacent-acre / shore-row search
  is retained. Waterfall 0C uses one collision-footprint centre per acre until
  actual waterfall actors/positions are wired; this is an explicit approximation.
- app/src/ambience.rs reports current sources in PostUpdate after transforms,
  using the fixed source microphone offset player+(0,240,77) in game units.
  Scratch storage is reused. Indoors, missing data/listener and scene exit
  clear desired voices; disappeared/caught/fleeing insects are refreshed by AI.
- Engine::set_ambience / advance_levels reuse LevSeManager on SE tracks 8-11.
  Continuous IDs mask bit 7 (echo flag); they are not ordinary trigger IDs.
  Start port 0 only once per sound; refresh gain/pan without restarting.
  Stop cancels pending port 0, writes port 5=-1 and silences gain. Replacement
  waits four audio updates before submitting a new start. Sequence stop/restart
  resets this state. Six trigger slots, MONO 14 and music/voices stay separate.
- Insect::level_sound: calm cicadas 9B/9A/98/97; grounded calm crickets
  9F/9E/A0/9D; mosquito CF except attack; available cicada/ladybug/mantis/roach
  flight 26 and beetle/skater flee 25. Conditions come from ac_ins_* state code,
  not a global seasonal layer. Crickets stop hopping/fleeing, cicadas stop
  singing when frightened, all cease reporting on catch/despawn.
- Pause/inventory/mail half gain per Sou_LevMake. Current weather remains
  clear because simulation does not expose rain; no guessed wind/rain loop.

Remaining: live listening/loop-lifetime confirmation; full waterfall actor and
camera-mic parity; pond/frog coverage; mole cricket/bee and additional movement
sound phases; room reverb/echo hardware fidelity. These are not claimed complete.
Action cues and trigger overlap also retain their recorded pending acceptance.

Stopping request: Philip asked to reach a stopping point because usage is low.
Finish checks, commit/push the audio batch, release the claim and stop. Next
session begins with listening feedback, not another feature. Restart; river ->
waterfall -> shore, walk away, enter/exit a house/shop, listen near singing/flying
insects then move away/catch one, pause/resume and check no doubled or stuck loops.

Workspace check and all-target clippy passed. No tests or disc probes added/run.
Formatting and workspace build passed. Listening remains pending.
