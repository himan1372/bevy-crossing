# dialogue-polish — text appears and sounds like the game

M4; after ui-from-disc. Sources: m_msg_main/_normal/_appear/_wait/_cursol, m_msg_sound.

- [x] Letter-by-letter text at the game's speed (m_msg), control codes for pauses/speed.
- [x] Name plate per speaker; the next-page cursor.
- [ ] Speaker voice blips (needs audio-from-disc, or a placeholder until then).
- [ ] Gate passes; live check; Philip playtest.

Progress (claude): typing at 30 chars/s (2x with Shift/B), CURSOR_SET_TIME pauses, colour codes (COLOR, SET_COLOR_CHAR; player name 75/95/155), the game's questions in the choice window with their real branches (NEXT_0..5). F7 (debug) talks to the nearest villager. Voice blips wait for audio-from-disc.
