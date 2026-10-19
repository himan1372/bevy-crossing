---
name: gc-audio-pipeline
description: Takes GameCube disc audio from extracted files to playable Bevy sound — format map, decode recipes, and the bevy_audio triggering cookbook. Use when the task involves getting any original sound or music from the AC disc into the game; pairs with gc-iso-reader (extraction) on the front end.
---

# gc-audio-pipeline

Covers the whole path: identify the disc audio format → decode to a Bevy-playable file → trigger it from game logic with `bevy_audio`. Extraction of raw files from the ISO is `gc-iso-reader`/`iso-reader`'s job; this skill starts where they stop.

> **Open question (not yet verified):** which audio formats the Animal Crossing ISO actually uses — sequenced JAudio (BMS) vs. streamed DSP — is still unknown. This skill covers both paths; check with the `gc-iso-reader` skill owners (or probe the ISO's audio directories) to prioritize. Don't assume one path and skip the other.

## Step 1 — Format map: what GameCube audio looks like

Two fundamentally different kinds of audio live on GC discs. Identify which you're holding before picking a tool:

**Streamed audio** (pre-rendered sound, decoded sample-by-sample):
- **DSP ADPCM** streams (`.dsp`, IDSP containers) — Nintendo's 4-bit ADPCM format. Most SFX and streamed music.
- **BRSTM** — streamed format with embedded loop points (common for looping BGM).
- **HPS** — another streamed container seen on GC discs.

**Sequenced audio** (music as note data + instrument samples, like MIDI + soundfont):
- **JAudio / JaiSeq**: sequences in **BMS** files, instruments in **IBNK** banks, samples organized via **WSYS** (wave systems), often packed into **AAF**/**BAA** audio archives.
- Terminology for Nintendo EAD titles (AAF/IBNK/WSYS/BMS and how they relate) is documented in the [JAMTools docs](https://xayr.gay/share/08-2022/jamtools.pdf) — read this before touching sequenced audio; the acronyms are meaningless without it.

Rule of thumb: short one-shot sounds (UI blips, footsteps, item jingles) are streamed DSP; background music may be either — check the file extensions and sizes in the extracted audio directories.

## Step 2 — Decode recipes

Goal of this step: get to **WAV** (or OGG/FLAC) files Bevy can play. Pick the recipe by format:

**Streamed (DSP/BRSTM/HPS) → WAV:**
- **vgmstream** decodes DSP ADPCM and BRSTM to WAV and **preserves loop points** — critical for looping BGM; don't re-encode in a way that drops them. Reference: [vgmdocs conversion tools](https://loveemu.github.io/vgmdocs/Conversion_Tools_for_Video_Game_Music.html).
- **vgmtoolbox** covers extraction/carving of DSP streams and RARC layout archives when files aren't cleanly separated. https://vgmtoolbox.com/

**Sequenced (BMS/IBNK/WSYS/AAF) → playable:**
- **VGMTrans** (Jasper's fork, which supports GameCube JaiSeq + sample export) — for inspecting sequences and exporting samples/instruments.
- **bms2mid** converts BMS sequences to MIDI, which can then be rendered with any soundfont/synth pipeline into WAV/OGG.
- This path is more work than streamed: sequence → MIDI → render → loop-point handling. Budget accordingly and say so in the task note.

**Verification checklist for this step:**
- [ ] Decoded WAV plays correctly in a desktop player (no static, correct pitch/speed — wrong decode parameters produce chipmunk audio or noise, not errors).
- [ ] Loop points survived for looping BGM (check in the WAV metadata / vgmstream output).
- [ ] Loudness is roughly consistent across the batch (normalize as a batch decision, document it).
- [ ] Files land in `assets/audio/<sfx|music|...>/` with disc-traceable names; log the decode recipe (tool, version, settings) next to the batch.

## Step 3 — bevy_audio cookbook (0.17+, rodio-based)

Bevy 0.19's audio stack is `bevy_audio`. The core types:

- **`AudioPlayer(Handle<AudioSource>)`** — component; spawning an entity with it starts playback.
- **`PlaybackSettings`** — loop mode, volume, speed, paused, muted, spatial, spatial_scale. Set these at spawn time; don't spawn-then-tweak in a second system unless the setting genuinely changes at runtime.
- **`AudioSink` / `SpatialAudioSink`** — runtime control (pause/resume/volume) of an already-playing sound; query for the sink tied to the entity.
- **`SpatialListener`** — exactly one per app; spatial audio is stereo panning only (no HRTF), so use it for positional world sounds, not for UI/music.
- **`GlobalVolume`** — resource for master volume; wire it to the settings UI, not per-sound volumes.
- **`Decodable` trait** (via `AddAudioSource`) — only reach for this if you need a custom source type; decoded WAV/OGG/FLAC don't need it.

**Cargo features** (in the `app` crate's bevy dependency): enable per format — `wav`, `flac`, `ogg` (vorbis via symphonia). `mp3` exists but is not recommended; prefer OGG for compressed music. If a sound silently doesn't play, the missing format feature is the first thing to check.

Reference: [taintedcoders Bevy audio guide](https://taintedcoders.com/bevy/audio), [bevy_audio docs](https://docs.rs/bevy_audio/latest/bevy_audio/).

## Step 4 — Event → sound triggering pattern

Game logic (in `logic/`) must not touch audio directly. The pattern:

1. `logic/` emits plain **events** for things that make sound (`ItemPickedUp`, `DoorOpened`, `UiClick`, `MusicZoneEntered`).
2. In `app/`, one **audio system** (or a small set, grouped by category: UI / world / music) observes those events and spawns `AudioPlayer` entities with the right `PlaybackSettings`.
3. Asset handles for sounds are preloaded at startup (don't `AssetServer::load` inside the event handler on first trigger — that's a hitch).
4. Music transitions (zone changes, time-of-day) go through a dedicated music manager resource, not ad-hoc spawns — overlapping BGM spawns are the classic bug.

Pseudocode shape (adapt to the actual `app/` structure):

```rust
// preload once
#[derive(Resource)]
struct SoundBank { click: Handle<AudioSource>, pickup: Handle<AudioSource>, /* ... */ }

// trigger from logic events
fn play_ui_sounds(
    mut events: EventReader<UiClick>,
    bank: Res<SoundBank>,
    mut commands: Commands,
) {
    for _ in events.read() {
        commands.spawn((
            AudioPlayer(bank.click.clone()),
            PlaybackSettings::default(), // or DESPAWN_ON_FINISH config
        ));
    }
}
```

Despawn finished one-shots (or use the despawn-on-finish playback mode) — leaked audio entities accumulate silently.

## Step 5 — Verify by ear, in the running game

1. Trigger each sound in the running game via the BRP runbook (spawn the event / drive the UI per `brp-visual-qa`'s scripted-drive pattern).
2. Confirm: correct sound, correct timing (no load hitch on first play = preloaded), correct volume balance, music loops seamlessly (no click at the loop point = loop points survived).
3. Spatial sounds: move the player and confirm panning follows position; confirm UI sounds are NOT spatial.

## Sources

- [vgmdocs — conversion tools for video game music](https://loveemu.github.io/vgmdocs/Conversion_Tools_for_Video_Game_Music.html)
- [JAMTools docs — Nintendo EAD audio terminology (PDF)](https://xayr.gay/share/08-2022/jamtools.pdf)
- [vgmtoolbox](https://vgmtoolbox.com/)
- [taintedcoders — Bevy audio guide](https://taintedcoders.com/bevy/audio)
- [bevy_audio docs](https://docs.rs/bevy_audio/latest/bevy_audio/)
