# Diary editing on the retail character grid

Claimed by claude 2026-10-27. Philip chose candidate 2 of the handoff.

Plan: 1) `logic/src/diary.rs`: pure in-text cursor kernel (insert/backspace/left/right/
up/down with the source's end-of-text rules: Right adds a space, Down adds a newline).
2) `app/src/player_room/diary.rs` Edit mode hosts `shop_nook/editor.rs` (grid, L/Z/Y toggles,
Start to confirm), paper lifted for the keyboard. 3) Gates, Temp QA, docs, comms.

Source: `m_editor_ovl.c` mED_move_cursol_* (1000-1060), mED_edit_func_diary (1533),
`m_diary_ovl.c` mDI_Play_read_to_write_scroll (460).

## Implemented (2026-10-27, claude)

Kernel `logic/src/diary.rs` gained the in-text cursor from `m_editor_ovl.c`
(`mED_input_multi_line`, `mED_backspace_func`, `mED_move_cursol_left/right/upper/lower`):
`insert` (992 bytes, 31 lines), `backspace`, `left`, `right` (past the end adds a space),
`upper`/`lower` (keep the pixel column; Down at the very end adds a newline) and
`line_ranges`/`cursor_line`. Pure, disc-free.

App: diary Edit mode (`app/src/player_room/diary.rs`) now hosts the shared retail grid
(`shop_nook::editor` made `pub(crate)`): arrows move the grid, Enter = A types the selected
key, Backspace = B (retail hold repeat), Space, F1 caps, F2 layout, F3 letters/signs/marks,
Ctrl+arrows move the text cursor, F4 = Start opens Yes/Rewrite, Esc discards. Typed letters
still insert. The paper rolls up 100 units (`diary_ovl->_28 = -100`) and shows five lines
with the cursor; keys cleanly replace the old Enter-newline/Ctrl+Enter scheme (newline is
Ctrl+Down at the end, as in the source).

Verified (Temp, disc, synthetic home with a diary on a table): grid key, caps key, space,
newline, typed text, mid-text insert and Backspace, 17-byte entry written exactly and only
`home_furniture.diary` (+ playtime) changed; discard via Esc leaves the saved notebook equal;
long wrapped text scrolls with the cursor; screenshots inspected; no WARN/ERROR in the disc
logs. Gates: fmt, clippy -D warnings, capture, default and release builds; no tests.

Not done: the X accent key, the real scroll-in animation of the paper, voices on input,
letters (writing/handbill editor) which still need their own host.
