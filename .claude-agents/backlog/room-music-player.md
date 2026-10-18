# Room music players

Philip authorizes registration and song playback through player-room furniture,
then an updated docs/roadmap.html showing implementation and proposed next work.
Net appearance is deferred explicitly.

Scope: source MUSIC_DISK furniture, shared home song collection, pocket
registration/duplicate refusal, selection/stop/removal, per-player selected song
and switches, save/F9, existing disc sequence player and scene arbitration.
Use existing modal UI first; original music-grid presentation and per-device
speaker effects/animation remain explicit follow-ups. Preserve original numeric
room/storage data and the actual save. No tests added/run without request.

Validation: workspace check, clippy, fmt; capture/default/release builds;
disposable save/runtime checks of registration, playback, duplicates, full
pockets, multiple devices, reload, removal and scene changes. Philip listening
and visual acceptance remain separate.

## Implementation and source audit

- ac_my_room_msg_ctrl: shared two-word home music_box; registration sets the
  bit and consumes exactly one normal pocket copy, duplicates preserve it.
  Selecting/registration assigns actor.items[0] and turns other MD devices off.
- m_tag_ovl: single-title take-out returns the copy only with a free pocket,
  clears the collection bit and all device references to that title.
- ac_my_room: MUSIC_DISK profile bit8, AllMDSwitchOff includes aerobics devices;
  registered selections reference the shared collection when picking up a leaf.
  Existing native owner-layer save-back/restore and furniture relocation remain.
- audio_defs BGM_MD0=128; existing runtime SEQ_TABLE resolves 128+song index.
  The existing BGM owner selects these only in the player's home, after native
  quiet-house policy. Selection/stop has no song crossfade; scene transitions
  retain existing policy. No second stream/parallel music engine.
- RoomSave adds music_box:[u32;2], default zero for old saves. Four numeric layers,
  per-device selections/switches and unrelated storage remain in their old form.
  No-disc saves preserve all these numbers without needing furniture profiles.
- E on an empty-handed targeted player offers register/choose/stop/take/quit.
  Registration uses pockets with only normal music items eligible; the shared
  list has three titles per page plus next/previous/quit. F2 inside the home
  adds song0x2A23 (K.K. Love Song) to a free pocket as an explicit playtest aid.

## Bounded runtime evidence (2026-10-18)

Disposable Temp bevy-crossing-room-music-review, no actual-save automation:
first/song35/last registration preserves exact IDs and two-word bits; duplicate
refusal has complete room/pockets/conditions/wallet equality. Two tape decks use
the same collection and exclusive switches. Stop is quiet; selection returns
to the right original sequence (128->111,163->146,182->165 observed). Single-title
take-out returns a copy and clears both matching device references; full pockets
refuse with exact numeric equality. Picking up a selected deck gives only its
leaf and keeps the collection. All55-title pagination next/previous/cancel and
last title checked. Initial fast scripted menu attempts outran text reveal;
excluded and repeated from confirmed screens with sufficient reveal time.

F5/F9 and a fresh process preserve collection/selection/switches. Active tape deck
push shifts selected title owner34->18 and switch512->2; F9 roundtrip is exact.
House exit restores town music; entry resumes the selected room song. Empty
library/no carried songs are safe notices. F2 yields a normal song. Fresh-process
diagnostics show finite mix/output, no limiting/nonfinite samples (counters
cumulative, including town/door traffic); this is not a listening comparison.
Main/fresh disc logs have no WARN/ERROR. Fresh explicit-missing-disc preview
shows the expected fallback notice; F5 preserves complete room/pocket/condition/
wallet state. Screenshots/scripts/synthetic saves/logs remain in Temp only.

docs/roadmap.html: latest work and proposed order, playtest controls/acceptance,
net deferral, duplicate removal and corrected migration/clock/asset facts; status
counts derive from71 items (36 implemented/6 in progress/29 planned). Temporary
headless browser checks search, remaining-work filter (35, zero implemented),
desktop1280 and mobile390 layouts with no horizontal overflow. Browser closed.

Check/all-target clippy -D warnings/fmt/capture/default/release builds pass
(final capture19.96s/default46.98s/release3m37s). No tests added or run.
No own preview/Cargo/browser remains. Ollama remains unavailable per the prior
handoff; direct source review used. Actual save baseline still
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

## Remaining work / acceptance

Original 55-song grid/art, original device open-close/player clips, device speaker
position/distance/filter character, source random selection on placement and
toggle delay/contact behavior are not claimed complete. The menu provides the
requested registration/playback behavior now. Multi-mark take-out and source
special storage scripts remain separate. Philip listening/visual acceptance,
M4/audio/migration remain open. Codex proposes music presentation, toy use,
remaining item conversions, then room decorating/special actions; not started.
