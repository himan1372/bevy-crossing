# Letter writing on the retail character grid

Claimed by claude 2026-10-27. Philip chose "letters editor next" after diary-retail-grid.

Plan: 1) generalise the `logic/src/diary.rs` text kernel with `Limits` (diary 992 bytes/31 lines,
letter body `MAIL_BODY_LEN` 192 bytes/`mBD_BODY_LINE_NUM` 6 lines). 2) `mail.rs` compose view uses the
grid (Enter=A, Bksp=B, F1/F2/F3, Ctrl+arrows text cursor, F4 sends); recipient moves to PgUp/PgDn
because arrows now drive the grid. 3) `mail_disc.rs` draws the grid under a lifted paper.
Source: m_editor_ovl.c letter field (1453-1500), m_board_ovl.h (6 lines, 192 px).

## Implemented (2026-10-27, claude)

Kernel: `logic/src/diary.rs` text functions now take `Limits` (`DIARY` 992 bytes/31 lines,
`LETTER` 192 bytes/6 lines = `MAIL_BODY_LEN`/`mBD_BODY_LINE_NUM`); no disc.

App: `mail.rs` compose view (stationery Write and Rewrite) hosts the shared grid: arrows = grid,
Enter = A, Backspace = B with retail repeat, Space, F1/F2/F3, Ctrl+arrows text cursor (Down at
the end adds a newline, Right a space), F4 = Start sends, Esc cancels (stationery kept).
The addressee moved to Page Up/Down because the arrows drive the grid; the old 200-char ASCII
cap is now 192 bytes/6 lines. `mail_disc.rs` rolls the paper up, shows the five lines around
the cursor and the keyboard below; hints at the top.

Verified (Temp, disc, synthetic save with museum contact and stationery): grid key, typed text,
Ctrl+Down newline, PgDn addressee, F4 sent body `hello friend?\nbye` to the Museum on paper 3,
stationery consumed, saved in the letters; Esc left the stationery and no letter; overfilling
text stopped at 6 wrapped lines (177 chars, trailing space trimmed); screenshots inspected; no
WARN/ERROR in logs. Rewrite shares the same code but was not driven (pocket-letter navigation).
The last 6-unit row shift (28 to 22) was not re-screenshotted. Gates: see comms.

Not done: X accent key, paper roll-in animation, voices, separate header/footer fields.
