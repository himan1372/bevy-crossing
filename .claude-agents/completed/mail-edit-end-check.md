# Mail edit-end confirmation ("Is this OK?")

2026-11-02, claude. Philip chose a source-backed M4 gap (step 3 option 2) after inventory-mail-polish. F4 currently commits a letter directly.

Source: m_editEndChk_ovl.c (mEE_TYPE_BOARD: question window lat_kakunin_*, answer window lat_sentaku_*, answers Yes / Rewrite / Throw it out), m_board_ovl.c mBD_move_Play/Obey (editor ends -> check opens, center_line 2; Yes commits and consumes one stationery sheet; No/Rewrite reopens the editor; Cancel/Throw out ends the board with no sheet used), mED_end_edit_func (editor leaves bottom).

Behaviour: question window slides in from the bottom and blinks (alpha 0..255 over 30 ticks); first A/Start expands the answer window (scale +0.2 a tick); up/down move when scale is 1; second A/Start picks; B picks Rewrite at once; sounds 0x1003 for Throw it out and B, NA_SE_MENU_EXIT otherwise, NA_SE_CURSOL on cursor moves.

- [x] Disc-independent kernel logic/src/edit_end_check.rs (own tests only).
- [x] mail.rs: F4 opens the check; Yes commits via finish_letter; Rewrite returns to the editor; Throw it out discards like Escape.
- [x] mail_disc.rs: keyboard slides out and paper rolls back while checking; question/answer windows from disc models and font.
- [x] check/clippy/fmt/capture/default/release; Temp QA with synthetic save (all three answers, B, numeric equality, no leaked keys).
- [x] Notes, roadmap, comms, complete claim, commit.

Out of scope: diary/design/notice board checks (same overlay, other types), no-disc fallback keeps the old direct F4 commit only if no disc UI exists.

## Completion 2026-11-02, claude

Kernel logic/src/edit_end_check.rs (EditEndCheck: slide-in, input gate, blink, scale, cursor, answers, sounds; own 2 tests pass). mail.rs: F4 opens the check (falls back to the old direct commit when the disc UI symbols are absent); Yes commits via finish_letter, Rewrite returns to the editor with the text kept and the cursor at the start (source resets cursor_idx), Throw it out discards like Escape, B picks Rewrite. mail_disc.rs: draw_check builds the disc lat_kakunin/lat_sentaku windows each frame (blink alpha cached per value), the keyboard slides out and the rolled paper releases while checking, and slides back on Rewrite.

Verified in Temp/bevy-inv-mail-20261102 (synthetic save, real save SHA256 0495927D...BB1BD9 unchanged): question window and blink arrow, answers with cursor on Yes and on Throw it out, B -> editor returns with "hi" and cursor 0, Yes -> letter "yhi" to Museum added and exactly one stationery sheet used, Throw it out -> letters/pockets/conditions/Bells identical to baseline, no leftover entities. check/clippy/fmt/capture/default/release pass; only the kernel's own tests were run.

Not done: the help line still reads "F4: send"; the diary/notice/design checks (same overlay, other types) are separate; Rewrite of an existing letter and the answer window's text scale animation were not inspected frame by frame; keyboard-only mapping (Enter/Space/F4 accept, Escape = B) is the PC adaptation.
