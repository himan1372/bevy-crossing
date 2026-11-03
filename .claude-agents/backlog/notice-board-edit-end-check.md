# Notice board edit-end check

2026-11-02, claude. Continues codex's inventory/mail UI: letters (`mail.rs`) and the diary (`player_room/diary.rs`)
already confirm through `logic/src/edit_end_check.rs` + `app/src/edit_end_chk_ui.rs`. The notice board (`mEE` board type)
still commits directly. See completed/diary-edit-end-check.md ("Not done").

- [ ] Find the native type for the notice board in `m_edit_ovl` / `m_editEndChk_ovl` (answer count and layout).
- [ ] Hook `EditEndCheck` into `notice_board.rs` where the post is committed; Rewrite returns to the editor.
- [ ] Gate passes (check / clippy -D warnings / fmt), Temp-copy QA with a synthetic save; Philip acceptance stays open.

## Status 2026-11-02 (claude)
Code done: the notice board's own Post it / Keep writing / Never mind list is replaced by the shared `EditEndCheck::with_answers(3)`
(`mEE_TYPE_NOTICE`, window `win_data[1]`) drawn by `render::draw_check` through `edit_end_chk_ui`. Yes posts, Rewrite reopens the editor,
Throw it out returns to the last page (`mNT_move_Obey`). check / clippy -D warnings / fmt pass. **Not run in the game**: no Temp-copy QA of
F4 -> window -> each answer yet; the window's depth (z 220) over the board is a copy of the diary's. Philip acceptance open.

## QA 2026-11-02 (claude)
Driven in `Temp/bevy-notice-20261102` (capture exe copy, synthetic save, board at world (0.0, -24.5)): F4 -> the question slides in and blinks ("Is this OK?"), a first accept opens the three answers
over the paper (depth correct), Rewrite reopened the editor on the same text (one more char typed, F4 again), Yes posted exactly one notice (4 chars, saved `notices` has 1 entry), Throw it out on a
second draft returned to the last page with no new notice. Real save untouched. Not driven: Escape-as-Rewrite, the first-job notice finish, a full six-line post. Philip acceptance open.
