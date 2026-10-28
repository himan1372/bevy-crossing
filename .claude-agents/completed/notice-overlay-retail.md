# Retail notice board overlay

Philip: "do two" (2026-10-28), second half: replace the plain notice board panel (`app/src/notice_board.rs`) with the retail overlay `m_notice_ovl.c`, drawn from the disc like the map (`completed/map-overlay-retail.md`).
Plan: source audit of `m_notice_ovl.c`, disc lookups, build on the same renderer, Temp QA, gates, note, comms.

## Implemented (2026-10-28, claude)
Source `m_notice_ovl.c` (`mNT_set_dl`, `mNT_set_page_dl`, `mNT_set_key_dl`, `mNT_set_day_strings_dl`). `app/src/notice_board/render.rs` now draws the retail window `kei_win_model` (was the diary paper),
the key display `kei_hyouji_model`, the page arrows `kei_win_yaji1T/2T` (only where a page lies that way), the stick marker `kei_win_stT_model` (resting picture `kei_win_st1`), the header "entry NN" and
"Month D, YYYY" in blue at 3/4 size (x 63 / 213-233, y 46), and the body in dark red (30,0,0) from (63, 63). The port's own read-mode hint lines are gone; edit/confirm keep their text hints and the
shop editor's keyboard. No logic change.
Verified (Temp capture copy, synthetic save with three posts, disc): the plaza board opened on the newest post ("entry 3", "October 28, 2026", left arrow, "latest entry / entry 1" keys, Write/Quit),
Enter opened the keyboard with the paper rolled up. No ERROR/panic. Gates: fmt, clippy -D warnings, capture build. Real save hash unchanged.
Not done: the slide in/out and the page-turn slide (the board appears and vanishes at once), the stick marker following the stick (always resting), the paper's own roll animation while editing (header is
hidden off the top when the paper is lifted, the port's text hints remain), `mNT_add_calc_dt` easing, the "Write/Quit" overlay's own button timing.
