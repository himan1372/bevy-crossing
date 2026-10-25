# Clothing try-on sparkles and wear sound

Philip selects priority3 after shop-category-previews (HEAD6dd71f34).
Bounded next shop detail: original ef_kigae/ef_kigae_light and NA_SE_WEAR;
other shop services remain separately scoped. Parent retail-shop-ui stays open.

Design: on each MENU_CHANGE1 crossing frame16, trigger one positional0x11C
and a60tick spiral emitter at the source feel joint. Every second tick emits
the native ef_takurami01_kira_modelT sparkle at radius22, descending angle9deg,
jitter+-2.5 and source timer-based height. Each child lasts30ticks, falls with
acceleration-.025, expands0->.009 over10ticks and shrinks over20. Reuse the
existing gong sparkle geometry/legacy IA8 decoder with native white alpha255.
Billboard to WorldCamera. No invented smoke/cloud texture or saved state.

Plan (inline execution; original C defines design):
- [x] Verify effect timer/feel joint and audio source/available asset helpers.
- [x] Add focused preview/effects.rs lifecycle and frame-crossing trigger;
  reset between gestures, on scene exit/load/AppState exit, freeze with gameplay.
- [x] Sequential check/clippy/fmt/capture/default/release. No test suites.
- [x] Synthetic Temp QA: try-on and return timing, visual spiral, exactly one
  trigger per gesture, skip/wall/floor no trigger, repeated preview, cleanup,
  complete saved-field comparisons and real-save hash only.
- [x] Independent review, notes/parent/comms/both roadmaps, complete own claim,
  explicit commit and push. Listening evidence distinct from trigger evidence.

Implementation/source evidence:

- MENU_CHANGE1 frame16 in m_player_main_change_cloth.c_inc calls wear plus
  KIGAE; m_player_sound.c_inc routes NA_SE_WEAR=0x11C (audio_defs.h) at actor
  position. m_player_draw.c_inc callback25 supplies feel_pos. Native short
  angular step is1638, startangle+16384; timer59 at second-step first emission.
  Child scale peaks.009 at age10, then returns to0 at30; acceleration-.025.
- Focused effects.rs uses previous propagated feel-joint transform, as native
  move reads the previous draw callback, and current animated root yaw. It
  keeps a gesture serial/once latch separate from frame sampling so both
  directions fire once even if a frame crosses16. se_at uses existing stream.
  Cached native geometry and IA8 image use shared gong helpers; gong callers
  still pass alpha200, try-on passes255. Original model ENV yellow preserved.
- Particle simulation uses the native capped60Hz accumulator; 60tick emitters
  emit every2 ticks, children fall/shrink for30 ticks. Billboard follows world
  camera; no saved fields or external assets. Effects finish naturally after
  a gesture and clear on leaving shop, ResetAudio or OnExitInGame; GameplaySet
  freezes simulation. Review requested explicit after LoadSet; added before QA.
  Final independent review has no remaining important findings.
- Temp evidence: change-effects-8693249d076d4693800f1913623f5919 under
  %TEMP%/bevy-crossing-room-critters. Two try-ons/two returns captured at
  .32/.52/.72/.95/1.3/1.8/2.5sec. Original spiral/sprite rendering inspected.
  Exactly one logged0x11C enqueue per gesture, at sampled frames16.54,16.65,
  16.22,16.70; peak15live particles, maximum worldscale.000225, zero at3sec.
  JSON samples assert finite positions, scales and expiry.
- effects-before versus both cancels, shirt skip, wall/floor previews and
  reload/reentry: every one of39 saved fields equal except position/playtime.
  Those non-clothing paths add no wear event and leave no sparkle entities.
  Synthetic stock renewed at startup before baseline; actual captured shirt
  is pop bloom shirt0x247B, with the source reserve layout used for approaches.
- Audio diagnostics enabled: effects bus reaches~.2965; compared mix/output
  peak~.498, min gain1, zero limited frames/nonfinite samples. No audible
  listening comparison performed; timing/source ID and engine diagnostics
  are evidence of correct triggering, not retail listening acceptance.
- No test suites added or run. First clippy flagged modulo style; replaced
  with is_multiple_of. Review/source cleanup ordering corrected before final
  capture. Missing effect-only symbols and forced pause/AppState invalidation
  are source-reviewed, not fabricated at runtime. Gong alpha callers retained
  by source review; no gong playback regression scenario was repeated.
- Explicit verified reentry used the fixed synthetic town approach after F9;
  confirmed player at shop x3000 before retrying. Initial follow-up purchase
  driver had teleported into the shop geometry while still in Town state;
  no transaction occurred. Corrected setup and retried through the real door.
  Successful preview purchase:0x247B price360, Bells80000->79640, onlystock2
  sold, ticket2C48, shop_sales+360. All unrelated saved fields preserved; two
  additional once-only events at frames16.02/16.08, no residual particles.
- Final diagnostics peak mix/output .522, gain1, zero limited/nonfinite samples.
  Final disc log no ERROR/WARN/panic. Explicit missing ISO opens/closes plain
  pockets and has no wear events/sparkles; image inspected (expected missing
  disc warning). Owned30648/13912 previews closed.
- Gates: check37.05s, final clippy20.61s, fmt, capture12.83s, default1m10s and
  release5m08s pass, one Cargo process at a time. Real save only hashed before/
  after, unchanged C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

This completes the clothing effect detail; other shop services, parent task/M4
and Philip acceptance remain open. Next options: remaining shop services,
retail pause/save presentation, special quest/mail tags.
