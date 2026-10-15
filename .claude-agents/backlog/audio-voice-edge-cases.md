# audio-voice-edge-cases — M4 step 7

Implemented by codex 2026-10-15; listening acceptance pending.
Philip authorized continuation after successful mixer playtest (a1f6295).

- [x] Audit charset, size/mood, deferred syllables/melody and applicable speakers.
- [x] Implement current villager message controls and melody requests.
- [x] Document unavailable speaking interactions and port timing adaptations.
- [x] Final compile/lint/format/build passed; no tests/probes added/run.
- [x] Philip accepts ordinary voices, emotion/size changes and pause/page behavior.
- [ ] Philip: accented letters and melody notes (not encountered during playtest).
- [ ] Combined audio acceptance (step 8).

## Changes

- messages::char_of now maps the missing Latin accents/ligatures and Latin
  symbols from m_font.h, including characters in ASCII punctuation positions.
  Both disc text decoding and Unicode-to-font lookup use this map.
  ac_code_opt lets audio leave unsupported Unicode silent rather than
  speaking the fallback question-mark glyph. Precomposed glyphs are supported;
  arbitrary Unicode/combining-mark normalization is not added.
- Message controls 75..79 and 97 retain normal/angry/sad/fun/sleepy/gloomy
  status. Controls 84/90 retain one-character and current-line scale.
  Private markup carries these through pages; strip removes them for plain UI.
  Non-normal status is carried to continued pages; each page starts its local
  controls at normal size. Text geometry/width uses the same scale as speech.
- Reveal converts supported characters through the font mapping; newlines use
  0x84, control/end boundaries 0xFF. Speech size is rounded/clamped from
  character scale * line scale / 32 (isotropic original font formula).
  Mood is driven by original message controls, not the port's guessed NPC mood.
- Voice status commands reset original wobble counters, including repeated
  explicit status commands. Default size 32/normal mood and existing spec
  mapping remain. The original fixed large/small volume branches (4.0/0.4)
  are retained; the accepted mixer protection remains active.
- Na_VoiceSe f=2 and Sou_SpecialRoutine10 now emit pending second syllables
  during idle frames, preserving original conversion/repeat behavior.
  Requests run at this port's 30 Hz game cadence; new input suppresses idle
  processing for a frame. Page changes/close cancel pending syllables and
  reset message history, avoiding old-speaker leakage across instant pages.
- Corrected r29 to the converted phoneme, as in C; punctuation updates both
  active voice slots even when it does not start a new syllable.
- The music-note glyph requests Na_MelodyVoice. The original font overrides
  its voice ID to 0 for size-32 text colored (50,60,50,255); other note styles
  retain the speaker ID. Deferred requests capture their own ID too.
  Speaker identity reads the
  actual aNPC_draw_data_c voice_type at offset 0x62 of 0x6C-byte disc records;
  it is not assumed to equal the resident table index.
- SoundRom reads melody_seq_offset/size from the player's runtime DOL and
  slices sequence 248. Engine prepares the fragment off the audio thread,
  relocates its 19 BE u16 addresses with original wrapping semantics, and
  writes only into the bounded SE melody work area.
  Na_MelodyVoice uses SE subtrack 6, note data [13,14,...], gain 1.3 and port 2
  speaker ID. Voice 0xFA is deliberately silent, matching C.
- Added safe owned 16-byte SFX state for sequence opcodes A0..A3. This replaces
  the melody's external pointer; no arbitrary memory-pointer command is added.
  Other melody/furniture instruments and editable town tunes are separate work.
- Replacing a melody first requests stop through port 0. The audio thread
  waits at least four updates and for consumption (-1) before copying/start;
  a half-second pending timeout and group-stop cleanup avoid stale starts.
  This nonblocking handoff adapts original Nap_WaitVsync to the existing mutex
  engine. Exact hardware/game/render interleaving is not claimed.
- Voice queue runs after RevealSet, ensuring text controls and characters are
  consumed in their original order in the same frame.

## Primary source

Original source root: ../shmanimal crossing/ACGC-PC-Port/
- include/m_font.h: charset and control-code enum.
- src/game/m_font_main.c_inc: per-character/line scale, reset after glyph/line,
  mFontSentence_animal_voice_se rounded size calculation.
- src/game/m_msg_sound.c_inc and m_msg_cursol.c_inc: lookahead/end boundaries,
  status commands, original speaker voice ID and spec selection.
- src/static/jaudio_NES/game/game64.c_inc: Na_VoiceSe f=0/f=2, Na_MessageStatus,
  Sou_SpecialRoutine10 and Sou_VoiceMake.
- game/melody.c: Na_MelodyVoice/Na_MelodyStart, sequence 248 and relocation.
- internal/track.c/sub_sys.c: SFX-state command semantics.
- include/ac_npc.h / actor/npc/ac_npc_ct.c_inc: voice_type record offset/assignment.

## Scope remaining

Normal residents are the current typed speakers. Shop transactions do not yet
provide a typed shopkeeper conversation; Nook/twins and other special NPC specs
are not assigned to unrelated UI messages. Their source-specific spec changes,
events, voice click/silent settings and speaking interactions remain explicit
gameplay/configuration follow-ups. Source-supported default spec mappings remain.

Sound-cut toggles and complete message cursor buffering/render timing still
need a fuller message-system port. This change adds the requested applicable
charset/size/mood/deferred/melody path, not every message control or audio feature.
Mixer resampling, reserve lists and hardware DSP remain in mix/fidelity notes.

## Listening checklist

1. Restart; F8 opens four pages per resident: normal speech, accented text,
   a default-color music note and a colored resident music note. E advances
   each page and closes after the fourth; F8 selects the next resident.
   Ordinary voices should remain recognizable and switch without hiccups.
2. During normal disc conversations, listen to accented names/words, questions,
   exclamations, emotion changes and large/small text as they occur.
3. Music-note glyphs should play a melody rather than silence: standard
   size/default color selects voice 0, styled notes retain the resident voice.
   Rapidly changing pages/speakers should not leave a delayed syllable behind.
4. Hold Run to speed reveal; check pauses, newlines, close/reopen and switching
   among all three normal voice sequences. Listen for abrupt tails/restarts.
5. Philip requested these F8 examples after accepting everything in the prior
   listening list except accents/music notes, which did not appear in play.
   Verify accents are visible and spoken; each music-note page should sound.
   No automated tests or disc probes added/run for this F8-only change.

## Validation / handoff

2026-10-15 final gate (including melody override): cargo check --workspace,
cargo clippy --workspace --all-targets -- -D warnings, cargo fmt --all --check,
cargo build --workspace and git diff --check passed.
No tests or probes added/run under the standing instruction; generated audio
and exact interleaving still require listening acceptance.

Commit/push authorized. Exclude unrelated .gitignore/savegame.json changes.
After feedback, proceed to combined audio acceptance; M4 audio remains open.

2026-10-15 F8 follow-up: examples added at Philip's request. Workspace check,
all-target clippy, formatting, build and diff whitespace passed. No automated
tests or disc probes run. Accents and both melody paths await Philip's listening.
