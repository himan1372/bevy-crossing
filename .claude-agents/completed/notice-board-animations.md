# Notice board animations

Philip chose "2" (candidate: notice/map animations), 2026-10-28. Source: `m_notice_ovl.c`
(`mNT_Play_page_move`, `_to_write`, `_to_read`, `mNT_move_Move`, `mNT_set_dl`) and
`m_submenu_ovl.c::mSM_move_Move`.

Plan: move the map's `Slide` (mSM_move_menu) into `logic/src/menu_slide.rs` and use it for both;
`logic/src/notice_board.rs::Motion` = the page turn / to-write / to-read states; the app slides the
board in and out, turns pages (one page eased, several as 74 px/frame hops), slides the draft page in
from the right and the key hints out below when writing, and back when the check is cancelled.

Not planned: the cursor-following roll (the port shows a fixed five-line window above the keyboard).

## Done (2026-10-28)
- `logic/src/menu_slide.rs`: `mSM_move_Move` (in from the top, out upward) moved out of `town_map.rs`; the map uses it too (`Slide::close`).
- `logic/src/notice_board.rs::Motion`: `mNT_Play_page_move` (one page eased with `add_calc`, 2-3 hops at 74 px/frame for longer jumps), `_to_write` (blank page in from the right, hints drop to -100) and `_to_read` (page in from the left, hints back). Three own tests.
- App: `notice_board/render.rs` builds the arriving and leaving pages and the hints as separate groups; `animate` moves them every frame (root y = slide, pages x = motion, hints y = control - 2 * slide). Input is ignored while the board slides or a page turns; the world gets the input back when the closing slide ends. The keyboard and the Edit/Confirm help rows appear once the page has settled.
- Driven in a Temp copy of the capture exe (synthetic save, three posted notices): slide in and out, one-page turn, jump to first page, write (page and hints), cancel at the end check (page back, draft page leaves right). Real save hash unchanged.

Not done: the cursor-following roll (the port shows a fixed five-line window and lifts the paper a fixed 100 px while editing; the lift snaps when the end check opens), the keyboard's own slide in, the page-turn sound per hop (still the existing cursor sound), the map overlay has no new animation (it already had slide, glide and pulse).
