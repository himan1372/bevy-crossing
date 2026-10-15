# audio-acceptance — M4 combined pass

2026-10-15, codex. Philip requested this combined verification pass.
Automated pass complete; final combined listening/fallback UI check pending.
M4 audio remains open until that result and remaining scope are accepted.

- [x] Audit overlap, rooms/menus/pause, load/reentry and disc fallback.
- [x] Fix concrete lifecycle defects and add regression coverage.
- [x] Workspace check/clippy/fmt/tests/build and available disc audio probes.
- [x] Separate automated evidence from listening and hardware fidelity.
- [ ] Philip: final combined route below.
- [ ] Review remaining scope before closing audio-from-disc.

## Fixes from this audit

1. Leaving InGame previously only cleared ambience; the audio stream entity
   and Sound/Bgm resources could survive the state change. SoundStream now
   marks its owner; OnExit stops engine groups, despawns every stream owner,
   removes Sound/Bgm/diagnostic state, clears requests and resets voice/door
   playback. Repeat cleanup is safe. The old ambience-only exit hook is removed.
2. Successful F9 load previously left local delayed door/voice playback intact.
   Playback state is now resettable resources. Save emits ResetAudio only after
   a successful read/apply; LoadSet orders the audio reset before submission.
   It clears effects/voice/door/foot-contact requests, stops the voice sequence
   and restarts only SE. Existing music continues under the scene policy, and
   ambience is repopulated at the loaded listener position. Missing save emits
   no reset. No user save or settings schema changes.
3. Nonfinite input is already silenced by the mixer; dry-bus peak meters now
   also reject nonfinite values so one invalid value cannot poison diagnostics.

## Automated evidence

Final gate:
- cargo check --workspace: passed.
- cargo clippy --workspace --all-targets -- -D warnings: passed.
- cargo fmt --all --check: passed.
- cargo test --workspace --quiet: 624 passed, 0 failed, 22 ignored.
- cargo build --workspace: passed.
- git diff --check: passed.

Tests added for this requested acceptance pass:
- No-disc sound submissions drain without creating an engine/stream.
- Load reset clears pending voice/door/effects/foot contacts without a disc.
- Leaving play removes stream owners/BGM/diagnostics and tolerates repeat cleanup.
- Quiet mixer unity gain and additive Engine output contract.
- Dense synthetic stereo mix stays finite/below 0.98, preserves stereo ratio
  and recovers gain; reverb impulse has a localized decaying tail.
- Invalid samples do not poison output, delay state or meters.
- Supported accented charset round-trip; expression markers strip from plain UI.
- Deferred syllables retain their speaker ID and cancel on close; punctuation/mood
  change live slot parameters.
- Returning from a quiet house selects current-hour town music; menu priority
  and real-time volume restoration; stable bounded emitters, quiet gain,
  loss/range cleanup and left/right pan.

The first new impulse test used ring length minus sub offset for first return.
Source/read-cursor review corrected that expectation to the sub offset; the DSP
topology did not change.

Four explicit ignored probes passed with the local Animal Crossing disc:
    cargo test -p bevy-crossing sound::tests:: -- --ignored --nocapture --test-threads=1
BEVY_CROSSING_ISO was set only for that command to the local rom image.

| Probe | Evidence |
| --- | --- |
| Combined lifecycle | Offline music + voices + effects + river ambience, SE reset and music gain changes; finite output and full stop/tail decay. Raw/output peak 0.5171847, minimum gain 1.0, 0 limited frames, 0 nonfinite samples. Final voice snapshot 9/24. |
| Gameplay effects | All 29 existing UI/mailbox/surface IDs render non-silent finite PCM. |
| Register tail | MONO register retains 1.163 s under ordinary-slot traffic; old ordinary-slot route reproduces 0.437 s cutoff. |
| Villager types | Sequences 243/244/245 and return to 243 produce distinct non-silent output; peaks 0.29053..0.40650 in this run. |

These are offline renders, not device listening or exact retail comparisons.
Existing probe WAVs remain under Windows Temp/bevy-crossing-*; no disc content
is committed. Automated checks did not overwrite user savegame/settings.

## Fallback audit

- Existing disc parser tests reject other game IDs, non-disc data and corrupt FST.
- game_disc::start_disc_load reports NotFound/Failed and keeps GameDisc absent
  when discovery/load fails. Sound startup requires GameDisc; optional sound
  consumers now have a regression check for draining requests without one.
- The placeholder UI/game loop itself still needs the manual missing-disc check.
  No valid disc is renamed, moved or edited for this test.

## Philip's final route (about 5–10 minutes)

Use the rebuilt game; report any step with a wrong/stuck/cut-off sound.

| Step | Action | Expected |
| --- | --- | --- |
| 1 | Near river/insects: walk/run, talk, pockets/mail, then close. In the shop buy something, move the selection during the register tail, then return to walking. | Appropriate overlap; full register tail; no duplicated ambience or distortion. |
| 2 | Town -> house -> town -> shop -> town. F6 indoors, then leave. | House quiet without furniture music, correct shop theme, current-hour town music restored; outside ambience returns once. |
| 3 | Open/close menus and P pause/resume near ambience. | Music/ambience duck and recover, no stuck volume dip or extra footsteps. |
| 4 | F5 in town, walk elsewhere, F9; then walk/talk/cast again. Also F9 soon after traversing a door if practical. | Loaded location's audio resumes once; no delayed old door cue, ghost syllable or lingering old emitter. |
| 5 | Quit and relaunch normally. | One music stream, normal effects, no missing or doubled audio. |
| 6 | Close the game. Launch once with an explicit nonexistent path (command below), walk/open menus, then quit and relaunch normally. | Placeholder notice/game remains usable and silent; normal disc audio returns on normal launch. |

From the repository directory, without changing settings:
    .\target\debug\bevy-crossing.exe --disc ".\audio-acceptance-missing.iso"
Use a path that does not exist. The flag overrides disc auto-discovery for this
process only. A real wrong-game image is not required: parser rejection is covered
automatically; step 6 checks the shared failure/fallback UI route.

## Existing acceptance and scope still open

Philip has already accepted normal voices/F8 accents and music-note playback,
hourly/scene music, menus/letters/pockets, player steps, ambience and the initial
mixer/spatial work. These are feature-level results, not inferred completion of
every combined route above.

Keep these explicit:
- No composed/configured town tune yet; beep/other note playback is accepted,
  not complete melody identity or town-tune editing.
- Linear resampling/shared 24-voice pool, reserve lists and fixed-point DSP/
  filters/Haas/surround remain fidelity work (audio-mix-and-fidelity.md).
- Weather/pond/frog/extra insect coverage, full source camera/actor behavior,
  absent reel-fight/catch/door animation phases, furniture audio and special
  speaking interactions/configuration remain gameplay-dependent follow-ups.
- Full retail menu visuals, render-look sign-off, retire-town-gen and deferred
  title scope remain separate M4 items; this pass does not schedule M5 early.

Commit/push authorized. Exclude unrelated .gitignore/savegame.json edits.
