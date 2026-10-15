# audio-from-disc — the game's sound and music from the disc

M4; in progress. Updated 2026-10-15: the disc-backed sequence player,
hourly town music, and villager Animalese voices are implemented. Philip
playtested and confirmed hourly music works and villagers make Animalese
sounds. M4 audio remains open: lifecycle/action/ambience/mixer listening and coverage,
hardware fidelity, voice edge cases and combined acceptance still need work. The ordered plan and completion criteria are in ../ROADMAP.md.

- [x] Research: formats on the disc, what logic::audio translates, the player needed.
- [x] Step 1: tables, sound banks and sample decoding (logic::nas_bank, app sound_rom, loaded as GameDisc::sound). All 159 banks parse; 5203 samples decode (step/amplitude median 0.25, so the VADPCM decode is right). Sound effects are instruments played by the SE sequence, so step 2 needs the sequence player for most effects.
- [x] Step 2a: the sequence player (logic::nas_seq: group/subtrack/note scripts, voices, ADSR, vibrato, sweeps, GameCube-rate mixer (32028 Hz, 140 samples/update); simplified voice stealing, linear resampling; CPU wet bus added 2026-10-15) and the disc provider (app sound_rom::DiscSound, tables from the DOL). Music sequences render to WAV; Philip subsequently confirmed hourly town music works in game (2026-10-14).
- [x] Step 2b: plays through Bevy (app sound.rs: the engine as a streaming source on the audio thread; Engine::process runs the game layer's AudioPort commands as Nap_Process1Command). Starts the effect player (seq 242) and the hourly town music (Sou_BgmStart, fine-weather mutes). Hourly music playtested by Philip on 2026-10-14.
- [x] Step 3: villager Animalese (`logic::na_voice`), preserving Na_VoiceSe/Na_SpecChange and Sou_VoiceMake slot volume/pitch; speaker-specific voice sequences and per-character phoneme lookahead feed the voice queue from app/src/msg_window.rs into app/src/sound.rs. Newlines use 0x84 and past-the-end uses 0xFF. Philip confirmed villagers make Animalese sounds on 2026-10-14.
- [x] Player footsteps implemented (Codex, 2026-10-14): animation contacts at frames 1/9, actual-motion gating, outdoor effective terrain, indoor floor metadata, alternating sample variants and walk/run volume. See player-footsteps.md.
- [x] Philip footstep playtest (2026-10-14): sounds work fine; menus and pushing against walls are silent.
- [x] UI/mailbox sounds implemented (Codex, 2026-10-14): pockets/pause, cursor changes, dialogue choices, shop transactions, mailbox panel/flag and letter read/compose/send feedback. Actor lid audio awaits a matching lid animation. Existing effect queue reused; all 29 probed gameplay/UI IDs render audio.
- [x] Philip: pockets and letter sounds accepted; asked to move on for now. Physical lid timing and general trigger stress remain pending. Historical correction details (ui-mailbox-sounds.md). First playtest found wrong exit/mailbox-menu mappings and a truncated register effect. Corrected inventory close to 0x31, mailbox opening to 0x8004 and actual inventory-mode closing to 0x31, and preserved MONO register routing on subtrack 14; probe reproduces old 0.437 s cutoff versus corrected 1.163 s under trigger traffic. Cursor movement and pocket opening are already confirmed good.
- [x] Music (hourly town BGM): Philip confirmed it works in game on 2026-10-14.
- [x] Live listening check by Philip: hourly music and villager voices are audible (2026-10-14).
- [x] Villager voice comparison: Philip playtested F8 and reports the voices sound good with no hiccups (2026-10-14). See villager-voice-types.md.
- [x] Pre-commit gate (Codex, 2026-10-14): cargo check --workspace, cargo clippy --workspace --all-targets -- -D warnings, cargo fmt --all --check, and cargo test --workspace all passed. The separate ignored real-disc voice probe also passed.
- [ ] audio-trigger-lifecycle: implemented using TrgSeManager, corrected command opcodes, audio-thread port completion and pending-start cleanup, priority/singleton management and MONO isolation. Compile/clippy/fmt/build passed; live overlap/completion listening remains pending.
- [x] audio-scene-music: implemented quiet player house, tier/closing shop themes and current-hour town restoration plus conversation/menu gain ramps. Compile/clippy/fmt/build passed; Philip reports it works perfectly. Weather masks verified/reused but live weather waits for simulation.
- [ ] audio-action-coverage: source matrix and available hooks implemented for doors, rod/net/tools, quest choices/handoffs and nearby surface-aware NPC contacts. Listening pending. Reel-fight loop, full catch fanfare/animation timing and retail door visuals await gameplay support.
- [ ] audio-world-ambience: implemented runtime river/shore metadata, localized waterfall fallback, supported AI insect loops, stable four-voice level slots, distance/pan and indoor/exit cleanup. Philip confirms playback works; cleanup/stress acceptance, pond/frogs, additional insect events and weather remain follow-ups.
- [ ] audio-mix-and-fidelity: CPU source-based reverb and indoor step sends, positional world actions/NPC pan, peak meters and linked stereo protection implemented. Accepted gains preserved; listening pending. Filtered resampling/reserve-list allocator audit documented; hardware gaps remain. See audio-mix-and-fidelity.md.
- [ ] audio-voice-edge-cases: charset, message size/mood, deferred/melody paths and applicable special speakers.
- [ ] audio-acceptance: combined retail listening, overlap/decay checks, pause/load/scene transitions and missing-disc fallback before M4 audio closes. Full ordering and acceptance criteria: ../ROADMAP.md.

Voice translation evidence (Claude transcript supplied by Philip, 2026-10-14):
tanboin_henkan, chouboin_henkan, connect_check and boin_shiin_check were
generated mechanically from the C after Qwen omitted connect_check. Claude
reports an exhaustive comparison matching all 590,080 reference cases.
This records Claude's result; the comparison was not rerun for this notes update.

Known limitations / follow-ups: simplified voice stealing, linear resampling,
fixed-point/filter/Haas/surround DSP, reserve-list allocation, Na_MelodyVoice,
special NPC voice specs, and handling
non-ASCII dialogue characters (currently mapped to silence in msg_window).
The normal villager voice path is implemented; these do not block its playback.

Research (claude, 2026-10-13): all audio is in audiorom.img (8.3 MB: banks, VADPCM samples, sequences), played by jaudio_NES (~37k lines C in src/static/jaudio_NES). logic::audio has only the Na_* game layer. Options given to Philip: A) SE samples only (bank reader + VADPCM decode, no music), B) full engine port (very large). Initially parked for budget while Philip chose pause-menu first; subsequently resumed with the bank decoder, sequence player, streaming playback and Animalese work above.

2026-10-14 listening update: Philip says letters sound okay and asked to move
on for now; pockets and letters accepted. Scene music implemented next (see
ROADMAP step 3 / audio-scene-music.md); general trigger stress and physical
mailbox lid timing remain pending rather than inferred accepted.

2026-10-15: Philip confirms ambience playback works. Requested mix/fidelity
(step 6) now has initial reverb/spatial/protection implementation and a focused
listening checklist. This does not close ambience coverage, trigger stress or
hardware fidelity. Next: listening fixes, voice edge cases (7), combined acceptance (8).
