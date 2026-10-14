# audio-from-disc — the game's sound and music from the disc

M4; in progress. Updated 2026-10-14: the disc-backed sequence player,
hourly town music, and villager Animalese voices are implemented. Philip
playtested and confirmed hourly music works and villagers make Animalese
sounds. Gameplay sound effects and final validation remain.

- [x] Research: formats on the disc, what logic::audio translates, the player needed.
- [x] Step 1: tables, sound banks and sample decoding (logic::nas_bank, app sound_rom, loaded as GameDisc::sound). All 159 banks parse; 5203 samples decode (step/amplitude median 0.25, so the VADPCM decode is right). Sound effects are instruments played by the SE sequence, so step 2 needs the sequence player for most effects.
- [x] Step 2a: the sequence player (logic::nas_seq: group/subtrack/note scripts, voices, ADSR, vibrato, sweeps, 48 kHz mixer; simplified voice stealing, linear resampling, no reverb yet) and the disc provider (app sound_rom::DiscSound, tables from the DOL). Music sequences render to WAV; Philip subsequently confirmed hourly town music works in game (2026-10-14).
- [x] Step 2b: plays through Bevy (app sound.rs: the engine as a streaming source on the audio thread; Engine::process runs the game layer's AudioPort commands as Nap_Process1Command). Starts the effect player (seq 242) and the hourly town music (Sou_BgmStart, fine-weather mutes). Hourly music playtested by Philip on 2026-10-14.
- [x] Step 3: villager Animalese (`logic::na_voice`), preserving Na_VoiceSe/Na_SpecChange and Sou_VoiceMake slot volume/pitch; speaker-specific voice sequences and per-character phoneme lookahead feed the voice queue from app/src/msg_window.rs into app/src/sound.rs. Newlines use 0x84 and past-the-end uses 0xFF. Philip confirmed villagers make Animalese sounds on 2026-10-14.
- [ ] Complete and playtest gameplay sound effects (footsteps, UI, mailbox, etc.). Trigger queue and SE sequence playback are implemented; full effect coverage is not signed off.
- [x] Music (hourly town BGM): Philip confirmed it works in game on 2026-10-14.
- [x] Live listening check by Philip: hourly music and villager voices are audible (2026-10-14).
- [ ] Compare voice/music fidelity with the original. Philip has only heard one villager voice type so far (2026-10-14). Follow-up: villager-voice-types.md; F8 cycles the same line across actual residents, including those indoors. The disc probe produces non-silent, different output for sequences 243/244/245 and after switching back, but listening fidelity remains open.
- [x] Pre-commit gate (Codex, 2026-10-14): cargo check --workspace, cargo clippy --workspace --all-targets -- -D warnings, cargo fmt --all --check, and cargo test --workspace all passed. The separate ignored real-disc voice probe also passed.
- [ ] Finish remaining sound-effect and voice-fidelity playtests.

Voice translation evidence (Claude transcript supplied by Philip, 2026-10-14):
tanboin_henkan, chouboin_henkan, connect_check and boin_shiin_check were
generated mechanically from the C after Qwen omitted connect_check. Claude
reports an exhaustive comparison matching all 590,080 reference cases.
This records Claude's result; the comparison was not rerun for this notes update.

Known limitations / follow-ups: simplified voice stealing, linear resampling,
no reverb, indoor music, Na_MelodyVoice, special NPC voice specs, and handling
non-ASCII dialogue characters (currently mapped to silence in msg_window).
The normal villager voice path is implemented; these do not block its playback.

Research (claude, 2026-10-13): all audio is in audiorom.img (8.3 MB: banks, VADPCM samples, sequences), played by jaudio_NES (~37k lines C in src/static/jaudio_NES). logic::audio has only the Na_* game layer. Options given to Philip: A) SE samples only (bank reader + VADPCM decode, no music), B) full engine port (very large). Initially parked for budget while Philip chose pause-menu first; subsequently resumed with the bank decoder, sequence player, streaming playback and Animalese work above.
