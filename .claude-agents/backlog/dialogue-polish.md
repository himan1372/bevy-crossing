# dialogue-polish — text appears and sounds like the game

M4; after ui-from-disc. Sources: m_msg_main/_normal/_appear/_wait/_cursol, m_msg_sound.

- [x] Letter-by-letter text at the game's speed (m_msg), control codes for pauses/speed.
- [x] Name plate per speaker; the next-page cursor.
- [x] Speaker Animalese voices: logic::na_voice plus per-character playback through app/src/msg_window.rs and app/src/sound.rs. Philip confirmed villagers make Animalese sounds on 2026-10-14.
- [x] Live voice listening check by Philip (2026-10-14).
- [x] Philip's F8 voice comparison (2026-10-14): voices sound good with no hiccups. F8 cycles a fixed line across actual residents; a speaker change resets typing even for identical text. See villager-voice-types.md.
- [x] Pre-commit workspace gate passed (Codex, 2026-10-14): check, clippy --all-targets -- -D warnings, fmt --all --check, and tests.
- [x] Remaining voice comparison playtest: Philip confirmed the voices sound good without hiccups (2026-10-14).

Progress (claude): typing at 30 chars/s (2x with Shift/B), CURSOR_SET_TIME pauses, colour codes (COLOR, SET_COLOR_CHAR; player name 75/95/155), the game's questions in the choice window with their real branches (NEXT_0..5). F7 (debug) talks to the nearest villager. Animalese voices are now connected to typing with the speaker's voice spec, phoneme lookahead, and Sou_VoiceMake volume/pitch. Newline phonemes use 0x84 and past-the-end uses 0xFF. Claude reports the four voice conversion/check functions matched the C across 590,080 cases (transcript supplied by Philip); this comparison was not rerun for this notes update. Philip confirmed hourly music and villager voices in game on 2026-10-14. Audio is no longer a blocker for the normal villager voice path. Melody voices, special NPC specs, and non-ASCII voice handling remain follow-ups in audio-from-disc.md.
