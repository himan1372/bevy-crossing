# audio-from-disc — the game's sound and music from the disc

M4; research first. How the original stores audio (sound banks, sequenced
music) and how much logic::audio already covers.

- [x] Research: formats on the disc, what logic::audio translates, the player needed.
- [x] Step 1: tables, sound banks and sample decoding (logic::nas_bank, app sound_rom, loaded as GameDisc::sound). All 159 banks parse; 5203 samples decode (step/amplitude median 0.25, so the VADPCM decode is right). Sound effects are instruments played by the SE sequence, so step 2 needs the sequence player for most effects.
- [x] Step 2a: the sequence player (logic::nas_seq: group/subtrack/note scripts, voices, ADSR, vibrato, sweeps, 48 kHz mixer; simplified voice stealing, linear resampling, no reverb yet) and the disc provider (app sound_rom::DiscSound, tables from the DOL). Music sequences render to WAV with sound that varies like music; Philip to listen.
- [x] Step 2b: plays through Bevy (app sound.rs: the engine as a streaming source on the audio thread; Engine::process runs the game layer's AudioPort commands as Nap_Process1Command). Starts the effect player (seq 242) and the hourly town music (Sou_BgmStart, fine-weather mutes). Philip to listen.
- [ ] Sound effects (footsteps, UI, mailbox, etc.).
- [ ] Music (hourly BGM).
- [ ] Gate passes; live check; Philip playtest.

Research (claude, 2026-10-13): all audio is in audiorom.img (8.3 MB: banks, VADPCM samples, sequences), played by jaudio_NES (~37k lines C in src/static/jaudio_NES). logic::audio has only the Na_* game layer. Options given to Philip: A) SE samples only (bank reader + VADPCM decode, no music), B) full engine port (very large). Parked for budget; Philip chose pause-menu first.
