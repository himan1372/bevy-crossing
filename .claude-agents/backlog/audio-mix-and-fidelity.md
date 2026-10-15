# audio-mix-and-fidelity — M4 mixer and spatial effects

Implemented 2026-10-15 by codex; Philip's listening acceptance remains pending.
Request: room reverb, sound balance, remaining positional effects/clipping.

- [x] Audit original room/scene reverb, sends, gains, resampling and allocation.
- [x] Implement source-backed wet processing and preserve accepted sequence/voice/effect gains.
- [x] Route available world effects through shared listener, range, gain and pan.
- [x] Add pre/post-output peak visibility and linked stereo peak protection.
- [x] Final compile/lint/format/build passed; no tests/probes added/run.
- [ ] Philip: room/town transitions, balance, near/far/offscreen sources, dense overlap and tails.

## Implementation

- logic/src/audio_mix.rs: float CPU approximation of NA_HALL_DELAY. Original
  preset is downsample 1, ring 32*64, feedback 0x4500, sub offset 20*64,
  sub gain 0x1100, return 0x7fff; no cross leak/filter/second reverb.
  Ring lengths scale by the existing engine's 140/200 output/spec mapping.
  Return main tap to dry, decay main in place, add sub tap + note sends at
  the offset. Wet/dry note gains ramp per update; released notes retain send.
  Tiny tails flush below 1e-15 to avoid denormals. Buffers are reused at the
  live fixed block size; no per-update DSP allocations.
- nas_seq now honors subtrack reverb index (E5/E7/E8); only index 0 is active,
  matching NA_SPEC_CONFIG. USE_SUB_REVERB selects live sub send; otherwise
  retain the layer send. Low seven bits normalize by 128 as the RSP does.
- Scene policy follows source: normal houses/shops share mode 2, not invented
  small/large-room presets. Indoor steps explicitly send 5; outdoor steps
  send 0. Normal typed voices and UI have no blanket added echo. The shared
  delay naturally keeps tails over group/room transitions.
- app sound queue distinguishes centered UI/player cues from world sources.
  Bobber landing/retrieval, net swing/catch, field pickup and physical mailbox
  flags now use shared fixed-camera microphone, original 540-unit range,
  quadratic distance2vol and angle2pan. NPC steps gain pan while retaining
  their existing distance-based level; player step gains remain unchanged.
  Menu sounds, player rod/rustle/quest cues and teleport door cues stay
  centered. Door cues cannot follow both widely separated room coordinates
  across instantaneous traversal; actor animation remains separate work.
- Effects flush in PostUpdate after transform propagation and foot contacts.
  Ambient loops retain accepted slot/lifetime behavior and microphone.
- No arbitrary bus rebalance: original sequence gains, voice slot volume/pitch,
  scene ducks and accepted footsteps remain the baseline. Four dry-bus peak
  meters (effects/music/voices/ambience) allow an informed listening adjustment.
- Protection is a port adaptation: instantaneous linked stereo attack,
  0.98 ceiling, 50 ms recovery, unity below ceiling after recovery.
  Includes wet return; final live stream clamp remains a safety net.
  Engine::update still ADDS to caller output; callers must clear their buffer.
- Set BEVY_CROSSING_AUDIO_DIAGNOSTICS=1 before starting to log every five real
  seconds: cumulative dry bus/mix/output peaks, minimum gain, limited frame
  count, nonfinite count and current voice load. Disabled by default.
  No logging on the audio thread. Shared wet return is included in mix peak,
  not attributed to a dry bus.

## Primary source audit

Original port root: ../shmanimal crossing/ACGC-PC-Port/
- src/static/jaudio_NES/internal/audioconst.c: NA_HALL_DELAY, NA_SPEC_CONFIG.
- internal/memory.c: Nas_SetDelayLine and delay parameter allocation.
- internal/driver.c: Nas_CpuFX sub offset, Nas_Load/SaveAuxBuffer1[_B],
  wet return/feedback, note envelope send and RSP resampling/filter commands.
- internal/track.c: subtrack reverb volume/index and packed parameters.
- internal/channel.c: retained release parameters and Nas_AllocationOnRequest.
- src/game/m_field_make.c: room sound mode 2, museum/lighthouse modes 15/16.
- game/game64.c_inc: Na_PlyWalkSeRoom/Na_NpcWalkSeRoom (normal send 5),
  Sou_PosTrgStart, distance2vol, angle2pan, scene echo toggles.

## Explicit remaining fidelity gaps

These are not claimed complete by this implementation:
- Float delay approximates the RSP's signal topology, not fixed-point rounding,
  saturation, optional filters, Haas/headset delay or surround phase behavior.
  The accepted port microphone remains the fixed camera approximation.
- Linear PCM interpolation remains; driver.c uses filtered aResample.
  No speculative interpolation replacement or runtime resampling comparison.
- 24 voices still share a pool. Original allocation policy bits choose
  subtrack/group/global reserve lists and previous-note reuse; current allocator
  only handles free/releasing/lower-priority selection. Stress listening and a
  deliberate reserve-list port remain before hardware-fidelity claims.
- Museum/lighthouse scenes are not playable here; their echo settings (step 50,
  trigger/level echo) are documented, not enabled in ordinary houses.
- Existing echo flag/DIST_REVERB corner behavior, stereo-phase/reverb high bit,
  camera rotation and source smoothing need a broader faithful DSP/game-layer
  pass. Current world triggers supply reverb 0; indoor steps explicitly use 5.
- Action animation timing, missing ambience/weather systems and voice edge cases
  remain in their own tasks.

## Listening checklist (Philip)

Restart the rebuilt game.
1. Walk town -> house -> town -> shop -> town. Listen for modest indoor step
   tails, no blanket echo on UI/Animalese, no stuck tail or wrong music/duck.
2. Talk while music/river/insects are nearby; use pockets/mail/pause and close.
   Voices should remain clear; levels must recover without pumping or distortion.
3. Cast left/right, retrieve the bobber, swing/catch with net, collect an item,
   and pass a walking villager/mailbox. Check stereo direction and near/far cutoff.
4. Overlap footsteps, rapid cursor feedback, register, voices and ambience.
   Listen for clipping, abrupt truncation or persistent level dips.
5. If anything sounds off, enable diagnostics and report the location/action
   with its peak/min-gain/limited-frame values. Cumulative peaks reset on restart.

## Validation / handoff

2026-10-15: cargo check --workspace, cargo clippy --workspace --all-targets --
-D warnings, cargo fmt --all --check, cargo build --workspace and git diff
--check passed. No tests or disc probes added/run under the standing
instruction. Local qwen2.5-coder:7b reviewed the small mixer; its suggestions
were generic or conflicted with the intentional additive output contract.
No actionable finding was accepted without source/code review.

Keep unrelated .gitignore/savegame.json changes excluded from commits.
Commit/push authorized. Voice edge cases (roadmap 7) and combined acceptance (8)
follow the listening check; M4 audio remains open.
