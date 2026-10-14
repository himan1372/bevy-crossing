# villager-voice-types — investigate indistinguishable villager voices

M4 audio follow-up, requested by Philip 2026-10-14. Philip confirms hourly
music and audible Animalese, but has only heard one villager voice type.
Audio-from-disc remains claimed by claude; this focused investigation is separate.

- [x] Compare personality-to-spec mapping and voice sequence selection with the original C: both match (mNpc_GetNpcSoundSpec and Na_SpecChange).
- [x] Trace speaker changes through dialogue, the voice queue and sequence playback. Fixed Reveal: its key now includes the speaker entity so identical text spoken by a different villager types/voices again. No fault forcing every villager onto one sequence was found.
- [x] Provide a repeatable in-game comparison: F8 from world input cycles residents by personality, including those indoors, speaking the same line with their real speaker entity. Wait for typing, E closes, F8 selects the next. Name plate identifies the resident; console logs personality, spec and sequence.
- [x] Record findings and checks below.
- [ ] Philip: compare F8 voices across all six starter personalities and report which names sound alike. Final audible fidelity remains open.

Findings (Codex, 2026-10-14):
- Normal, peppy, snooty -> spec 4 / sequence 245.
- Lazy, jock -> spec 2 / sequence 243.
- Cranky -> spec 3 / sequence 244.
- This is retail's three-way grouping; six different personality voices are not expected.
- app/src/sound.rs now shares play_voice between live typing and an ignored real-disc probe.
- The probe renders the same phrase through specs 2 -> 3 -> 4 -> 2, keeping one engine to exercise sequence replacement. All four clips are non-silent and finite; PCM differs across the three sequence types. This establishes different generated output, not listening fidelity.
- Probe RMS by pass: 0.05345, 0.06887, 0.05406, 0.05493; peaks 0.30447, 0.40650, 0.25233, 0.31624. Samples are in the Windows temp directory, bevy-crossing-voices (never committed).
- Passed: cargo check -p bevy-crossing; cargo clippy -p bevy-crossing --all-targets -- -D warnings; cargo fmt --all --check; the ignored sound::tests::probe_villager_voice_types with Philip's disc.
- Local qwen2.5-coder:7b reviewed the diagnostic plan at Philip's request; returned general concerns, no concrete defect established.

Re-run the disc check with BEVY_CROSSING_ISO pointing at the player's image:
cargo test -p bevy-crossing sound::tests::probe_villager_voice_types -- --ignored --nocapture

cargo build -p bevy-crossing passed; the updated debug executable is ready. The new game UI has not been observed in a live run for this update. Philip's F8 listening check remains necessary. Codex released this task for the listening follow-up; it is not marked complete.
