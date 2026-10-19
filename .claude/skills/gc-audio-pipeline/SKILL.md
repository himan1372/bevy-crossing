---
name: gc-audio-pipeline
description: "Trace original disc music and sound through the existing Nas engine, audio triggers and mixer; use for playback, timing or lifecycle changes."
---

# Runtime disc audio

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

## Follow the existing stream

`app/src/sound_rom.rs::SoundRom` reads DOL audio tables and `audiorom.img`. The bank/sample decoder and `logic/src/nas_seq.rs` feed the jaudio_NES/Nas sequence engine. `app/src/sound.rs` bridges game commands into one custom Bevy audio stream. This is not an assumed BMS/WSYS-to-WAV pipeline.

Locate the original C caller and the current Rust trigger before changing playback. Inspect `logic/src/audio.rs`, the Nas engine/bank modules and app ambience/footstep/dialogue callers as relevant. Preserve sequence timing, pitch, sample loop points, gain and source filter tables. Do not batch-normalize samples or replace native music with rendered tracks.

## Trigger and retire correctly

Use the existing audio port helpers, including `se`/`se_at`, music/group handles and stable level-source keys, as appropriate to the current caller. Do not spawn a parallel `AudioPlayer` for every effect or introduce another music controller.

Check start, update and stop ownership. Repeated proximity updates must not restart a loop each frame; scene exit, pause and QuickLoad must not leave voices or stale level sources behind. Follow the existing `ResetAudio` path and menu/music ducking rules rather than adding isolated cleanup.

## Verify the claimed behavior

For clipping, nonfinite samples or voice leaks, launch the isolated preview with `BEVY_CROSSING_AUDIO_DIAGNOSTICS=1` and inspect its bounded diagnostics. Compare the source trigger/timing and lifecycle transitions; listening is needed for audible acceptance. Clean counters do not prove the sound matches retail, and existing synthesis/resampling approximations remain unless the task addresses them.

Keep any diagnostic audio in Temp; retain runtime disc loading and missing-disc behavior. Exporting WAVs, installing a soundfont/MIDI toolchain or adding a new mixer is separate work, not a prerequisite for fixing this path.
