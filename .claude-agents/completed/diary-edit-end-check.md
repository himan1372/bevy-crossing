# Diary edit-end check + board-type correction

2026-11-02, claude. Philip: "either" (diary check or audio listening); diary chosen.

Finding: mEE_TYPE_BOARD is index 0 = the two-answer window (lat_sentaku2_*; char -45,43; answer 65,-74; question 0,0; answers Yes / Rewrite). The mail check shipped in 9c2dc726 wrongly used the three-answer layout (index 1). Diary uses the same type (m_diary_ovl.c mEE_TYPE_BOARD). The diary currently shows a plain-text "Is this OK?" row.

- [x] Kernel: answer count parameter; mail uses 2 answers.
- [x] Shared disc renderer for the check window (mail + diary).
- [x] Diary Confirm mode uses the kernel and the disc windows (Yes saves, Rewrite/B edits again).
- [x] Gates, Temp QA for mail and diary, notes/roadmap/comms, commit.

## Completion 2026-11-02, claude

Correction: the mail check shipped in 9c2dc726 used the three-answer layout and a Throw it out answer; mEE_TYPE_BOARD (index 0) is the two-answer lat_sentaku2 window (Yes / Rewrite, B = Rewrite). Kernel EditEndCheck now takes an answer count (new() = 2; with_answers(3) for the other types; 3 own tests pass). Shared renderer app/src/edit_end_chk_ui.rs (both layouts) is used by mail_disc::draw_check and the new player_room/diary/render.rs::draw_check. The diary's plain-text Is-this-OK row is replaced: F4 -> disc question window, accept -> answers, Yes saves the month and closes, Rewrite/B reopens the editor on the same text with the cursor at the start (source resets cursor_idx). The diary's own PC help rows are hidden while confirming (they covered the answers).

Verified in Temp/bevy-inv-mail-20261102 with synthetic saves (real save SHA256 0495927D...BB1BD9 unchanged): mail two-answer window, cursor clamped at Rewrite, Rewrite returns to the editor, Yes added the letter and used one sheet; diary question and answer windows, Rewrite back to the editor, Yes stored November text and closed with no leftover entities. check/clippy/fmt/capture/default/release pass; only the kernel's own tests were run. The final help-row removal was a tiny change after the last diary screenshot and was not re-captured.

Not done: notice board / design / other mEE types; diary Escape-in-editor discard unchanged; Philip acceptance.
