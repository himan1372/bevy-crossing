# Music source fidelity follow-ups

Philip explicitly requests roadmap item 3: original music multi-mark take-out,
submenu/cloth transitions, placement selection/toggle/contact and device audio
filters/hiss. Audit original C and existing native kernels before adapting.
Environmental pinwheel wind depends on weather and stays separate until that
system exists. Preserve accepted playback, net/toy behavior, numeric transactions,
settings, unrelated .gitignore and actual save. No tests or actual-save automation.
Starting actual-save SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

Use original runtime-disc assets only; evidence/synthetic saves/helpers in Temp.
Update roadmap.html, ROADMAP.md and comms with implementation, bounded evidence
and remaining work. Check/clippy/fmt and capture/default/release builds; close
own disposable previews and release claim before publishing authorized changes.

## Source audit and implementation

- m_tag_ovl music_select_many / music_takeout: marked cursor offers Take out
  marked / Quit; unmarked cursor retains Listen / Take out / Quit. M maps the
  original X marking action. Only collected titles can be marked. Return marked
  titles in ascending source index until pockets fill; preserve unreturned bits.
  Validate each return before clearing registration/references. Full pockets
  refuse without numeric mutation and retain transient marks after the warning.
- m_music_ovl original mus_mark assets: 16x16 legacy row-major IA8, mirrored
  twice across the 0..32 texel coordinates; original triangular green pulse.
  This uses a scoped decoder override, not a global GX behavior change.
- m_submenu_ovl move_menu and pinned 122867f submenu translation audited:
  60 Hz accumulator/alternate speed steps, incoming -300 to0 with slowdown past
  -120; successful action exits right, cancellation left, accelerating to75.
  Texture advances0.3535 per60 Hz tick; music tile origin multiplies by4 before
  the texture fractional conversion, so cloth phase uses the full source step.
  Existing PocketPattern material is reused with panel-local offset.
- aMR_MiniDiskCommonCt: placement selects a registered ordinal, power stays off;
  reload retains selection. App uses fresh OS-randomized hash entropy to choose
  the ordinal; it does not alter town/save RNG. Empty collection stays empty.
  aMR_Check... MUSIC_DISK home branch requires front contact; app now enforces
  the same front-side/facing guard as existing storage. Menu Start/Stop reuses
  exclusive source switches and LIGHT_ON0x16/OFF0x17 effects. The30-tick quick
  toggle belongs to the non-player-room path, not this home menu.
- Na_MDPlayerPos device classes:0x30/1.08 plus level0x4E hiss for five older
  players;0xE3/1.4 for three radios/tape decks; zero/unity for modern devices.
  Existing profile gain classes retained. Nas_SetBPFilter table indexing read
  exactly as C from runtime DOL LSF_TABLE/BP_FILTER; no embedded coefficients.
  SoundData exposes optional taps; source command0x13 assigns each subtrack's
  owned filter. Per-voice CPU eight-tap Q15-scaled convolution precedes dry/wet
  sends, retains release tails and clears history on voice/filter change. Both
  BGM handles receive/reset commands. Other effects/voices are not filtered.
  Hiss uses the existing four-source ambience mixer, with a distinct stable key
  alongside pinwheel level sound; no extra stream or independent loop layer.

## Bounded evidence

All helpers/synthetic saves/screenshots/logs are Temp-only under
bevy-crossing-music-source-fidelity. Actual save never loaded by automation.
Multi-take0/35/54 returns exact IDs10752/10787/10806 across both collection words,
clears active selected references; two-free-pocket case returns0/35 and leaves54
registered. Full-pocket refusal, unregistered marking/cancel and F5/F9 preserve
complete home_furniture/pockets/conditions/wallet. Unmarked35 takes only35 while
0/54 remain registered. Reload clears marks and overlay.

Newly placed7772 selects one of registered0/35/54, stays off, consumes exactly
one pocket item and preserves selection on reload; front Start/Stop roundtrip
preserves layers. Back contact screenshot shows no menu; pockets can still open.
First contact helper used too-short turn duration and failed its front check;
excluded and repeated after fully establishing facing. Phonograph7672,
radio7684 and modern player7772 play/save unchanged through all three device
classes. Diagnostic output finite, no limiting/nonfinite, hiss ambience peak
0.082297884 and active voices settle0 after stop. This is not listening approval.

Initial mark decoder caused zero-size texture/GPU failure; corrected locally.
Subsequent visual checks found legacy row-major/mirrored texture requirements and
warning depth ordering; corrected and final repeat passes. Verified screenshots
show mirrored circular green marks and readable warning above the collection.
BRP confirms warning panel z=-20 then0 after dismissal, retained two-choice marked
menu, normal three-choice menu after unmark, incoming negative x to0, successful
exit positive x then removal, cancellation negative x then removal. Complete
numeric state unchanged after warning/confirm/cancel. Exclude pre-fix captures
from acceptance evidence. Final verified disc log has no WARN/ERROR. No tests
added/run.

Fresh missing-disc preview12128 displays expected placeholder notice; F5 keeps
exact home_furniture/pockets/pocket_conds/bells, screenshot inspected. Both final
own previews36828/12128 closed before default/release builds. Earlier final
preview36132 exited normally with window closure. Actual save SHA256 still the
starting baseline; settings/.gitignore/handoff preserved.

## Remaining gates and limits

Final check/all-target clippy -D warnings/fmt/capture build pass (capture12.63s).
Default workspace build passes (41.31s), release workspace build passes (2m57s).
All required implementation/build/preview checks for this slice are complete;
release claim and publish authorized source/docs. No own previews/browser/Cargo
left running. Actual save hash matches the starting baseline.
Browser roadmap checks pass:72 items,39 done,
6 in progress,27 planned; remaining33 excludes done; search works and no overflow
at1280/390 widths. Exact RSP fixed-point/filter rounding/resampling, full
warning/choice-window animation and retail comparison remain broader fidelity
work. Pinwheel wind needs weather. Philip acceptance of these additions, M4 and
migration remain open; previous music playback acceptance stays bounded.
