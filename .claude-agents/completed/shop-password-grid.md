# Retail character-grid entry (shared editor overlay)

Unclaimed. Philip (2026-10-26): pull graphics from the disc rather than redraw
them; any rewritten logic must be a pure kernel in `logic/` that does not depend on
the disc so it can move to a new project; reuse rustimal-crossing kernels.

Findings: pinned/sibling `rust/src/name_editor.rs` has only buffer helpers and
constants (`buffer::clear`, `is_all_spaces`, entry types, 10x4 keyboard size);
its note says keyboard tables, insert/delete and the state machine stayed in C.
Password entry lives in `m_editor_ovl.c` (2635 lines): `mED_get_code`
(grid code lookup, line 63), cursor/stick area and movement (140-420), shift/
arrange/input-mode and convert (422-520), `mED_input_pw_chk_line`/`pw_make_line`
(already in `logic/password.rs`), pw table moves (1572-1760) and the per-key
button draws (`kai_sousa_*` models/textures, 1905-2300, loaded from the disc).
Letters and the diary use the same overlay.

Plan: 1) new `logic/src/editor_keyboard.rs`: grid tables, cursor/shift/mode
state machine and key outcomes as pure functions (data copied from C source,
no disc). 2) app overlay that draws the grid/buttons with disc models
(`disc_gfx`) and feeds the kernel from keyboard/gamepad, replacing the typed
adapter for passwords first, then letters/diary. 3) Gates, Temp QA, docs.

## Implemented (2026-10-26, claude)

Kernel: `logic/src/editor_keyboard.rs` (pure, no disc): the 10x4 key tables for
lower/upper case, QWERTY/alphabetical, signs and marks (`mED_get_code`), the grid
`Keyboard` (cursor, `step` with the original hold-repeat 16/12 frames, shift/arrange/
mode toggles, `selected` skipping empty keys), `ButtonRepeat` (fire, wait 30, then 6,
then 2 frames as `_10/_0F/_0E`), and `Field<N>` for the receiver names (insert at the
cursor, backspace, left, right-adds-a-space). `logic/src/password.rs` gained
`cursor_left/right` and `change_line` for the code window.

App: `shop_nook/editor.rs` builds the retail editor from the disc: `kai_sousa_*`
panel, shita/controller/mojiban, L (caps/small label, blue/red frame), R/space,
A/B/X/Y (pressed faces through segment 8), D-pad faces, the tilting stick,
the letter-mode indicator, and the 40 key faces from `kai_sousa2_v` with `testbutton`
mirrored, letters in the disc font, the chosen letter on the board, the space mark from
`lat_sousa_spT_model`. `shop_nook/password/ui.rs` hosts both windows on it: Say code
over `pwd_win_*` ("Enter a secret code.", the two yellow lines and a cursor bar) and the
receiver over `pas_win_*` ("Enter a receiver.", framed town/name, start arrow once the
town is filled). Keys: arrows = grid, Enter = A, Backspace = B, Ctrl+arrows = text
cursor/line, F1 = L (case), F2 = Z (layout), F3 = Y (letters/signs/marks), F4 = Start,
Space = the space key, Esc = back out; typed letters still insert. Start is now the only way
to finish (the code window no longer auto-finishes at 28 letters), the receiver needs a town,
then a player name (original `mED_edit_func_pw_make` gates), sounds 0x411/0x35/0x1034/0x5F/
0x37/0x1003 from the source.

Verified (Temp, disc, synthetic save): both windows screenshot against the retail look
(controller, keys, labels, key icon/package icon); arrows move the red/blue selection;
Enter types; F1 flips to the Caps/Small face and upper-case keys; F3 switches to the
punct keys; mid-line insert, Backspace, Ctrl+Left, Ctrl+Down all behave; codes redeem
through the new window (F4) and the receiver flows to the confirmation message. Not done:
the X button's exchange (accent) feature and its plate stays empty, controller pad
rumble/other-pad support, the original scrolling window texture, the cursor pulse, the
character voice on input, and pressed-face display for R.

