# audio-from-disc — the game's sound and music from the disc

M4; research first. How the original stores audio (sound banks, sequenced
music) and how much logic::audio already covers.

- [x] Research: formats on the disc, what logic::audio translates, the player needed.
- [x] Step 1: tables, sound banks and sample decoding (logic::nas_bank, app sound_rom, loaded as GameDisc::sound). All 159 banks parse; 5203 samples decode (step/amplitude median 0.25, so the VADPCM decode is right). Sound effects are instruments played by the SE sequence, so step 2 needs the sequence player for most effects.
- [ ] Sound effects (footsteps, UI, mailbox, etc.).
- [ ] Music (hourly BGM).
- [ ] Gate passes; live check; Philip playtest.

Research (claude, 2026-10-13): all audio is in audiorom.img (8.3 MB: banks, VADPCM samples, sequences), played by jaudio_NES (~37k lines C in src/static/jaudio_NES). logic::audio has only the Na_* game layer. Options given to Philip: A) SE samples only (bank reader + VADPCM decode, no music), B) full engine port (very large). Parked for budget; Philip chose pause-menu first.
